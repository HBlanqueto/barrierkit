use ctappearance::material::Material;
use ctdesign::{Rgba, SCROLL_EDGE};
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use objc2::{MainThreadMarker, MainThreadOnly, msg_send, sel};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSColor, NSGlassEffectViewStyle, NSSplitViewController,
    NSSplitViewItem, NSView, NSViewController, NSVisualEffectBlendingMode, NSVisualEffectMaterial,
    NSVisualEffectState,
};
use objc2_core_foundation::CFData;
use objc2_core_graphics::{CGColor, CGDataProvider, CGImage};
use objc2_foundation::{NSArray, NSNumber, NSObjectProtocol, NSString, ns_string};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use objc2_quartz_core::{CAGradientLayer, CALayer, CATransaction, kCACornerCurveContinuous};

use super::view::{Effect, Glass, GlassGroup, PaneSplit};
use crate::platform::backdrop::{Chrome, Effect as Declared, strip_rect};
use crate::platform::window_style::Edge;

pub(super) fn material(material: Material) -> NSVisualEffectMaterial {
    match material {
        Material::Titlebar => NSVisualEffectMaterial::Titlebar,
        Material::Selection => NSVisualEffectMaterial::Selection,
        Material::Menu => NSVisualEffectMaterial::Menu,
        Material::Popover => NSVisualEffectMaterial::Popover,
        Material::Sidebar => NSVisualEffectMaterial::Sidebar,
        Material::HeaderView => NSVisualEffectMaterial::HeaderView,
        Material::Sheet => NSVisualEffectMaterial::Sheet,
        Material::WindowBackground => NSVisualEffectMaterial::WindowBackground,
        Material::HudWindow => NSVisualEffectMaterial::HUDWindow,
        Material::FullScreenUi => NSVisualEffectMaterial::FullScreenUI,
        Material::ToolTip => NSVisualEffectMaterial::ToolTip,
        Material::ContentBackground => NSVisualEffectMaterial::ContentBackground,
        Material::UnderWindowBackground => NSVisualEffectMaterial::UnderWindowBackground,
        Material::UnderPageBackground => NSVisualEffectMaterial::UnderPageBackground,
    }
}

// The kit places from the top left of the window in logical pixels; AppKit's views start at the bottom left.
pub(super) fn frame((x, y, width, height): (i32, i32, i32, i32), container_height: f64) -> NSRect {
    NSRect::new(
        NSPoint::new(
            f64::from(x),
            container_height - f64::from(y) - f64::from(height),
        ),
        NSSize::new(f64::from(width), f64::from(height)),
    )
}

pub(super) fn autoresizing(
    (_, y, _, height): (i32, i32, i32, i32),
    container_height: f64,
) -> NSAutoresizingMaskOptions {
    if y == 0 && f64::from(height) == container_height {
        NSAutoresizingMaskOptions::ViewHeightSizable
    } else {
        NSAutoresizingMaskOptions::ViewMinYMargin
    }
}

pub(super) fn effect(mtm: MainThreadMarker, material: Material, radius: f32) -> Retained<Effect> {
    let effect = Effect::new(mtm);
    effect.setMaterial(self::material(material));
    effect.setBlendingMode(NSVisualEffectBlendingMode::WithinWindow);
    effect.setState(NSVisualEffectState::Active);
    effect.setWantsLayer(true);
    if let Some(layer) = effect.layer() {
        layer.setCornerRadius(f64::from(radius));
        // SAFETY: AppKit's own constant.
        layer.setCornerCurve(unsafe { kCACornerCurveContinuous });
        layer.setMasksToBounds(true);
    }
    effect
}

pub(super) fn edge_points(edge: Edge) -> ((f64, f64), (f64, f64)) {
    match edge {
        Edge::Top => ((0.5, 1.0), (0.5, 0.0)),
    }
}

pub(super) fn edge_rect(
    edge: Edge,
    (x, y): (i32, i32),
    (width, _): (u32, u32),
    depth: u32,
) -> (i32, i32, i32, i32) {
    match edge {
        Edge::Top => (x, y, width as i32, depth as i32),
    }
}

// SwiftUI's sidebar scroll pocket under a full-screen bar, read from Resonance Swift (docs/barrierkit/09-titlebars.md#full-screen), with Core Animation's private
// backdrop layers: a half-scale variable blur of the rows, and over it the background behind the rows, captured below them and shown
// through a gradient. Both run `SCROLL_EDGE.pocket_overrun` past the pocket's bottom; the blur's mask and the gradient each ramp over `pocket_ramp` there.
const POCKET_MASK_ROWS: usize = 128;

pub(super) struct Pocket {
    pub(super) capture: Retained<NSView>,
    pub(super) view: Retained<NSView>,
    tint: Retained<CALayer>,
}

impl Pocket {
    // None where Core Animation lacks the private backdrop layer; the sidebar then goes without a pocket.
    pub(super) fn new(mtm: MainThreadMarker, group: &str) -> Option<Self> {
        let group = NSString::from_str(group);
        let captured = backdrop(Some(&group))?;
        // SAFETY: `captureOnly` makes the backdrop feed its group without drawing.
        let _: () = unsafe { msg_send![&captured, setCaptureOnly: true] };
        let veil = backdrop(Some(&group))?;
        let gradient = CAGradientLayer::new();
        let (clear, veiled) = (
            CGColor::new_generic_rgb(1.0, 0.0, 1.0, 0.0),
            CGColor::new_generic_rgb(1.0, 0.0, 1.0, SCROLL_EDGE.pocket_veil),
        );
        // SAFETY: a CGColor is a Core Foundation object, and CAGradientLayer takes them as its colors.
        let colors = unsafe {
            NSArray::from_slice(&[
                &*(std::ptr::from_ref::<CGColor>(&clear).cast::<AnyObject>()),
                &*(std::ptr::from_ref::<CGColor>(&veiled).cast::<AnyObject>()),
            ])
        };
        // SAFETY: the colors are the two CGColors above, and `destIn` a compositing filter name.
        unsafe {
            gradient.setColors(Some(&colors));
            gradient.setCompositingFilter(Some(ns_string!("destIn")));
        }
        veil.addSublayer(&gradient);
        let blur = backdrop(None)?;
        let tint = CALayer::new();
        let host = |layers: &[&CALayer]| {
            let view = NSView::new(mtm);
            view.setWantsLayer(true);
            view.setAutoresizingMask(NSAutoresizingMaskOptions::ViewMinYMargin);
            let host = view.layer()?;
            layers.iter().for_each(|layer| host.addSublayer(layer));
            Some(view)
        };
        Some(Self {
            capture: host(&[&captured])?,
            view: host(&[&blur, &veil, &tint])?,
            tint,
        })
    }

    // Veils the detail from `left` to the right end with the window color, or nothing.
    pub(super) fn set_tint(&self, tint: Option<(f64, Rgba)>) {
        CATransaction::begin();
        CATransaction::setDisableActions(true);
        self.tint.setHidden(tint.is_none());
        if let Some((left, color)) = tint {
            let size = self.view.frame().size;
            self.tint.setFrame(NSRect::new(
                NSPoint::new(left, 0.0),
                NSSize::new((size.width - left).max(0.0), size.height),
            ));
            let color = NSColor::colorWithSRGBRed_green_blue_alpha(
                f64::from(color.r),
                f64::from(color.g),
                f64::from(color.b),
                SCROLL_EDGE.detail_veil,
            );
            self.tint.setBackgroundColor(Some(&color.CGColor()));
        }
        CATransaction::commit();
    }

    pub(super) fn remove(&self) {
        self.capture.removeFromSuperview();
        self.view.removeFromSuperview();
    }

    pub(super) fn set_frame(&self, frame: NSRect) {
        let (width, depth) = (frame.size.width, frame.size.height);
        let overrun = NSRect::new(
            NSPoint::new(0.0, -SCROLL_EDGE.pocket_overrun),
            NSSize::new(width, depth + SCROLL_EDGE.pocket_overrun),
        );
        let bounds = NSRect::new(NSPoint::new(0.0, 0.0), frame.size);
        CATransaction::begin();
        CATransaction::setDisableActions(true);
        self.capture.setFrame(frame);
        self.view.setFrame(frame);
        if let Some(layer) = self.capture.layer().as_deref().and_then(first_sublayer) {
            layer.setFrame(bounds);
        }
        // SAFETY: the sublayers are only read; the pocket made them.
        let layers = self
            .view
            .layer()
            .and_then(|host| unsafe { host.sublayers() });
        if let (Some(layers), Some(blur)) = (layers, variable_blur(depth)) {
            for layer in &layers {
                layer.setFrame(overrun);
            }
            if let Some(layer) = layers.firstObject() {
                // SAFETY: a CAFilter is what a layer's filters hold.
                unsafe { layer.setFilters(Some(&NSArray::from_slice(&[&*blur]))) };
            }
            if let Some(gradient) = Some(layers.objectAtIndex(1))
                .as_deref()
                .and_then(first_sublayer)
                .and_then(|layer| layer.downcast::<CAGradientLayer>().ok())
            {
                let height = overrun.size.height;
                gradient.setFrame(NSRect::new(NSPoint::new(0.0, 0.0), overrun.size));
                gradient.setStartPoint(NSPoint::new(0.5, SCROLL_EDGE.pocket_veil_start / height));
                gradient.setEndPoint(NSPoint::new(
                    0.5,
                    (SCROLL_EDGE.pocket_veil_start + SCROLL_EDGE.pocket_ramp) / height,
                ));
            }
        }
        CATransaction::commit();
    }
}

fn first_sublayer(layer: &CALayer) -> Option<Retained<CALayer>> {
    // SAFETY: the sublayers are only read.
    unsafe { layer.sublayers() }?.firstObject()
}

fn backdrop(group: Option<&NSString>) -> Option<Retained<CALayer>> {
    let class = AnyClass::get(c"CABackdropLayer")?;
    // SAFETY: CABackdropLayer is a CALayer subclass, made by CALayer's `layer` factory.
    let layer: Retained<CALayer> = unsafe { msg_send![class, layer] };
    // SAFETY: `scale` is the backdrop's capture resolution, a CGFloat; `groupName` an NSString or nil.
    unsafe {
        let _: () = msg_send![&layer, setScale: SCROLL_EDGE.backdrop_scale];
        let _: () = msg_send![&layer, setGroupName: group];
    }
    Some(layer)
}

fn variable_blur(depth: f64) -> Option<Retained<AnyObject>> {
    let bytes = CFData::from_bytes(&pocket_mask(depth));
    let provider = CGDataProvider::with_cf_data(Some(&bytes))?;
    // SAFETY: one 8-bit row per mask byte, and no decode array.
    let mask = unsafe {
        CGImage::mask_create(
            1,
            POCKET_MASK_ROWS,
            8,
            8,
            1,
            Some(&provider),
            std::ptr::null(),
            true,
        )
    }?;
    let class = AnyClass::get(c"CAFilter")?;
    // SAFETY: CAFilter's factory takes a filter type name and returns an autoreleased filter.
    let filter: Retained<AnyObject> =
        unsafe { msg_send![class, filterWithType: ns_string!("variableBlur")] };
    let set = |key: &NSString, value: &AnyObject| {
        // SAFETY: a CAFilter takes its inputs by key-value coding.
        let _: () = unsafe { msg_send![&filter, setValue: value, forKey: key] };
    };
    for key in [
        "inputRadius",
        "inputFade",
        "inputNormalizeEdges",
        "inputDither",
    ] {
        set(&NSString::from_str(key), &NSNumber::new_f64(1.0));
    }
    // SAFETY: a CGImage is a Core Foundation object, which key-value coding takes as an object.
    set(ns_string!("inputMaskImage"), unsafe {
        &*std::ptr::from_ref::<CGImage>(&mask).cast::<AnyObject>()
    });
    Some(filter)
}

// The same bytes as SwiftUI's mask for this depth: zero, then a `pocket_ramp` ramp ending at the pocket's bottom, then whole over the overrun.
fn pocket_mask(depth: f64) -> [u8; POCKET_MASK_ROWS] {
    let (ramp, height) = (SCROLL_EDGE.pocket_ramp, depth + SCROLL_EDGE.pocket_overrun);
    std::array::from_fn(|row| {
        let y = row as f64 / (POCKET_MASK_ROWS - 1) as f64 * height;
        (((y - (depth - ramp)) / ramp).clamp(0.0, 1.0) * 255.0) as u8
    })
}

pub(super) fn fade(effect: &Effect, edge: Edge) {
    let (whole, tail) = (SCROLL_EDGE.fade_whole, SCROLL_EDGE.fade_tail);
    let gray = |alpha| CGColor::new_generic_gray(0.0, alpha);
    let (start, end) = edge_points(edge);
    let layer = CAGradientLayer::new();
    layer.setFrame(effect.bounds());
    let (opaque, clear) = (gray(whole), gray(tail));
    // SAFETY: a CGColor is a Core Foundation object, and CAGradientLayer takes them as its colors.
    let colors = unsafe {
        NSArray::from_slice(&[
            &*(std::ptr::from_ref::<CGColor>(&opaque).cast::<objc2::runtime::AnyObject>()),
            &*(std::ptr::from_ref::<CGColor>(&opaque).cast::<objc2::runtime::AnyObject>()),
            &*(std::ptr::from_ref::<CGColor>(&clear).cast::<objc2::runtime::AnyObject>()),
        ])
    };
    // SAFETY: the colors are the three CGColors above.
    unsafe { layer.setColors(Some(&colors)) };
    let stops: Vec<_> = SCROLL_EDGE
        .fade_stops
        .iter()
        .map(|stop| NSNumber::new_f64(f64::from(*stop)))
        .collect();
    layer.setLocations(Some(&NSArray::from_retained_slice(&stops)));
    layer.setStartPoint(NSPoint::new(start.0, start.1));
    layer.setEndPoint(NSPoint::new(end.0, end.1));
    effect.setWantsLayer(true);
    if let Some(host) = effect.layer() {
        // SAFETY: a gradient layer is a layer.
        unsafe { host.setMask(Some(&layer)) };
    }
}

// NSGlassEffectView is macOS 26's: without it the controls paint their own fill.
pub(super) fn glass_available() -> bool {
    objc2::runtime::AnyClass::get(c"NSGlassEffectView").is_some()
        && objc2::runtime::AnyClass::get(c"NSGlassEffectContainerView").is_some()
}

pub(super) fn glass(
    mtm: MainThreadMarker,
    radius: f32,
    clear: bool,
    tint: Option<Rgba>,
) -> Retained<Glass> {
    let glass = Glass::new(mtm);
    // Setting `regular` turns the view into another glass than the default one AppKit's toolbar uses.
    if clear {
        glass.setStyle(NSGlassEffectViewStyle::Clear);
    }
    glass.setCornerRadius(f64::from(radius));
    if let Some(tint) = tint {
        glass.setTintColor(Some(&NSColor::colorWithSRGBRed_green_blue_alpha(
            f64::from(tint.r),
            f64::from(tint.g),
            f64::from(tint.b),
            f64::from(tint.a),
        )));
    }
    // A press lights the glass where it lands, as SwiftUI's toolbar does; AppKit has it from macOS 27.
    if glass.respondsToSelector(sel!(setEffectIsInteractive:)) {
        // SAFETY: the selector exists on this macOS and takes a BOOL.
        let _: () = unsafe { msg_send![&*glass, setEffectIsInteractive: true] };
    }
    glass
}

// Glass views of one chrome merge when they are near, as in a SwiftUI `GlassEffectContainer`.
pub(super) fn glass_group(
    mtm: MainThreadMarker,
    frame: NSRect,
    members: &[Retained<Glass>],
    spacing: f64,
) -> Retained<GlassGroup> {
    let group = GlassGroup::new(mtm);
    group.setFrame(frame);
    group.setSpacing(spacing);
    let holder = NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::ZERO, frame.size));
    for member in members {
        holder.addSubview(member);
    }
    group.setContentView(Some(&holder));
    group
}

fn empty_controller(mtm: MainThreadMarker) -> Retained<NSViewController> {
    let controller = NSViewController::new(mtm);
    controller.setView(&NSView::new(mtm));
    controller
}

// A sidebar or inspector split item is how AppKit draws that pane on the running macOS: glass on 26, the behind-window material before.
pub(super) fn pane(mtm: MainThreadMarker, inspector: bool) -> Retained<NSSplitViewController> {
    let split = NSSplitViewController::new(mtm);
    split.setSplitView(&PaneSplit::new(mtm));
    let content = NSSplitViewItem::splitViewItemWithViewController(&empty_controller(mtm));
    content.setCollapsed(true);
    if inspector {
        split.addSplitViewItem(&content);
        split.addSplitViewItem(&NSSplitViewItem::inspectorWithViewController(
            &empty_controller(mtm),
        ));
    } else {
        split.addSplitViewItem(&NSSplitViewItem::sidebarWithViewController(
            &empty_controller(mtm),
        ));
        split.addSplitViewItem(&content);
    }
    split
}

pub(super) fn place_pane(split: &NSSplitViewController, frame: NSRect) {
    if let Some(pane) = split
        .splitViewItems()
        .iter()
        .find(|item| !item.isCollapsed())
    {
        pane.setMinimumThickness(frame.size.width);
        pane.setMaximumThickness(frame.size.width);
    }
    split.view().setFrame(frame);
}

pub(crate) type PlacedGlass = ((i32, i32, i32, i32), u32, bool, Option<Rgba>);

impl Chrome {
    // The protocol's backdrop holds one radius and one variant; macOS draws a native view per glass, so a nested one keeps its own.
    pub(crate) fn glasses(&self, scale: f32) -> Vec<PlacedGlass> {
        let origin = self.placement(scale).origin;
        self.effects
            .iter()
            .filter_map(|(effect, area)| match *effect {
                Declared::Glass {
                    radius,
                    clear,
                    tint,
                    ..
                } => Some((strip_rect(area, origin, scale), radius, clear, tint)),
                _ => None,
            })
            .collect()
    }
}

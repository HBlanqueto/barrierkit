use ctappearance::material::Material;
use ctdesign::Rgba;
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

// A band that spans the whole height keeps both its margins while the window resizes; any other keeps its distance from the top.
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

pub(super) fn effect(
    mtm: MainThreadMarker,
    material: Material,
    blending: NSVisualEffectBlendingMode,
    radius: f32,
) -> Retained<Effect> {
    let effect = Effect::new(mtm);
    effect.setMaterial(self::material(material));
    effect.setBlendingMode(blending);
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

// The start and the end of the fade of a soft edge, in the unit square of a layer that is not flipped (y up).
pub(super) fn edge_points(edge: Edge) -> ((f64, f64), (f64, f64)) {
    match edge {
        Edge::Top => ((0.5, 1.0), (0.5, 0.0)),
        Edge::Bottom => ((0.5, 0.0), (0.5, 1.0)),
        Edge::Leading => ((0.0, 0.5), (1.0, 0.5)),
        _ => ((1.0, 0.5), (0.0, 0.5)),
    }
}

// The edge of a strip, `depth` deep, in the strip's own coordinates measured from the top left.
pub(super) fn edge_rect(
    edge: Edge,
    (x, y): (i32, i32),
    (width, height): (u32, u32),
    depth: u32,
) -> (i32, i32, i32, i32) {
    let (width, height, depth) = (width as i32, height as i32, depth as i32);
    match edge {
        Edge::Top => (x, y, width, depth),
        Edge::Bottom => (x, y + height - depth, width, depth),
        Edge::Leading => (x, y, depth, height),
        _ => (x + width - depth, y, depth, height),
    }
}

// The material stays whole over the first part of the depth and fades to the end's strength at the depth.
const FADE_STOPS: [f64; 3] = [0.0, 0.75, 1.0];

// How much of the material a soft edge shows where it is whole and at its end: the SwiftUI window over a red row shows it at half.
pub(super) const FADE_STRENGTH: (f64, f64) = (0.6, 0.0);

// SwiftUI's sidebar scroll pocket under a full-screen bar, read from Resonance Swift (ADR 0116), with Core Animation's private
// backdrop layers: a half-scale variable blur of the rows, and over it the background behind the rows, captured below them and shown
// through a gradient. Both run 10 pt past the pocket's bottom; the blur's mask and the gradient each ramp over 30 pt there.
const POCKET_RAMP: f64 = 30.0;
const POCKET_OVERRUN: f64 = 10.0;
const POCKET_MASK_ROWS: usize = 128;
const POCKET_VEIL_START: f64 = 0.5;
const POCKET_VEIL: f64 = 0.85;

pub(super) struct Pocket {
    // Under the sidebar's rows, it captures the background they lie on.
    pub(super) capture: Retained<NSView>,
    pub(super) view: Retained<NSView>,
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
            CGColor::new_generic_rgb(1.0, 0.0, 1.0, POCKET_VEIL),
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
            view: host(&[&blur, &veil])?,
        })
    }

    pub(super) fn remove(&self) {
        self.capture.removeFromSuperview();
        self.view.removeFromSuperview();
    }

    pub(super) fn set_frame(&self, frame: NSRect) {
        let (width, depth) = (frame.size.width, frame.size.height);
        let overrun = NSRect::new(
            NSPoint::new(0.0, -POCKET_OVERRUN),
            NSSize::new(width, depth + POCKET_OVERRUN),
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
            if let Some(gradient) = layers
                .lastObject()
                .as_deref()
                .and_then(first_sublayer)
                .and_then(|layer| layer.downcast::<CAGradientLayer>().ok())
            {
                let height = overrun.size.height;
                gradient.setFrame(NSRect::new(NSPoint::new(0.0, 0.0), overrun.size));
                gradient.setStartPoint(NSPoint::new(0.5, POCKET_VEIL_START / height));
                gradient.setEndPoint(NSPoint::new(
                    0.5,
                    (POCKET_VEIL_START + POCKET_RAMP) / height,
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
        let _: () = msg_send![&layer, setScale: 0.5f64];
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

// The same bytes as SwiftUI's mask for this depth: zero, then a 30 pt ramp ending at the pocket's bottom, then whole over the overrun.
fn pocket_mask(depth: f64) -> [u8; POCKET_MASK_ROWS] {
    let height = depth + POCKET_OVERRUN;
    std::array::from_fn(|row| {
        let y = row as f64 / (POCKET_MASK_ROWS - 1) as f64 * height;
        (((y - (depth - POCKET_RAMP)) / POCKET_RAMP).clamp(0.0, 1.0) * 255.0) as u8
    })
}

// The effect shows through a mask that is whole at the edge and keeps the tail's strength at the depth.
pub(super) fn fade(effect: &Effect, edge: Edge, (whole, tail): (f64, f64)) {
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
    let stops: Vec<_> = FADE_STOPS
        .iter()
        .map(|stop| NSNumber::new_f64(*stop))
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

pub(super) fn container(mtm: MainThreadMarker, frame: NSRect) -> Retained<NSView> {
    let container = NSView::initWithFrame(NSView::alloc(mtm), frame);
    container.setAutoresizesSubviews(true);
    container
}

// A glass in its strip: the rectangle, the radius, whether it is clear and its tint.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::backdrop::Requests;
    use crate::platform::backdrop::tests::{at, chrome, glass};

    #[test]
    fn a_rect_measured_from_the_top_is_placed_from_the_bottom() {
        let rect = frame((0, 0, 220, 480), 480.0);
        assert_eq!((rect.origin.y, rect.size.height), (0.0, 480.0));
        let rect = frame((10, 52, 100, 40), 480.0);
        assert_eq!(rect.origin.y, 388.0);
    }

    #[test]
    fn a_full_height_band_stretches_and_any_other_keeps_its_top() {
        assert_eq!(
            autoresizing((0, 0, 220, 480), 480.0),
            NSAutoresizingMaskOptions::ViewHeightSizable
        );
        assert_eq!(
            autoresizing((0, 52, 220, 100), 480.0),
            NSAutoresizingMaskOptions::ViewMinYMargin
        );
    }

    #[test]
    fn a_soft_edge_fades_from_its_side_to_its_depth() {
        assert_eq!(edge_points(Edge::Top), ((0.5, 1.0), (0.5, 0.0)));
        assert_eq!(edge_points(Edge::Leading), ((0.0, 0.5), (1.0, 0.5)));
        assert_eq!(
            edge_rect(Edge::Top, (10, 20), (300, 200), 48),
            (10, 20, 300, 48)
        );
        assert_eq!(
            edge_rect(Edge::Bottom, (10, 20), (300, 200), 48),
            (10, 172, 300, 48)
        );
        assert_eq!(
            edge_rect(Edge::Trailing, (10, 20), (300, 200), 48),
            (262, 20, 48, 200)
        );
    }

    #[test]
    fn every_material_has_its_own_native_one() {
        let all = [
            (Material::Titlebar, NSVisualEffectMaterial::Titlebar),
            (Material::Sidebar, NSVisualEffectMaterial::Sidebar),
            (Material::HudWindow, NSVisualEffectMaterial::HUDWindow),
            (Material::Sheet, NSVisualEffectMaterial::Sheet),
            (
                Material::ContentBackground,
                NSVisualEffectMaterial::ContentBackground,
            ),
        ];
        for (kit, native) in all {
            assert_eq!(material(kit), native);
        }
    }

    #[test]
    fn a_nested_glass_keeps_its_own_radius_and_variant_where_the_request_keeps_one() {
        let outer = at(10.0, 10.0, 100.0, 60.0);
        let chrome = chrome(
            vec![
                (glass(28, false, 0), outer),
                (glass(10, true, 0), at(20.0, 20.0, 40.0, 20.0)),
            ],
            outer,
        );
        assert_eq!(
            chrome.glasses(1.0),
            vec![
                ((0, 0, 100, 60), 28, false, None),
                ((10, 10, 40, 20), 10, true, None)
            ]
        );
        assert_eq!(Requests::of(&chrome, 1.0).backdrop.rects.len(), 1);
    }

    #[test]
    fn the_pocket_mask_is_swiftuis_mask_revealed_and_at_rest() {
        let revealed = pocket_mask(82.0);
        assert_eq!(revealed[71..=74], [0, 1, 7, 13]);
        assert_eq!(revealed[112..=114], [247, 253, 255]);
        let rest = pocket_mask(52.0);
        assert_eq!(rest[45..=48], [0, 3, 8, 12]);
        assert_eq!(rest[105..=107], [248, 252, 255]);
    }
}

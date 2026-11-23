use ctappearance::material::Material;
use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly, msg_send, sel};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSGlassEffectViewStyle, NSSplitViewController, NSSplitViewItem,
    NSView, NSViewController, NSVisualEffectBlendingMode, NSVisualEffectMaterial,
    NSVisualEffectState,
};
use objc2_core_graphics::CGColor;
use objc2_foundation::{NSArray, NSNumber, NSObjectProtocol};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use objc2_quartz_core::{CAGradientLayer, kCACornerCurveContinuous};

use super::view::{Effect, Glass, GlassGroup};
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

// The full-screen bar of the SwiftUI twin blurs a sidebar row under it whole at the top and still at 0.6 where the bar ends.
pub(super) const POCKET_STRENGTH: (f64, f64) = (1.0, 0.6);

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

pub(super) fn glass(mtm: MainThreadMarker, radius: f32, clear: bool) -> Retained<Glass> {
    let glass = Glass::new(mtm);
    // Setting `regular` turns the view into another glass than the default one AppKit's toolbar uses.
    if clear {
        glass.setStyle(NSGlassEffectViewStyle::Clear);
    }
    glass.setCornerRadius(f64::from(radius));
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

// A sidebar split item is how AppKit draws the sidebar of the running macOS: glass on 26, the behind-window material before.
pub(super) fn sidebar(mtm: MainThreadMarker) -> Retained<NSSplitViewController> {
    let split = NSSplitViewController::new(mtm);
    split.addSplitViewItem(&NSSplitViewItem::sidebarWithViewController(
        &empty_controller(mtm),
    ));
    let content = NSSplitViewItem::splitViewItemWithViewController(&empty_controller(mtm));
    content.setCollapsed(true);
    split.addSplitViewItem(&content);
    split
}

pub(super) fn place_sidebar(split: &NSSplitViewController, frame: NSRect) {
    if let Some(sidebar) = split.splitViewItems().firstObject() {
        sidebar.setMinimumThickness(frame.size.width);
        sidebar.setMaximumThickness(frame.size.width);
    }
    split.view().setFrame(frame);
}

pub(super) fn container(mtm: MainThreadMarker, frame: NSRect) -> Retained<NSView> {
    let container = NSView::initWithFrame(NSView::alloc(mtm), frame);
    container.setAutoresizesSubviews(true);
    container
}

// A glass in its strip: the rectangle, the radius and whether it is clear.
pub(crate) type PlacedGlass = ((i32, i32, i32, i32), u32, bool);

impl Chrome {
    // The protocol's backdrop holds one radius and one variant; macOS draws a native view per glass, so a nested one keeps its own.
    pub(crate) fn glasses(&self, scale: f32) -> Vec<PlacedGlass> {
        let origin = self.placement(scale).origin;
        self.effects
            .iter()
            .filter_map(|(effect, area)| match *effect {
                Declared::Glass { radius, clear, .. } => {
                    Some((strip_rect(area, origin, scale), radius, clear))
                }
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
            vec![((0, 0, 100, 60), 28, false), ((10, 10, 40, 20), 10, true)]
        );
        assert_eq!(Requests::of(&chrome, 1.0).backdrop.rects.len(), 1);
    }
}

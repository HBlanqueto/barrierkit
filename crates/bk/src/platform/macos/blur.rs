use ctappearance::material::Material;
use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSGlassEffectViewStyle, NSView, NSVisualEffectBlendingMode,
    NSVisualEffectMaterial, NSVisualEffectState,
};
use objc2_core_graphics::CGColor;
use objc2_foundation::{NSArray, NSNumber};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use objc2_quartz_core::{CAGradientLayer, kCACornerCurveContinuous};

use super::view::{Effect, Glass, GlassGroup};
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

// The material stays whole over the first part of the depth and fades to clear at the end.
const FADE_STOPS: [f64; 3] = [0.0, 0.75, 1.0];

// How much of the material a soft edge shows where it is whole: the SwiftUI window over a red row shows it at half.
const FADE_STRENGTH: f64 = 0.6;

// The effect shows through a mask that is opaque at the edge and clear at the depth.
pub(super) fn fade(effect: &Effect, edge: Edge) {
    let gray = |alpha| CGColor::new_generic_gray(0.0, alpha);
    let (start, end) = edge_points(edge);
    let layer = CAGradientLayer::new();
    layer.setFrame(effect.bounds());
    let (opaque, clear) = (gray(FADE_STRENGTH), gray(0.0));
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

pub(super) fn glass(mtm: MainThreadMarker, radius: f32) -> Retained<Glass> {
    let glass = Glass::new(mtm);
    glass.setStyle(NSGlassEffectViewStyle::Regular);
    glass.setCornerRadius(f64::from(radius));
    glass
}

// Glass views of one chrome merge when they are near, as in a SwiftUI `GlassEffectContainer`.
pub(super) fn glass_group(
    mtm: MainThreadMarker,
    frame: NSRect,
    members: &[Retained<Glass>],
) -> Retained<GlassGroup> {
    let group = GlassGroup::new(mtm);
    group.setFrame(frame);
    let holder = NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::ZERO, frame.size));
    for member in members {
        holder.addSubview(member);
    }
    group.setContentView(Some(&holder));
    group
}

pub(super) fn container(mtm: MainThreadMarker, frame: NSRect) -> Retained<NSView> {
    let container = NSView::initWithFrame(NSView::alloc(mtm), frame);
    container.setAutoresizesSubviews(true);
    container
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

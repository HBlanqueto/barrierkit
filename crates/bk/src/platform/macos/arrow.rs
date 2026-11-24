use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::Bool;
use objc2_app_kit::{NSGraphicsContext, NSImage};
use objc2_core_foundation::CFRetained;
use objc2_core_graphics::{CGContext, CGMutablePath};
use objc2_foundation::{NSRect, NSSize};

use super::place::Rect;
use crate::platform::window_style::PopoverShape;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

type Vertex = ((f64, f64), f64);

pub(super) struct Shape {
    radius: f64,
    band: f64,
    width: f64,
    height: f64,
    tip_radius: f64,
    fillet: f64,
}

pub(super) fn metrics(shape: PopoverShape) -> Shape {
    let popover = ctdesign::POPOVER;
    let band = f64::from(popover.arrow_height);
    let drawn = |radius: f32, width: f32, height: f32, tip_radius: f32, fillet: f32| Shape {
        radius: f64::from(radius),
        band,
        width: f64::from(width),
        height: f64::from(height),
        tip_radius: f64::from(tip_radius),
        fillet: f64::from(fillet),
    };
    match shape {
        PopoverShape::Label => drawn(
            ctdesign::DOCK_LABEL_RADIUS,
            ctdesign::DOCK_LABEL_ARROW_WIDTH,
            ctdesign::DOCK_LABEL_ARROW_HEIGHT,
            ctdesign::DOCK_LABEL_ARROW_TIP_RADIUS,
            ctdesign::DOCK_LABEL_ARROW_FILLET,
        ),
        _ => drawn(
            ctdesign::POPUP.corner_radius as f32,
            popover.arrow_width,
            popover.arrow_height,
            ctdesign::DOCK_MENU_ARROW_TIP_RADIUS,
            ctdesign::DOCK_MENU_ARROW_FILLET,
        ),
    }
}

pub(super) fn facing_edge(anchor: Rect, popup: Rect) -> Edge {
    if popup.y + popup.height <= anchor.y {
        Edge::Bottom
    } else if popup.x + popup.width <= anchor.x {
        Edge::Right
    } else if popup.x >= anchor.x + anchor.width {
        Edge::Left
    } else {
        Edge::Top
    }
}

fn base_reach(shape: &Shape) -> f64 {
    let half_base = shape.width / 2.0;
    let side = (-half_base / half_base.hypot(shape.height)).acos();
    let fillet_reach = shape.fillet / (side / 2.0).tan();
    shape.radius + half_base + fillet_reach
}

pub(super) fn tip(anchor: Rect, popup: Rect, edge: Edge, shape: &Shape) -> f64 {
    let (centre, length) = match edge {
        Edge::Top | Edge::Bottom => (
            f64::from(anchor.x) + f64::from(anchor.width) / 2.0 - f64::from(popup.x),
            f64::from(popup.width),
        ),
        Edge::Left | Edge::Right => (
            f64::from(anchor.y) + f64::from(anchor.height) / 2.0 - f64::from(popup.y),
            f64::from(popup.height),
        ),
    };
    let reach = base_reach(shape);
    if length < 2.0 * reach {
        length / 2.0
    } else {
        centre.clamp(reach, length - reach)
    }
}

pub(super) fn outline(
    (width, height): (f64, f64),
    edge: Edge,
    tip: f64,
    shape: &Shape,
) -> Vec<Vertex> {
    let (along, depth) = match edge {
        Edge::Top | Edge::Bottom => (width, height),
        Edge::Left | Edge::Right => (height, width),
    };
    let half_base = shape.width / 2.0;
    let apex = shape.band - shape.height;
    [
        ((0.0, shape.band), shape.radius),
        ((tip - half_base, shape.band), shape.fillet),
        ((tip, apex), shape.tip_radius),
        ((tip + half_base, shape.band), shape.fillet),
        ((along, shape.band), shape.radius),
        ((along, depth), shape.radius),
        ((0.0, depth), shape.radius),
    ]
    .map(|((u, v), radius)| {
        let at = match edge {
            Edge::Top => (u, v),
            Edge::Bottom => (u, height - v),
            Edge::Left => (v, u),
            Edge::Right => (width - v, u),
        };
        (at, radius)
    })
    .to_vec()
}

// AppKit's y runs up, the outline's runs down: a view whose layer is not flipped takes the height to turn it.
pub(super) fn path(outline: &[Vertex], height: Option<f64>) -> CFRetained<CGMutablePath> {
    let path = CGMutablePath::new();
    let flipped = |(x, y): (f64, f64)| (x, height.map_or(y, |height| height - y));
    let (last, first) = (flipped(outline[outline.len() - 1].0), flipped(outline[0].0));
    // SAFETY: the path is live, the transform is null and every number is finite.
    unsafe {
        CGMutablePath::move_to_point(
            Some(&path),
            std::ptr::null(),
            (last.0 + first.0) / 2.0,
            (last.1 + first.1) / 2.0,
        );
        for (index, (corner, radius)) in outline.iter().enumerate() {
            let (corner, next) = (
                flipped(*corner),
                flipped(outline[(index + 1) % outline.len()].0),
            );
            CGMutablePath::add_arc_to_point(
                Some(&path),
                std::ptr::null(),
                corner.0,
                corner.1,
                next.0,
                next.1,
                *radius,
            );
        }
    }
    CGMutablePath::close_subpath(Some(&path));
    path
}

// The alpha of the image is what the material and the window shadow show; the image draws again at each scale.
pub(super) fn mask_image(path: CFRetained<CGMutablePath>, size: NSSize) -> Retained<NSImage> {
    let draw = RcBlock::new(move |_: NSRect| {
        if let Some(context) = NSGraphicsContext::currentContext() {
            let context = context.CGContext();
            CGContext::set_rgb_fill_color(Some(&context), 0.0, 0.0, 0.0, 1.0);
            CGContext::add_path(Some(&context), Some(&path));
            CGContext::fill_path(Some(&context));
        }
        Bool::YES
    });
    NSImage::imageWithSize_flipped_drawingHandler(size, false, &draw)
}

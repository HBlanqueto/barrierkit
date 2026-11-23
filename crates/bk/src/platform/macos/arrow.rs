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

// The body starts `band` from the edge of the popup; the arrow is `width` wide there and its sharp apex is `height` above it.
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
        PopoverShape::Menu => drawn(
            ctdesign::POPUP.corner_radius as f32,
            popover.arrow_width,
            popover.arrow_height,
            ctdesign::DOCK_MENU_ARROW_TIP_RADIUS,
            ctdesign::DOCK_MENU_ARROW_FILLET,
        ),
        PopoverShape::Label => drawn(
            ctdesign::DOCK_LABEL_RADIUS,
            ctdesign::DOCK_LABEL_ARROW_WIDTH,
            ctdesign::DOCK_LABEL_ARROW_HEIGHT,
            ctdesign::DOCK_LABEL_ARROW_TIP_RADIUS,
            ctdesign::DOCK_LABEL_ARROW_FILLET,
        ),
        _ => drawn(
            popover.radius,
            ctdesign::POPOVER_APPKIT_ARROW_WIDTH,
            ctdesign::POPOVER_APPKIT_ARROW_APEX_HEIGHT,
            ctdesign::POPOVER_APPKIT_ARROW_TIP_RADIUS,
            ctdesign::POPOVER_APPKIT_ARROW_FILLET,
        ),
    }
}

// The edge of the popup that faces its anchor, as the compositor picks it.
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

// How far from a corner of the body the arrow keeps: the corner, half the base and the start of the fillet.
fn base_reach(shape: &Shape) -> f64 {
    let half_base = shape.width / 2.0;
    let side = (-half_base / half_base.hypot(shape.height)).acos();
    let fillet_reach = shape.fillet / (side / 2.0).tan();
    shape.radius + half_base + fillet_reach
}

// The arrow points at the centre of the anchor, kept clear of the corners.
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

// The corners of the outline, clockwise from the top left in a popup whose arrow band is at its top, each with its radius.
// The other edges turn that outline: `along` runs beside the edge and `depth` away from it.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: i32, y: i32, width: i32, height: i32) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn a_popup_below_its_anchor_has_the_arrow_on_its_top_edge() {
        assert_eq!(
            facing_edge(rect(0, 0, 40, 20), rect(0, 20, 200, 100)),
            Edge::Top
        );
    }

    #[test]
    fn a_popup_above_its_anchor_has_the_arrow_on_its_bottom_edge() {
        assert_eq!(
            facing_edge(rect(0, 200, 40, 20), rect(0, 90, 200, 100)),
            Edge::Bottom
        );
    }

    #[test]
    fn a_popup_beside_its_anchor_has_the_arrow_on_the_edge_that_faces_it() {
        let anchor = rect(300, 0, 40, 20);
        assert_eq!(facing_edge(anchor, rect(90, 0, 200, 100)), Edge::Right);
        assert_eq!(facing_edge(anchor, rect(350, 0, 200, 100)), Edge::Left);
    }

    #[test]
    fn the_arrow_points_at_the_anchor_centre_but_stays_clear_of_the_corners() {
        let shape = metrics(PopoverShape::Popover);
        let popup = rect(100, 40, 240, 160);
        let reach = base_reach(&shape);
        let at = |anchor_x| tip(rect(anchor_x, 0, 40, 20), popup, Edge::Top, &shape);
        assert_eq!(at(180), 100.0);
        assert_eq!(at(-500), reach);
        assert_eq!(at(900), 240.0 - reach);
    }

    #[test]
    fn a_body_narrower_than_the_arrow_needs_gets_the_arrow_in_the_middle() {
        let shape = metrics(PopoverShape::Popover);
        let popup = rect(0, 40, 60, 160);
        assert_eq!(tip(rect(500, 0, 40, 20), popup, Edge::Top, &shape), 30.0);
    }

    #[test]
    fn the_apex_is_its_height_above_the_body_on_whichever_edge_faces_the_anchor() {
        let shape = metrics(PopoverShape::Popover);
        let apex = shape.band - shape.height;
        let at = |edge, size| outline(size, edge, 100.0, &shape)[2].0;
        assert_eq!(at(Edge::Top, (240.0, 173.0)), (100.0, apex));
        assert_eq!(at(Edge::Bottom, (240.0, 173.0)), (100.0, 173.0 - apex));
        assert_eq!(at(Edge::Left, (173.0, 240.0)), (apex, 100.0));
        assert_eq!(at(Edge::Right, (173.0, 240.0)), (173.0 - apex, 100.0));
    }

    #[test]
    fn the_body_starts_one_band_from_the_edge_that_faces_the_anchor() {
        let shape = metrics(PopoverShape::Popover);
        let outline = outline((240.0, 173.0), Edge::Top, 100.0, &shape);
        assert_eq!(outline[0].0, (0.0, shape.band));
        assert_eq!(outline[5].0, (240.0, 173.0));
    }
}

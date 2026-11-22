use super::super::popup::{PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupSpec};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Rect {
    pub(super) x: i32,
    pub(super) y: i32,
    pub(super) width: i32,
    pub(super) height: i32,
}

impl Rect {
    fn right(self) -> i32 {
        self.x + self.width
    }

    fn bottom(self) -> i32 {
        self.y + self.height
    }
}

// (-1, 0, 1) along each axis: the side of the anchor rectangle, or the side of the popup that touches the anchor.
fn anchor_sides(anchor: PopupAnchor) -> (i32, i32) {
    match anchor {
        PopupAnchor::TopLeft => (-1, -1),
        PopupAnchor::Top => (0, -1),
        PopupAnchor::TopRight => (1, -1),
        PopupAnchor::Left => (-1, 0),
        PopupAnchor::Right => (1, 0),
        PopupAnchor::BottomLeft => (-1, 1),
        PopupAnchor::Bottom => (0, 1),
        PopupAnchor::BottomRight => (1, 1),
    }
}

fn gravity_sides(gravity: PopupGravity) -> (i32, i32) {
    match gravity {
        PopupGravity::TopLeft => (-1, -1),
        PopupGravity::Top => (0, -1),
        PopupGravity::TopRight => (1, -1),
        PopupGravity::Left => (-1, 0),
        PopupGravity::Right => (1, 0),
        PopupGravity::BottomLeft => (-1, 1),
        PopupGravity::Bottom => (0, 1),
        PopupGravity::BottomRight => (1, 1),
    }
}

// xdg_positioner: the anchor point plus the offset is where the popup touches; gravity says which way it extends.
fn along(start: i32, length: i32, anchor: i32, gravity: i32, offset: i32, size: i32) -> i32 {
    let point = start + (anchor + 1) * length / 2 + offset;
    point - (1 - gravity) * size / 2
}

fn at(spec: &PopupSpec, anchor: (i32, i32), gravity: (i32, i32)) -> Rect {
    let (x, y, width, height) = spec.anchor_rect;
    let (width, height) = (width as i32, height as i32);
    let size = (spec.size.0 as i32, spec.size.1 as i32);
    Rect {
        x: along(x, width, anchor.0, gravity.0, spec.offset.0, size.0),
        y: along(y, height, anchor.1, gravity.1, spec.offset.1, size.1),
        width: size.0,
        height: size.1,
    }
}

fn overflows(start: i32, end: i32, low: i32, high: i32) -> bool {
    start < low || end > high
}

// Per axis: flip, then slide, then (vertically) resize, as xdg_positioner orders the adjustments.
pub(super) fn place(spec: &PopupSpec, bounds: Rect) -> Rect {
    let PopupConstraintAdjustment {
        flip_x,
        flip_y,
        slide_x,
        slide_y,
        resize_y,
    } = spec.constraint_adjustment;
    let (anchor, gravity) = (anchor_sides(spec.anchor), gravity_sides(spec.gravity));
    let mut rect = at(spec, anchor, gravity);
    if flip_x && overflows(rect.x, rect.right(), bounds.x, bounds.right()) {
        let flipped = at(spec, (-anchor.0, anchor.1), (-gravity.0, gravity.1));
        if !overflows(flipped.x, flipped.right(), bounds.x, bounds.right()) {
            rect.x = flipped.x;
        }
    }
    if flip_y && overflows(rect.y, rect.bottom(), bounds.y, bounds.bottom()) {
        let flipped = at(spec, (anchor.0, -anchor.1), (gravity.0, -gravity.1));
        if !overflows(flipped.y, flipped.bottom(), bounds.y, bounds.bottom()) {
            rect.y = flipped.y;
        }
    }
    if slide_x && overflows(rect.x, rect.right(), bounds.x, bounds.right()) {
        rect.x = rect.x.min(bounds.right() - rect.width).max(bounds.x);
    }
    if slide_y && overflows(rect.y, rect.bottom(), bounds.y, bounds.bottom()) {
        rect.y = rect.y.min(bounds.bottom() - rect.height).max(bounds.y);
    }
    if resize_y && rect.bottom() > bounds.bottom() {
        rect.height = (bounds.bottom() - rect.y).max(1);
    }
    rect
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Rect = Rect {
        x: 0,
        y: 0,
        width: 1000,
        height: 800,
    };

    fn below(anchor_rect: (i32, i32, u32, u32), size: (u32, u32)) -> PopupSpec {
        PopupSpec {
            anchor_rect,
            anchor: PopupAnchor::BottomLeft,
            gravity: PopupGravity::BottomRight,
            size,
            ..PopupSpec::default()
        }
    }

    #[test]
    fn a_popup_below_its_anchor_opens_at_the_corner_of_the_anchor() {
        let rect = place(&below((100, 50, 80, 20), (200, 300)), SCREEN);
        assert_eq!(
            rect,
            Rect {
                x: 100,
                y: 70,
                width: 200,
                height: 300
            }
        );
    }

    #[test]
    fn a_popup_that_runs_off_the_bottom_flips_above_its_anchor_with_flip_y() {
        let mut spec = below((100, 700, 80, 20), (200, 300));
        spec.constraint_adjustment.flip_y = true;
        assert_eq!(place(&spec, SCREEN).y, 400);
    }

    #[test]
    fn a_popup_that_fits_on_neither_side_slides_inside_the_screen_with_slide_y() {
        let mut spec = below((100, 350, 80, 20), (200, 790));
        spec.constraint_adjustment.flip_y = true;
        spec.constraint_adjustment.slide_y = true;
        assert_eq!(place(&spec, SCREEN).y, 10);
    }

    #[test]
    fn a_popup_taller_than_the_room_below_shrinks_with_resize_y() {
        let mut spec = below((100, 700, 80, 20), (200, 300));
        spec.constraint_adjustment.resize_y = true;
        assert_eq!(place(&spec, SCREEN).height, 80);
    }

    #[test]
    fn a_submenu_that_runs_off_the_right_flips_to_the_left_of_its_anchor() {
        let mut spec = PopupSpec {
            anchor_rect: (900, 100, 80, 20),
            anchor: PopupAnchor::TopRight,
            gravity: PopupGravity::BottomRight,
            size: (150, 100),
            ..PopupSpec::default()
        };
        spec.constraint_adjustment.flip_x = true;
        assert_eq!(place(&spec, SCREEN).x, 750);
    }
}

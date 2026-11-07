use std::rc::Rc;

use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{Area, Gaps};
use vow_design::POPOVER;

use crate::platform::{PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupSpec, Popups};
use crate::theme::{SurfaceKind, color, popover_palette, resolved};

pub fn show_popover(anchor: Area, size: (f32, f32), content: impl Fn() -> Element + 'static) {
    let spec = PopupSpec {
        anchor_rect: (
            anchor.origin.x.round() as i32,
            anchor.origin.y.round() as i32,
            anchor.size.width.round().max(1.0) as u32,
            anchor.size.height.round().max(1.0) as u32,
        ),
        anchor: PopupAnchor::Bottom,
        gravity: PopupGravity::Bottom,
        size: (
            (size.0 + 2.0 * POPOVER.padding).ceil() as u32,
            (size.1 + 2.0 * POPOVER.padding + POPOVER.arrow_height).ceil() as u32,
        ),
        constraint_adjustment: PopupConstraintAdjustment {
            slide_x: true,
            ..PopupConstraintAdjustment::default()
        },
        grab: true,
        ..PopupSpec::default()
    };
    Popups::get().open_popover(spec, AppComponent::new(PopoverBody(Rc::new(content))));
}

struct PopoverBody(Rc<dyn Fn() -> Element>);

impl App for PopoverBody {
    fn render(&self) -> impl IntoElement {
        let menu = popover_palette(&resolved(), SurfaceKind::Toplevel);
        rect()
            .expanded()
            .background(color(menu.bg))
            .padding(Gaps::new(
                POPOVER.arrow_height + POPOVER.padding,
                POPOVER.padding,
                POPOVER.padding,
                POPOVER.padding,
            ))
            .child((self.0)())
    }
}

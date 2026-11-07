use std::rc::Rc;

use ctdesign::POPOVER;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{Area, Gaps};

use crate::platform::{PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupSpec, Popups};
use crate::theme::color;

pub struct Popover {
    size: (f32, f32),
    content: Rc<dyn Fn() -> Element>,
}

impl Popover {
    pub fn new(size: (f32, f32), content: impl Fn() -> Element + 'static) -> Self {
        Self {
            size,
            content: Rc::new(content),
        }
    }

    pub fn show(self, anchor: Area) {
        let size = self.size;
        let spec = PopupSpec {
            anchor_rect: crate::platform::anchor_rect(anchor),
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
        Popups::get().open_popover(spec, AppComponent::new(PopoverBody(self.content)));
    }
}

struct PopoverBody(Rc<dyn Fn() -> Element>);

impl App for PopoverBody {
    fn render(&self) -> impl IntoElement {
        let menu = crate::styles::popover_style();
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

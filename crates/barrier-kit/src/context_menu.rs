//! Menu opened with the secondary button at the pointer.

use freya_core::prelude::*;

use crate::menu::MenuItem;
use crate::popup_button::open_popover_menu;

pub fn context_menu(
    element: Rect,
    items: Vec<MenuItem>,
    on_activate: impl Into<Callback<String, ()>>,
) -> Rect {
    let on_activate = on_activate.into();
    element.on_secondary_down(move |event: Event<PressEventData>| {
        let PressEventData::Mouse(mouse) = event.data() else {
            return;
        };
        let on_activate = on_activate.clone();
        open_popover_menu(
            items.clone(),
            (
                mouse.global_location.x.round() as i32,
                mouse.global_location.y.round() as i32,
                1,
                1,
            ),
            (0, 0),
            move |id| on_activate.call(id),
        );
    })
}

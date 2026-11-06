use freya_core::integration::{SingleThreadErasedEvent, UserEvent};
use freya_core::prelude::*;

use crate::i18n::text;

use crate::menu::MenuItem;
use crate::platform::{EditShortcut, PopupAnchor};
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
            PopupAnchor::TopLeft,
            (
                mouse.global_location.x.round() as i32,
                mouse.global_location.y.round() as i32,
                1,
                1,
            ),
            None,
            move |id| on_activate.call(id),
        );
    })
}

pub(crate) fn edit_menu(element: Rect, field: AccessibilityId) -> Rect {
    let platform = Platform::get();
    let items = vec![
        MenuItem::action("x", text!("cut")),
        MenuItem::action("c", text!("copy")),
        MenuItem::action("v", text!("paste")),
        MenuItem::Separator,
        MenuItem::action("a", text!("select-all")),
    ];
    context_menu(element, items, move |key: String| {
        platform.send(UserEvent::Erased(SingleThreadErasedEvent(Box::new(
            EditShortcut { field, key },
        ))));
    })
}

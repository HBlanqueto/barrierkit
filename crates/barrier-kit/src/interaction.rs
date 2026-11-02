//! Hover, press and keyboard focus shared by every interactive control.

use freya_core::prelude::*;
use torin::prelude::{Position, Size, SizeFnContext};
use vow_design::FOCUS_RING;

#[derive(Clone, Copy)]
pub struct Interaction {
    a11y_id: AccessibilityId,
    hovered: State<bool>,
    pressed: State<bool>,
    focus: Memo<Focus>,
}

pub fn use_interaction() -> Interaction {
    let a11y_id = use_a11y();
    Interaction {
        a11y_id,
        hovered: use_state(|| false),
        pressed: use_state(|| false),
        focus: use_focus(a11y_id),
    }
}

impl Interaction {
    pub fn hovered(&self) -> bool {
        *self.hovered.read()
    }

    pub fn pressed(&self) -> bool {
        *self.pressed.read()
    }

    pub fn keyboard_focused(&self) -> bool {
        matches!(*self.focus.read(), Focus::Keyboard)
    }

    pub fn attach(&self, element: Rect, enabled: bool) -> Rect {
        let mut hovered = self.hovered;
        let mut pressed = self.pressed;
        let a11y_id = self.a11y_id;
        element
            .a11y_id(self.a11y_id)
            .a11y_focusable(enabled)
            .on_pointer_enter(move |_| hovered.set(true))
            .on_pointer_leave(move |_| {
                hovered.set(false);
                pressed.set_if_modified(false);
            })
            .maybe(enabled, |el| {
                el.on_pointer_down(move |_| {
                    pressed.set(true);
                    a11y_id.request_focus();
                })
            })
            .on_global_pointer_press(move |_| pressed.set_if_modified(false))
    }

    pub fn focus_ring(&self, element: Rect, corner_radius: f32) -> Rect {
        if !self.keyboard_focused() {
            return element;
        }
        ring(element, corner_radius, crate::theme::resolved().accent)
    }
}

pub fn ring(element: Rect, corner_radius: f32, color: vow_design::Rgba) -> Rect {
    let spread = FOCUS_RING.offset + FOCUS_RING.width;
    let around = move |context: SizeFnContext| Some(context.parent + 2.0 * spread);
    element.child(
        rect()
            .position(Position::new_absolute().left(-spread).top(-spread))
            .width(Size::func(around))
            .height(Size::func(around))
            .corner_radius(corner_radius + spread)
            .border(
                Border::new()
                    .width(FOCUS_RING.width)
                    .alignment(BorderAlignment::Inner)
                    .fill(crate::theme::color(color)),
            ),
    )
}

use ctdesign::FOCUS_RING;
use freya_core::prelude::*;
use torin::prelude::{Position, Size, SizeFnContext};

use crate::platform::WindowControls;
use crate::styles::ControlState;

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
        self.held() && self.hovered()
    }

    pub fn held(&self) -> bool {
        *self.pressed.read()
    }

    pub fn focused(&self) -> bool {
        !matches!(*self.focus.read(), Focus::Not)
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
            .maybe(!enabled, |el| el.a11y_builder(|node| node.set_disabled()))
            .on_pointer_enter(move |_| hovered.set(true))
            .on_pointer_leave(move |_| hovered.set(false))
            .maybe(enabled, |el| {
                el.on_pointer_down(move |_| {
                    pressed.set(true);
                    a11y_id.request_focus();
                })
            })
            .on_global_pointer_press(move |_| pressed.set_if_modified(false))
    }

    // An element that sets its own on_global_pointer_press replaces attach's; it calls this instead.
    pub(crate) fn release(&self) {
        let mut pressed = self.pressed;
        pressed.set_if_modified(false);
    }

    pub fn request_focus(&self) {
        self.a11y_id.request_focus();
    }

    fn shows_focus(&self) -> bool {
        self.keyboard_focused() && crate::platform::window_activated()
    }

    pub fn focus_ring(&self, element: Rect, corner_radius: f32) -> Rect {
        if !self.shows_focus() {
            return element;
        }
        ring(element, corner_radius, crate::theme::resolved().focus_ring)
    }

    pub fn inner_focus_ring(&self, element: Rect) -> Rect {
        element.maybe(self.shows_focus(), |el| {
            el.border(
                Border::new()
                    .width(FOCUS_RING.width)
                    .alignment(BorderAlignment::Inner)
                    .fill(crate::theme::color(crate::theme::resolved().focus_ring)),
            )
        })
    }
}

pub(crate) fn control_state(interaction: Interaction, disabled: bool) -> ControlState {
    ControlState {
        hovered: interaction.hovered(),
        pressed: interaction.pressed(),
        focused: interaction.focused(),
        disabled,
    }
}

pub(crate) fn ring<E: ChildrenExt>(element: E, corner_radius: f32, color: ctdesign::Rgba) -> E {
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

pub(crate) fn use_held_modifiers() -> State<Modifiers> {
    let mut held = use_state(Modifiers::empty);
    let status = try_consume_root_context::<WindowControls>().map(|controls| controls.status);
    use_side_effect(move || {
        if status.is_some_and(|status| !status.read().activated) {
            held.set_if_modified(Modifiers::empty());
        }
    });
    held
}

fn modifier_of(key: &Key) -> Modifiers {
    match key {
        Key::Named(NamedKey::Control) => Modifiers::CONTROL,
        Key::Named(NamedKey::Shift) => Modifiers::SHIFT,
        Key::Named(NamedKey::Meta) => Modifiers::META,
        Key::Named(NamedKey::Alt) => Modifiers::ALT,
        _ => Modifiers::empty(),
    }
}

pub(crate) fn track_modifiers(element: Rect, mut held: State<Modifiers>) -> Rect {
    element
        .on_global_key_down(move |event: Event<KeyboardEventData>| {
            held.set_if_modified(event.modifiers | modifier_of(&event.key));
        })
        .on_global_key_up(move |event: Event<KeyboardEventData>| {
            held.set_if_modified(event.modifiers - modifier_of(&event.key));
        })
}

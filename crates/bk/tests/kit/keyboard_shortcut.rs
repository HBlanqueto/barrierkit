use std::cell::Cell;
use std::rc::Rc;

use bk::app::Shortcut;
use bk::controls::{Button, ButtonRole};
use bk::elements::VStack;
use bk::modifiers::EnvironmentModifiers;
use freya_core::integration::{AppComponent, KeyboardEventName, PlatformEvent};
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::Size2D;

struct Case {
    shortcut: Shortcut,
    own: Option<Shortcut>,
    disabled: bool,
    presses: Rc<Cell<u32>>,
}

impl App for Case {
    fn render(&self) -> impl IntoElement {
        let presses = self.presses.clone();
        let button = Button::new("Delete")
            .role(ButtonRole::Destructive)
            .on_press(move |_| presses.set(presses.get() + 1));
        let button = match self.own {
            Some(own) => button.keyboard_shortcut(own),
            None => button,
        };
        VStack::new()
            .child(button)
            .disabled(self.disabled)
            .keyboard_shortcut(self.shortcut)
    }
}

fn presses(case: Case, keys: &[Key]) -> u32 {
    let count = case.presses.clone();
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(case),
        Size2D::new(160.0, 60.0),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    for key in keys {
        test.send_event(PlatformEvent::Keyboard {
            name: KeyboardEventName::KeyDown,
            key: key.clone(),
            code: Code::Unidentified,
            modifiers: Modifiers::empty(),
        });
        test.sync_and_update();
    }
    count.get()
}

fn case(shortcut: Shortcut, own: Option<Shortcut>, disabled: bool) -> Case {
    Case {
        shortcut,
        own,
        disabled,
        presses: Rc::default(),
    }
}

#[test]
fn a_shortcut_on_a_view_presses_the_buttons_inside_it() {
    let default = Key::Named(NamedKey::Enter);
    let cancel = Key::Named(NamedKey::Escape);
    let keys = [default.clone(), cancel, default];
    assert_eq!(
        presses(case(Shortcut::DEFAULT_ACTION, None, false), &keys),
        2
    );
}

#[test]
fn a_shortcut_on_a_view_is_ignored_while_the_view_is_disabled() {
    let keys = [Key::Named(NamedKey::Escape)];
    assert_eq!(presses(case(Shortcut::CANCEL_ACTION, None, true), &keys), 0);
}

#[test]
fn the_shortcut_of_a_button_wins_over_the_one_of_its_view() {
    let keys = [Key::Named(NamedKey::Enter), Key::Named(NamedKey::Escape)];
    let case = case(
        Shortcut::DEFAULT_ACTION,
        Some(Shortcut::CANCEL_ACTION),
        false,
    );
    assert_eq!(presses(case, &keys), 1);
}

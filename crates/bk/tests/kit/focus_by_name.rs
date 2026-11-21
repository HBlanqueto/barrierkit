use std::cell::Cell;
use std::rc::Rc;

use bk::app::{Shortcut, ShortcutKey};
use bk::controls::{Button, TextField};
use bk::elements::VStack;
use bk::focus::{FocusState, use_focus_state};
use bk::modifiers::EnvironmentModifiers;
use freya_core::integration::{AppComponent, KeyboardEventName, PlatformEvent};
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::Size2D;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Field {
    Name,
    Email,
}

struct FocusCase {
    default: Option<Field>,
    email_shown: State<bool>,
    handle: Handle,
    seen: State<Option<Field>>,
}

impl App for FocusCase {
    fn render(&self) -> impl IntoElement {
        let focus = use_focus_state::<Field>();
        let name = use_state(String::new);
        let email = use_state(String::new);
        let (mut handle, mut seen) = (self.handle, self.seen);
        handle.set_if_modified(Some(focus));
        seen.set_if_modified(focus.get());
        let form = VStack::new()
            .child(TextField::new(name, "Name").focused(focus, Field::Name))
            .maybe_child(
                (*self.email_shown.read())
                    .then(|| TextField::new(email, "Email").focused(focus, Field::Email)),
            );
        match self.default {
            Some(field) => form.default_focus(focus, field).into_element(),
            None => form.into_element(),
        }
    }
}

type Handle = State<Option<FocusState<Field>>>;
type Run = (TestingRunner, Handle, State<Option<Field>>, State<bool>);

fn focus_runner(default: Option<Field>) -> Run {
    let handle = State::create_global(None);
    let seen = State::create_global(None);
    let email_shown = State::create_global(true);
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(FocusCase {
            default,
            email_shown,
            handle,
            seen,
        }),
        Size2D::new(240.0, 120.0),
        |_| {},
        1.0,
    );
    for _ in 0..4 {
        test.sync_and_update();
    }
    (test, handle, seen, email_shown)
}

fn settle(test: &mut TestingRunner) {
    for _ in 0..4 {
        test.sync_and_update();
    }
}

#[test]
fn nothing_is_focused_until_something_asks() {
    let (_, _, seen, _) = focus_runner(None);
    assert_eq!(*seen.peek(), None);
}

#[test]
fn setting_the_focus_state_moves_the_focus_to_the_named_field_and_clearing_it_leaves() {
    let (mut test, handle, seen, _) = focus_runner(None);
    let focus = (*handle.peek()).expect("the host rendered");
    test.run_in(|| focus.set(Some(Field::Email)));
    settle(&mut test);
    assert_eq!(*seen.peek(), Some(Field::Email));
    test.run_in(|| focus.set(Some(Field::Name)));
    settle(&mut test);
    assert_eq!(*seen.peek(), Some(Field::Name));
    test.run_in(|| focus.set(None));
    settle(&mut test);
    assert_eq!(*seen.peek(), None);
}

#[test]
fn the_focus_state_follows_the_focus_when_the_user_moves_it() {
    let (mut test, handle, seen, _) = focus_runner(None);
    let focus = (*handle.peek()).expect("the host rendered");
    test.run_in(|| focus.set(Some(Field::Name)));
    settle(&mut test);
    test.press_key(Key::Named(NamedKey::Tab));
    settle(&mut test);
    assert_eq!(*seen.peek(), Some(Field::Email));
}

#[test]
fn the_default_focus_goes_to_the_named_field_once_the_view_has_appeared() {
    let (_, _, seen, _) = focus_runner(Some(Field::Email));
    assert_eq!(*seen.peek(), Some(Field::Email));
}

struct ShortcutCase {
    shortcut: Shortcut,
    disabled: bool,
    presses: Rc<Cell<u32>>,
}

impl App for ShortcutCase {
    fn render(&self) -> impl IntoElement {
        let presses = self.presses.clone();
        Button::new("Save")
            .keyboard_shortcut(self.shortcut)
            .disabled(self.disabled)
            .on_press(move |_| presses.set(presses.get() + 1))
    }
}

fn key_down(test: &mut TestingRunner, key: &str, modifiers: Modifiers) {
    test.send_event(PlatformEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key: Key::Character(key.into()),
        code: Code::Unidentified,
        modifiers,
    });
    test.sync_and_update();
}

fn shortcut_presses(disabled: bool, keys: &[(&str, Modifiers)]) -> u32 {
    let presses = Rc::new(Cell::new(0));
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(ShortcutCase {
            shortcut: Shortcut::ctrl(ShortcutKey::Char('k')),
            disabled,
            presses: presses.clone(),
        }),
        Size2D::new(120.0, 40.0),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    for (key, modifiers) in keys {
        key_down(&mut test, key, *modifiers);
    }
    presses.get()
}

#[test]
fn a_button_presses_on_its_shortcut_and_only_on_it() {
    let keys = [
        ("k", Modifiers::CONTROL),
        ("k", Modifiers::empty()),
        ("k", Modifiers::CONTROL | Modifiers::SHIFT),
        ("j", Modifiers::CONTROL),
        ("K", Modifiers::CONTROL),
    ];
    assert_eq!(shortcut_presses(false, &keys), 2);
}

#[test]
fn a_disabled_button_ignores_its_shortcut() {
    assert_eq!(shortcut_presses(true, &[("k", Modifiers::CONTROL)]), 0);
}

#[test]
fn a_field_that_leaves_the_view_is_no_longer_a_target() {
    let (mut test, handle, seen, mut email_shown) = focus_runner(None);
    let focus = (*handle.peek()).expect("the host rendered");
    email_shown.set(false);
    settle(&mut test);
    test.run_in(|| focus.set(Some(Field::Email)));
    settle(&mut test);
    assert_eq!(*seen.peek(), None);
    test.run_in(|| focus.set(Some(Field::Name)));
    settle(&mut test);
    assert_eq!(*seen.peek(), Some(Field::Name));
}

use std::cell::Cell;
use std::rc::Rc;

use bk::controls::{Button, Checkbox, Switch};
use bk::modifiers::ViewModifiers;
use ctappearance::{AppearanceConfig, ColorScheme, resolve};
use ctdesign::SANS_FAMILIES;
use freya_core::integration::{App, AppComponent};
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Direction, Gaps, Size, Size2D};

struct Case(fn() -> Element);

impl App for Case {
    fn render(&self) -> impl IntoElement {
        rect()
            .width(Size::px(400.0))
            .height(Size::px(200.0))
            .direction(Direction::Vertical)
            .child((self.0)())
    }
}

fn runner(build: fn() -> Element) -> TestingRunner {
    let resolved = resolve(
        &AppearanceConfig {
            color_scheme: ColorScheme::Light,
            reduce_transparency: true,
            ..Default::default()
        },
        None,
    );
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Case(build)),
        Size2D::new(400.0, 200.0),
        move |runner| runner.provide_root_context(move || State::create(resolved)),
        1.0,
    );
    test.set_default_fonts(&SANS_FAMILIES.map(std::borrow::Cow::Borrowed));
    test.sync_and_update();
    test.sync_and_update();
    test
}

fn size_of(build: fn() -> Element) -> (f32, f32) {
    let test = runner(build);
    test.find(|node, element| {
        (element.accessibility().builder.label() == Some("Save")).then(|| {
            let area = node.layout().area;
            (area.width(), area.height())
        })
    })
    .expect("a node labelled Save")
}

fn plain() -> (f32, f32) {
    size_of(|| Button::new("Save").into_element())
}

#[test]
fn padding_height_and_margin_set_properties_on_the_buttons_own_node() {
    let (width, height) = plain();
    let padded = size_of(|| {
        Button::new("Save")
            .padding(Gaps::new(0.0, 20.0, 0.0, 20.0))
            .into_element()
    });
    assert!(padded.0 > width && padded.1 == height);
    let tall = size_of(|| Button::new("Save").height(Size::px(50.0)).into_element());
    assert_eq!(tall, (width, 50.0));
    let margin = size_of(|| Button::new("Save").margin(8.0).into_element());
    assert_eq!(margin, (width + 16.0, height + 16.0));
}

#[test]
fn the_order_of_modifier_calls_does_not_change_the_layout() {
    let before = size_of(|| {
        Button::new("Save")
            .padding(Gaps::new(0.0, 20.0, 0.0, 20.0))
            .on_press(|_| {})
            .into_element()
    });
    let after = size_of(|| {
        Button::new("Save")
            .on_press(|_| {})
            .padding(Gaps::new(0.0, 20.0, 0.0, 20.0))
            .into_element()
    });
    assert_eq!(before, after);
}

#[test]
fn event_handlers_on_a_control_keep_its_size_even_when_it_fills() {
    let handled = size_of(|| Button::new("Save").on_mouse_move(|_| {}).into_element());
    assert_eq!(handled, plain());
    let filled = size_of(|| {
        Button::new("Save")
            .width(Size::fill())
            .on_mouse_move(|_| {})
            .into_element()
    });
    assert_eq!(filled.0, 400.0);
}

#[test]
fn help_and_handlers_reach_the_control_without_replacing_its_own_press() {
    thread_local! { static COUNTS: (Rc<Cell<u32>>, Rc<Cell<u32>>) = Default::default(); }
    let (moved, pressed) = COUNTS.with(|counts| counts.clone());
    let mut test = runner(|| {
        let (moved, pressed) = COUNTS.with(|counts| counts.clone());
        Button::new("Save")
            .on_press(move |_| pressed.set(pressed.get() + 1))
            .on_mouse_move(move |_| moved.set(moved.get() + 1))
            .help("Save the file")
            .into_element()
    });
    test.move_cursor((20.0, 10.0));
    test.sync_and_update();
    test.click_cursor((20.0, 10.0));
    test.sync_and_update();
    assert_eq!((moved.get() > 0, pressed.get()), (true, 1));
}

#[test]
fn other_controls_take_the_same_modifiers() {
    let switch = size_of(|| Switch::new("Save").into_element());
    let margin = size_of(|| Switch::new("Save").margin(8.0).into_element());
    assert_eq!(margin, (switch.0 + 16.0, switch.1 + 16.0));
    let checkbox = size_of(|| Checkbox::new("Save").into_element());
    let margin = size_of(|| Checkbox::new("Save").margin(8.0).into_element());
    assert_eq!(margin, (checkbox.0 + 16.0, checkbox.1 + 16.0));
}

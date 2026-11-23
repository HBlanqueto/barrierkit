use std::cell::{Cell, RefCell};
use std::rc::Rc;

use bk::adornments::Link;
use bk::buttons::{ComboButton, HelpLink, MenuButton};
use bk::choices::RadioGroup;
use bk::color_picker::ColorPicker;
use bk::combo_box::ComboBox;
use bk::compact::{Face, IconButton, Stepper, ToggleGroup};
use bk::controls::{
    Button, Checkbox, CircularSlider, SearchField, SegmentedControl, Slider, Switch, TextField,
};
use bk::date::{CivilDate, CivilDateTime};
use bk::date_picker::DatePicker;
use bk::elements::VStack;
use bk::environment::{use_environment, use_is_enabled};
use bk::level::LevelSlider;
use bk::menu::MenuItem;
use bk::modifiers::EnvironmentModifiers;
use bk::popup_button::PopUpButton;
use bk::symbol::Symbol;
use ctdesign::Rgba;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Gaps, Size2D};

use crate::support::{differing, runner, scheme};

const SIZE: (f32, f32) = (300.0, 120.0);

fn pixels(dark: bool, build: impl Fn() -> Element + 'static) -> Vec<u8> {
    let mut test = runner(SIZE, scheme(dark), move || {
        rect()
            .expanded()
            .padding(Gaps::new(24.0, 24.0, 24.0, 24.0))
            .child(build())
            .into_element()
    });
    crate::image::decode(test.render().as_bytes()).2
}

fn date_time() -> CivilDateTime {
    CivilDateTime {
        date: CivilDate {
            year: 2026,
            month: 11,
            day: 21,
        },
        hour: 9,
        minute: 41,
    }
}

type Row = (&'static str, fn(bool) -> Element);
type Plain = (&'static str, fn() -> Element);

fn controls_with_a_disabled_flag() -> Vec<Row> {
    vec![
        ("button", |disabled| {
            Button::new("Save").disabled(disabled).into_element()
        }),
        ("switch", |disabled| {
            Switch::new("Wi-Fi")
                .on(true)
                .disabled(disabled)
                .into_element()
        }),
        ("checkbox", |disabled| {
            Checkbox::new("Remember")
                .checked(true)
                .disabled(disabled)
                .into_element()
        }),
        ("slider", |disabled| {
            Slider::new("Level")
                .value(0.5)
                .disabled(disabled)
                .into_element()
        }),
        ("circular-slider", |disabled| {
            CircularSlider::new("Dial")
                .value(0.3)
                .disabled(disabled)
                .into_element()
        }),
        ("segmented-control", |disabled| {
            SegmentedControl::new("View")
                .segment("One")
                .segment("Two")
                .disabled(disabled)
                .into_element()
        }),
        ("radio-group", |disabled| {
            RadioGroup::new("Radio")
                .option("One")
                .option("Two")
                .disabled(disabled)
                .into_element()
        }),
        ("stepper", |disabled| {
            Stepper::new("Cores", 1, 16)
                .value(4)
                .disabled(disabled)
                .into_element()
        }),
        ("icon-button", |disabled| {
            IconButton::new(Symbol::Plus, "Add")
                .disabled(disabled)
                .into_element()
        }),
        ("text-field", |disabled| {
            TextField::new(use_state(|| "ana".to_string()), "Name")
                .disabled(disabled)
                .into_element()
        }),
        ("search-field", |disabled| {
            SearchField::new(use_state(|| "wifi".to_string()), "Search")
                .disabled(disabled)
                .into_element()
        }),
        ("pop-up-button", |disabled| {
            PopUpButton::new("Option")
                .item("One")
                .item("Two")
                .disabled(disabled)
                .into_element()
        }),
        ("date-picker", |disabled| {
            DatePicker::new("Date", date_time())
                .disabled(disabled)
                .into_element()
        }),
        ("color-picker", |disabled| {
            ColorPicker::new("Color", Rgba::rgb8(0, 136, 255))
                .disabled(disabled)
                .into_element()
        }),
        ("combo-box", |disabled| {
            ComboBox::new(use_state(|| "One".to_string()), "Combo")
                .item("One")
                .disabled(disabled)
                .into_element()
        }),
        ("level-slider", |disabled| {
            LevelSlider::new("Level")
                .value(0.4)
                .disabled(disabled)
                .into_element()
        }),
    ]
}

#[test]
fn a_disabled_container_draws_its_controls_as_their_own_flag_does() {
    for (name, build) in controls_with_a_disabled_flag() {
        for dark in [false, true] {
            let own = pixels(dark, move || {
                VStack::new().child(build(true)).into_element()
            });
            let scoped = pixels(dark, move || {
                VStack::new()
                    .child(build(false))
                    .disabled(true)
                    .into_element()
            });
            let enabled = pixels(dark, move || {
                VStack::new().child(build(false)).into_element()
            });
            assert_eq!(
                differing(&own, &scoped),
                0,
                "{name} (dark {dark}) in a disabled container differs from its own disabled flag"
            );
            assert!(
                dark || differing(&enabled, &own) > 0,
                "{name} looks the same disabled and enabled"
            );
        }
    }
}

#[test]
fn an_enabled_container_leaves_every_control_as_it_is() {
    for (name, build) in controls_with_a_disabled_flag() {
        let plain = pixels(false, move || {
            VStack::new().child(build(false)).into_element()
        });
        let scoped = pixels(false, move || {
            VStack::new()
                .child(build(false))
                .disabled(false)
                .into_element()
        });
        assert_eq!(differing(&plain, &scoped), 0, "{name}");
    }
}

fn interactive_nodes_are_disabled(test: &TestingRunner) -> Vec<bool> {
    test.find_many(|_, element| {
        let node = &element.accessibility().builder;
        matches!(
            node.role(),
            AccessibilityRole::Button | AccessibilityRole::Link
        )
        .then(|| node.is_disabled())
    })
}

#[test]
fn the_controls_without_a_flag_announce_a_disabled_container() {
    let builds: Vec<Plain> = vec![
        ("menu-button", || {
            MenuButton::new("Menu", vec![MenuItem::action("one", "One")]).into_element()
        }),
        ("combo-button", || {
            ComboButton::new("Combo", vec![MenuItem::action("one", "One")]).into_element()
        }),
        ("help-link", || HelpLink::new().into_element()),
        ("link", || Link::new("Link").into_element()),
        ("toggle-group", || {
            ToggleGroup::new("Format")
                .item("B", Face::Bold, "Bold", false)
                .into_element()
        }),
    ];
    for (name, build) in builds {
        for disabled in [false, true] {
            let test = runner(SIZE, scheme(false), move || {
                VStack::new()
                    .child(build())
                    .disabled(disabled)
                    .into_element()
            });
            let nodes = interactive_nodes_are_disabled(&test);
            assert!(!nodes.is_empty(), "{name} has no button or link node");
            assert!(
                nodes.iter().all(|node| *node == disabled),
                "{name} in a container with disabled {disabled}: {nodes:?}"
            );
        }
    }
}

struct PressCase {
    disabled: State<bool>,
    inner_enabled: bool,
    presses: Rc<Cell<u32>>,
}

impl App for PressCase {
    fn render(&self) -> impl IntoElement {
        let presses = self.presses.clone();
        let button = Button::new("Save").on_press(move |_| presses.set(presses.get() + 1));
        let inner = if self.inner_enabled {
            VStack::new().child(button).disabled(false).into_element()
        } else {
            VStack::new().child(button).into_element()
        };
        VStack::new().child(inner).disabled(*self.disabled.read())
    }
}

fn press_runner(
    disabled: bool,
    inner_enabled: bool,
) -> (TestingRunner, State<bool>, Rc<Cell<u32>>) {
    let disabled = State::create_global(disabled);
    let presses = Rc::new(Cell::new(0));
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(PressCase {
            disabled,
            inner_enabled,
            presses: presses.clone(),
        }),
        Size2D::new(160.0, 60.0),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    (test, disabled, presses)
}

fn settle(test: &mut TestingRunner) {
    for _ in 0..4 {
        test.sync_and_update();
    }
}

fn click(test: &mut TestingRunner) {
    test.click_cursor((20.0, 10.0));
    settle(test);
}

#[test]
fn a_button_presses_until_a_container_above_it_is_disabled_and_again_when_it_is_not() {
    let (mut test, mut disabled, presses) = press_runner(false, false);
    click(&mut test);
    assert_eq!(presses.get(), 1);
    disabled.set(true);
    settle(&mut test);
    click(&mut test);
    assert_eq!(presses.get(), 1);
    disabled.set(false);
    settle(&mut test);
    click(&mut test);
    assert_eq!(presses.get(), 2);
}

#[test]
fn a_view_that_is_not_disabled_does_not_enable_what_an_outer_one_disabled() {
    let (mut test, _, presses) = press_runner(true, true);
    click(&mut test);
    assert_eq!(presses.get(), 0);
}

struct Probe {
    mounts: Rc<Cell<u32>>,
}

impl PartialEq for Probe {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Component for Probe {
    fn render(&self) -> impl IntoElement {
        let mounts = self.mounts.clone();
        use_hook(move || mounts.set(mounts.get() + 1));
        rect()
    }
}

struct MountCase {
    disabled: State<bool>,
    mounts: Rc<Cell<u32>>,
}

impl App for MountCase {
    fn render(&self) -> impl IntoElement {
        VStack::new()
            .child(Probe {
                mounts: self.mounts.clone(),
            })
            .disabled(*self.disabled.read())
    }
}

#[test]
fn disabling_a_container_does_not_rebuild_what_is_inside() {
    let mut disabled = State::create_global(false);
    let mounts = Rc::new(Cell::new(0));
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(MountCase {
            disabled,
            mounts: mounts.clone(),
        }),
        Size2D::new(40.0, 40.0),
        |_| {},
        1.0,
    );
    settle(&mut test);
    disabled.set(true);
    settle(&mut test);
    disabled.set(false);
    settle(&mut test);
    assert_eq!(mounts.get(), 1);
}

struct EnabledReader {
    seen: Rc<RefCell<Vec<bool>>>,
}

impl PartialEq for EnabledReader {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Component for EnabledReader {
    fn render(&self) -> impl IntoElement {
        self.seen.borrow_mut().push(use_is_enabled());
        rect()
    }
}

struct EnabledCase {
    disabled: State<bool>,
    seen: Rc<RefCell<Vec<bool>>>,
}

impl App for EnabledCase {
    fn render(&self) -> impl IntoElement {
        let reader = EnabledReader {
            seen: self.seen.clone(),
        };
        let outer = VStack::new().child(reader.into_element());
        VStack::new().child(outer.disabled(*self.disabled.read()).into_element())
    }
}

#[test]
fn use_is_enabled_is_true_outside_any_container_and_follows_the_container_above() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let mut disabled = State::create_global(false);
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(EnabledCase {
            disabled,
            seen: seen.clone(),
        }),
        Size2D::new(40.0, 40.0),
        |_| {},
        1.0,
    );
    settle(&mut test);
    assert_eq!(seen.borrow().last(), Some(&true));
    disabled.set(true);
    settle(&mut test);
    assert_eq!(seen.borrow().last(), Some(&false));
    disabled.set(false);
    settle(&mut test);
    assert_eq!(seen.borrow().last(), Some(&true));
}

#[derive(Clone, PartialEq)]
struct Account(&'static str);

impl Default for Account {
    fn default() -> Self {
        Self("nobody")
    }
}

struct AccountReader {
    seen: Rc<RefCell<Vec<&'static str>>>,
}

impl PartialEq for AccountReader {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Component for AccountReader {
    fn render(&self) -> impl IntoElement {
        self.seen.borrow_mut().push(use_environment::<Account>().0);
        rect()
    }
}

struct AccountCase {
    name: State<&'static str>,
    seen: Rc<RefCell<Vec<&'static str>>>,
    nested: bool,
}

impl App for AccountCase {
    fn render(&self) -> impl IntoElement {
        let reader = AccountReader {
            seen: self.seen.clone(),
        };
        if self.nested {
            VStack::new()
                .child(reader)
                .environment(Account("inner"))
                .environment(Account(*self.name.read()))
                .into_element()
        } else {
            VStack::new()
                .child(reader)
                .environment(Account(*self.name.read()))
                .into_element()
        }
    }
}

fn account_runner(
    nested: bool,
) -> (
    TestingRunner,
    State<&'static str>,
    Rc<RefCell<Vec<&'static str>>>,
) {
    let name = State::create_global("Ana");
    let seen = Rc::new(RefCell::new(Vec::new()));
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(AccountCase {
            name,
            seen: seen.clone(),
            nested,
        }),
        Size2D::new(40.0, 40.0),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    (test, name, seen)
}

#[test]
fn an_environment_value_reaches_its_readers_and_updates_them_when_it_changes() {
    let (mut test, mut name, seen) = account_runner(false);
    assert_eq!(seen.borrow().last(), Some(&"Ana"));
    name.set("Luis");
    settle(&mut test);
    assert_eq!(seen.borrow().last(), Some(&"Luis"));
}

#[test]
fn the_nearest_environment_value_of_a_type_wins() {
    let (mut test, mut name, seen) = account_runner(true);
    assert_eq!(seen.borrow().last(), Some(&"inner"));
    name.set("Luis");
    settle(&mut test);
    assert_eq!(seen.borrow().last(), Some(&"inner"));
}

struct NoProvider {
    seen: Rc<RefCell<Vec<&'static str>>>,
}

impl App for NoProvider {
    fn render(&self) -> impl IntoElement {
        AccountReader {
            seen: self.seen.clone(),
        }
    }
}

#[test]
fn a_reader_without_a_provider_gets_the_default() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(NoProvider { seen: seen.clone() }),
        Size2D::new(40.0, 40.0),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    assert_eq!(seen.borrow().last(), Some(&"nobody"));
}

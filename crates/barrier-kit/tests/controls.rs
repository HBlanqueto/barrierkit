//! Golden-image tests for every control in light and dark.

use std::{
    fs,
    path::{Path, PathBuf},
};

use barrier_kit::choices::{ColorWell, RadioGroup};
use barrier_kit::controls::{
    Button, ButtonStyle, Checkbox, Popover, ScrollView, SearchField, SecureField, SegmentedControl,
    Slider, Switch, TextField,
};
use barrier_kit::theme;
use freya_components::menu::{Menu, MenuItem};
use freya_components::theming::hooks::use_init_theme;
use freya_core::integration::{AppComponent, NavigationMode, Platform};
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use keyboard_types::{Key, NamedKey};
use torin::prelude::{Alignment, Direction, Gaps, Size, Size2D};
use vow_design::ACCENT_SWATCHES;

const SIZE: (f32, f32) = (360.0, 240.0);
const TOLERANCE: u8 = 3;

struct Case {
    dark: bool,
    build: fn() -> Element,
}

impl App for Case {
    fn render(&self) -> impl IntoElement {
        let dark = self.dark;
        use_init_theme(|| theme::for_scheme(dark));
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .direction(Direction::Vertical)
            .main_align(Alignment::Center)
            .cross_align(Alignment::Center)
            .padding(Gaps::new(24.0, 24.0, 24.0, 24.0))
            .child((self.build)())
    }
}

type Control = (&'static str, fn() -> Element);

fn cases() -> Vec<Control> {
    vec![
        ("button", button),
        ("switch", switch),
        ("slider", slider),
        ("segmented-control", segmented_control),
        ("text-field", text_field),
        ("checkbox", checkbox),
        ("radio-group", radio_group),
        ("color-well", color_well),
        ("secure-field", secure_field),
        ("scroll-view", scroll_view),
        ("menu", menu),
        ("popover", popover),
    ]
}

fn button() -> Element {
    rect()
        .direction(Direction::Horizontal)
        .spacing(12.0)
        .child(Button::new("Cancelar"))
        .child(Button::new("Guardar").style(ButtonStyle::Primary))
        .child(Button::new("Borrar").disabled(true))
        .into_element()
}

fn switch() -> Element {
    rect()
        .direction(Direction::Horizontal)
        .spacing(16.0)
        .child(Switch::new("Apagado"))
        .child(Switch::new("Encendido").on(true))
        .child(Switch::new("Inactivo").on(true).disabled(true))
        .into_element()
}

fn slider() -> Element {
    Slider::stepped(5)
        .value(2)
        .labels(&["A", "B", "C", "D", "E"])
        .into_element()
}

fn segmented_control() -> Element {
    SegmentedControl::new("Periodo")
        .segment("Día")
        .segment("Semana")
        .segment("Mes")
        .selected(1)
        .into_element()
}

fn radio_group() -> Element {
    rect()
        .direction(Direction::Vertical)
        .spacing(12.0)
        .child(
            RadioGroup::new("Aplicar el tinte a")
                .option("Solo las ventanas")
                .option("Todo el escritorio"),
        )
        .child(
            RadioGroup::new("Inactivo")
                .option("Uno")
                .option("Dos")
                .selected(1)
                .disabled(true),
        )
        .into_element()
}

fn color_well() -> Element {
    ColorWell::new("Color de acento", &ACCENT_SWATCHES)
        .selected(Some(0))
        .into_element()
}

fn checkbox() -> Element {
    Checkbox::new("Recordarme").checked(true).into_element()
}

fn text_field() -> Element {
    let name = use_state(|| "hblanqueto".to_string());
    let empty = use_state(String::new);
    let query = use_state(|| "wifi".to_string());
    rect()
        .width(Size::px(240.0))
        .direction(Direction::Vertical)
        .spacing(10.0)
        .child(TextField::new(name, "Nombre de usuario"))
        .child(TextField::new(empty, "Nombre de usuario"))
        .child(SearchField::new(query, "Buscar"))
        .into_element()
}

fn secure_field() -> Element {
    SecureField::new().into_element()
}

fn scroll_view() -> Element {
    ScrollView::new()
        .width(Size::px(200.0))
        .height(Size::px(120.0))
        .children([
            label().text("Uno").into_element(),
            label().text("Dos").into_element(),
            label().text("Tres").into_element(),
        ])
        .into_element()
}

fn menu() -> Element {
    Menu::new()
        .children([
            MenuItem::new().child("Abrir").into_element(),
            MenuItem::new().child("Guardar").into_element(),
        ])
        .into_element()
}

fn popover() -> Element {
    Popover::new()
        .children([
            label().text("Popover").into_element(),
            Button::new("Aceptar").into_element(),
        ])
        .into_element()
}

fn golden_path(name: &str, dark: bool) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(format!(
            "{name}-{}.png",
            if dark { "dark" } else { "light" }
        ))
}

fn render(_name: &str, dark: bool, build: fn() -> Element) -> Vec<u8> {
    let app = AppComponent::new(Case { dark, build });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(SIZE.0, SIZE.1), |_| {}, 1.0);
    test.sync_and_update();
    let data = test.render();
    data.as_ref().to_vec()
}

fn decode(path: &Path) -> (u32, u32, Vec<u8>) {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().unwrap();
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).unwrap();
    let rgba = match info.color_type {
        png::ColorType::Rgba => buffer[..info.buffer_size()].to_vec(),
        png::ColorType::Rgb => buffer[..info.buffer_size()]
            .chunks(3)
            .flat_map(|px| [px[0], px[1], px[2], 255])
            .collect(),
        other => panic!("unexpected PNG colour type {other:?}"),
    };
    (info.width, info.height, rgba)
}

fn compare(golden: &Path, actual_png: &[u8], name: &str, dark: bool) {
    let actual_path = std::env::temp_dir().join(format!(
        "malaysia-control-{name}-{}.png",
        if dark { "dark" } else { "light" }
    ));
    fs::write(&actual_path, actual_png).unwrap();
    let (gw, gh, golden_px) = decode(golden);
    let (aw, ah, actual_px) = decode(&actual_path);
    assert_eq!((gw, gh), (aw, ah), "{name}: image size changed");
    let bad = golden_px
        .iter()
        .zip(&actual_px)
        .filter(|(a, b)| a.abs_diff(**b) > TOLERANCE)
        .count();
    assert_eq!(
        bad,
        0,
        "{name} ({}): {bad} channels differ by more than {TOLERANCE}; actual at {actual_path:?}",
        if dark { "dark" } else { "light" }
    );
}

#[test]
fn controls_match_their_golden_images() {
    let update = std::env::var_os("UPDATE_GOLDEN").is_some();
    for (name, build) in cases() {
        for dark in [false, true] {
            let path = golden_path(name, dark);
            let actual = render(name, dark, build);
            if update || !path.exists() {
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(&path, &actual).unwrap();
            } else {
                compare(&path, &actual, name, dark);
            }
        }
    }
}

#[test]
fn slider_click_snaps_to_the_nearest_step() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let values: Rc<RefCell<Vec<usize>>> = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    struct SliderCase {
        values: Rc<RefCell<Vec<usize>>>,
    }
    impl App for SliderCase {
        fn render(&self) -> impl IntoElement {
            let values = self.values.clone();
            Slider::stepped(4)
                .value(1)
                .labels(&["Sutil", "Normal", "Fuerte", "Intenso"])
                .on_change(move |v| values.borrow_mut().push(v))
        }
    }

    let app = AppComponent::new(SliderCase {
        values: values_clone,
    });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(240.0, 80.0), |_| {}, 1.0);
    test.sync_and_update();

    test.click_cursor((190.0, 20.0));
    test.sync_and_update();
    assert_eq!(values.borrow().as_slice(), &[3]);

    test.click_cursor((130.0, 20.0));
    test.sync_and_update();
    assert_eq!(values.borrow().as_slice(), &[3, 2]);
}

#[test]
fn disabled_slider_does_not_emit() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let values: Rc<RefCell<Vec<usize>>> = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    struct SliderCase {
        values: Rc<RefCell<Vec<usize>>>,
    }
    impl App for SliderCase {
        fn render(&self) -> impl IntoElement {
            let values = self.values.clone();
            Slider::stepped(4)
                .value(1)
                .disabled(true)
                .on_change(move |v| values.borrow_mut().push(v))
        }
    }

    let app = AppComponent::new(SliderCase {
        values: values_clone,
    });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(240.0, 80.0), |_| {}, 1.0);
    test.sync_and_update();

    test.click_cursor((190.0, 20.0));
    test.sync_and_update();
    assert!(values.borrow().is_empty());
}

#[test]
fn checkbox_click_toggles_and_disabled_does_not_emit() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let values: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    struct CheckboxCase {
        values: Rc<RefCell<Vec<bool>>>,
    }
    impl App for CheckboxCase {
        fn render(&self) -> impl IntoElement {
            let values = self.values.clone();
            Checkbox::new("Recordarme")
                .checked(false)
                .on_toggle(move |v| values.borrow_mut().push(v))
        }
    }

    let app = AppComponent::new(CheckboxCase {
        values: values_clone,
    });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(160.0, 60.0), |_| {}, 1.0);
    test.sync_and_update();

    test.click_cursor((8.0, 8.0));
    test.sync_and_update();
    assert_eq!(values.borrow().as_slice(), &[true]);

    test.click_cursor((8.0, 8.0));
    test.sync_and_update();
    assert_eq!(values.borrow().as_slice(), &[true, false]);
}

#[test]
fn switch_click_toggles_and_disabled_does_not_emit() {
    use std::cell::RefCell;
    use std::rc::Rc;

    struct SwitchCase {
        disabled: bool,
        values: Rc<RefCell<Vec<bool>>>,
    }
    impl App for SwitchCase {
        fn render(&self) -> impl IntoElement {
            let values = self.values.clone();
            Switch::new("Wi-Fi")
                .disabled(self.disabled)
                .on_toggle(move |v| values.borrow_mut().push(v))
        }
    }

    for (disabled, expected) in [(false, vec![true, false]), (true, vec![])] {
        let values = Rc::new(RefCell::new(Vec::new()));
        let app = AppComponent::new(SwitchCase {
            disabled,
            values: values.clone(),
        });
        let (mut test, _) = TestingRunner::new(app, Size2D::new(60.0, 30.0), |_| {}, 1.0);
        test.sync_and_update();
        for _ in 0..2 {
            test.click_cursor((10.0, 9.0));
            test.sync_and_update();
        }
        assert_eq!(*values.borrow(), expected);
    }
}

#[test]
fn segmented_control_click_selects_once_per_new_segment() {
    use std::cell::RefCell;
    use std::rc::Rc;

    struct SegmentedCase {
        values: Rc<RefCell<Vec<usize>>>,
    }
    impl App for SegmentedCase {
        fn render(&self) -> impl IntoElement {
            let values = self.values.clone();
            SegmentedControl::new("Vista")
                .segment("Cuadrícula")
                .segment("Lista")
                .on_change(move |i| values.borrow_mut().push(i))
        }
    }

    let values = Rc::new(RefCell::new(Vec::new()));
    let app = AppComponent::new(SegmentedCase {
        values: values.clone(),
    });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(200.0, 30.0), |_| {}, 1.0);
    test.sync_and_update();
    for x in [150.0, 150.0, 50.0] {
        test.click_cursor((x, 15.0));
        test.sync_and_update();
    }
    assert_eq!(*values.borrow(), [1, 0]);
}

struct FocusCase {
    keyboard: bool,
    switch: State<bool>,
    checkbox: State<bool>,
    segment: State<usize>,
}

impl App for FocusCase {
    fn render(&self) -> impl IntoElement {
        let (mut switch, mut checkbox, mut segment) = (self.switch, self.checkbox, self.segment);
        if self.keyboard {
            Platform::get()
                .navigation_mode
                .set_if_modified(NavigationMode::Keyboard);
        }
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .direction(Direction::Vertical)
            .main_align(Alignment::Center)
            .cross_align(Alignment::Center)
            .padding(Gaps::new(24.0, 24.0, 24.0, 24.0))
            .spacing(16.0)
            .child(Switch::new("Wi-Fi").on_toggle(move |v| switch.set(v)))
            .child(Checkbox::new("Recordarme").on_toggle(move |v| checkbox.set(v)))
            .child(
                SegmentedControl::new("Vista")
                    .segment("Cuadrícula")
                    .segment("Lista")
                    .on_change(move |i| segment.set(i)),
            )
    }
}

fn focus_case(keyboard: bool) -> (TestingRunner, State<bool>, State<bool>, State<usize>) {
    let switch = State::create_global(false);
    let checkbox = State::create_global(false);
    let segment = State::create_global(0);
    let app = AppComponent::new(FocusCase {
        keyboard,
        switch,
        checkbox,
        segment,
    });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(SIZE.0, SIZE.1), |_| {}, 1.0);
    test.sync_and_update();
    (test, switch, checkbox, segment)
}

fn press(test: &mut TestingRunner, key: Key) {
    test.press_key(key);
    test.sync_and_update();
}

#[test]
fn tab_reaches_each_control_and_space_activates_it() {
    let (mut test, switch, checkbox, segment) = focus_case(false);
    let space = || Key::Character(" ".into());

    press(&mut test, Key::Named(NamedKey::Tab));
    press(&mut test, space());
    press(&mut test, Key::Named(NamedKey::Tab));
    press(&mut test, space());
    press(&mut test, Key::Named(NamedKey::Tab));
    press(&mut test, Key::Named(NamedKey::Tab));
    press(&mut test, Key::Named(NamedKey::Enter));

    assert!(*switch.peek());
    assert!(*checkbox.peek());
    assert_eq!(*segment.peek(), 1);
}

#[test]
fn keyboard_focus_ring_matches_its_golden_image() {
    let (mut test, ..) = focus_case(true);
    press(&mut test, Key::Named(NamedKey::Tab));
    test.poll_n(std::time::Duration::from_millis(16), 3);
    let actual = test.render().as_ref().to_vec();
    let path = golden_path("focus-ring", false);
    if std::env::var_os("UPDATE_GOLDEN").is_some() || !path.exists() {
        fs::write(&path, &actual).unwrap();
    } else {
        compare(&path, &actual, "focus-ring", false);
    }
}

#[test]
fn button_press_emits_only_when_enabled() {
    use std::cell::RefCell;
    use std::rc::Rc;

    struct ButtonCase {
        disabled: bool,
        presses: Rc<RefCell<u32>>,
    }
    impl App for ButtonCase {
        fn render(&self) -> impl IntoElement {
            let presses = self.presses.clone();
            Button::new("Guardar")
                .disabled(self.disabled)
                .on_press(move |_| *presses.borrow_mut() += 1)
        }
    }

    for (disabled, expected) in [(false, 1), (true, 0)] {
        let presses = Rc::new(RefCell::new(0));
        let app = AppComponent::new(ButtonCase {
            disabled,
            presses: presses.clone(),
        });
        let (mut test, _) = TestingRunner::new(app, Size2D::new(120.0, 40.0), |_| {}, 1.0);
        test.sync_and_update();
        test.click_cursor((20.0, 15.0));
        test.sync_and_update();
        assert_eq!(*presses.borrow(), expected);
    }
}

#[test]
fn search_field_clear_button_empties_the_query() {
    struct SearchCase {
        query: State<String>,
    }
    impl App for SearchCase {
        fn render(&self) -> impl IntoElement {
            SearchField::new(self.query, "Buscar")
        }
    }

    let query = State::create_global("wifi".to_string());
    let app = AppComponent::new(SearchCase { query });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(200.0, 30.0), |_| {}, 1.0);
    test.sync_and_update();
    test.click_cursor((180.0, 15.0));
    test.sync_and_update();
    assert!(query.peek().is_empty());
}

#[test]
fn radio_group_click_selects_and_disabled_does_not_emit() {
    use std::cell::RefCell;
    use std::rc::Rc;

    struct RadioCase {
        disabled: bool,
        values: Rc<RefCell<Vec<usize>>>,
    }
    impl App for RadioCase {
        fn render(&self) -> impl IntoElement {
            let values = self.values.clone();
            RadioGroup::new("Alcance")
                .option("Ventanas")
                .option("Escritorio")
                .disabled(self.disabled)
                .on_change(move |i| values.borrow_mut().push(i))
        }
    }

    for (disabled, expected) in [(false, vec![1]), (true, vec![])] {
        let values = Rc::new(RefCell::new(Vec::new()));
        let app = AppComponent::new(RadioCase {
            disabled,
            values: values.clone(),
        });
        let (mut test, _) = TestingRunner::new(app, Size2D::new(300.0, 30.0), |_| {}, 1.0);
        test.sync_and_update();
        test.click_cursor((110.0, 9.0));
        test.sync_and_update();
        assert_eq!(*values.borrow(), expected);
    }
}

#[test]
fn disabled_checkbox_does_not_emit() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let values: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    struct CheckboxCase {
        values: Rc<RefCell<Vec<bool>>>,
    }
    impl App for CheckboxCase {
        fn render(&self) -> impl IntoElement {
            let values = self.values.clone();
            Checkbox::new("Recordarme")
                .checked(false)
                .disabled(true)
                .on_toggle(move |v| values.borrow_mut().push(v))
        }
    }

    let app = AppComponent::new(CheckboxCase {
        values: values_clone,
    });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(160.0, 60.0), |_| {}, 1.0);
    test.sync_and_update();

    test.click_cursor((8.0, 8.0));
    test.sync_and_update();
    assert!(values.borrow().is_empty());
}

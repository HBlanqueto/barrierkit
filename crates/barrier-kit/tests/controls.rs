//! Golden images for every named control, in light and dark (ADR 0034).
//!
//! `UPDATE_GOLDEN=1 cargo test -p barrier-kit --test controls` refreshes the
//! references; a normal run compares every pixel with a small tolerance so
//! antialiasing differences are ignored but layout or colour drift is not.

use std::{
    fs,
    path::{Path, PathBuf},
};

use barrier_kit::controls::{
    Button, Checkbox, Popover, ScrollView, SecureField, SegmentedControl, Slider, Switch, TextField,
};
use barrier_kit::theme;
use freya_components::menu::{Menu, MenuItem};
use freya_components::segmented_button::ButtonSegment;
use freya_components::theming::hooks::use_init_theme;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Alignment, Direction, Gaps, Size, Size2D};

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
        ("secure-field", secure_field),
        ("scroll-view", scroll_view),
        ("menu", menu),
        ("popover", popover),
    ]
}

fn button() -> Element {
    Button::new().child("Guardar").into_element()
}

fn switch() -> Element {
    Switch::new().into_element()
}

fn slider() -> Element {
    Slider::stepped(5)
        .value(2)
        .labels(&["A", "B", "C", "D", "E"])
        .into_element()
}

fn segmented_control() -> Element {
    SegmentedControl::new()
        .children([
            ButtonSegment::new().child("Uno").into_element(),
            ButtonSegment::new()
                .selected(true)
                .child("Dos")
                .into_element(),
        ])
        .into_element()
}

fn checkbox() -> Element {
    Checkbox::new("Recordarme").checked(true).into_element()
}

fn text_field() -> Element {
    let value = use_state(|| "Hola".to_string());
    TextField::new(value).into_element()
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
            Button::new().child("Aceptar").into_element(),
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

    // Width 200, knob 20, run 180, four steps at x=10, 70, 130, 190.
    // Click near the right end to select Intenso (3).
    test.click_cursor((190.0, 20.0));
    test.sync_and_update();
    assert_eq!(values.borrow().as_slice(), &[3]);

    // Click near the middle to select Fuerte (2).
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

    let app = AppComponent::new(SliderCase { values: values_clone });
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

    let app = AppComponent::new(CheckboxCase { values: values_clone });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(160.0, 60.0), |_| {}, 1.0);
    test.sync_and_update();

    test.click_cursor((8.0, 8.0));
    test.sync_and_update();
    assert!(values.borrow().is_empty());
}

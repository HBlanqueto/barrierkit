use std::rc::Rc;

use bk::controls::Button;
use bk::elements::{Label, Text};
use bk::styles::ButtonStyle;
use bk::symbol::Symbol;
use bk::theme;
use ctappearance::{AppearanceConfig, ColorScheme, resolve};
use ctdesign::SANS_FAMILIES;
use freya_components::theming::hooks::use_init_theme;
use freya_core::integration::AppComponent;
use freya_core::prelude::Label as TextRun;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Gaps, Size2D};

const SIZE: (f32, f32) = (200.0, 70.0);
const CURSOR: (f64, f64) = (60.0, 35.0);

#[derive(Clone, Copy, Debug)]
enum Look {
    Plain,
    Prominent,
    Disabled,
}

#[derive(Clone, Copy, Debug)]
enum View {
    Drawn,
    DrawnSymbol,
    FreyaLabel,
    KitText,
    KitLabel,
}

fn button(look: Look, view: View) -> Element {
    let button = match view {
        View::Drawn => Button::new("Guardar"),
        View::DrawnSymbol => Button::system_image("Guardar", Symbol::Plus),
        View::FreyaLabel => Button::new("Guardar").label(|| label().text("Guardar")),
        View::KitText => Button::new("Guardar").label(|| Text::new("Guardar")),
        View::KitLabel => Button::new("Guardar").label(|| Label::new("Guardar", Symbol::Plus)),
    };
    match look {
        Look::Plain => button,
        Look::Prominent => button.style(ButtonStyle::bordered_prominent),
        Look::Disabled => button.disabled(true),
    }
    .into_element()
}

struct Case {
    dark: bool,
    build: Rc<dyn Fn() -> Element>,
}

impl App for Case {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(self.dark));
        rect()
            .expanded()
            .padding(Gaps::new(24.0, 24.0, 24.0, 24.0))
            .child((self.build)())
    }
}

fn runner(dark: bool, build: Rc<dyn Fn() -> Element>) -> TestingRunner {
    let resolved = resolve(
        &AppearanceConfig {
            color_scheme: if dark {
                ColorScheme::Dark
            } else {
                ColorScheme::Light
            },
            reduce_transparency: true,
            reduced_motion: true,
            ..Default::default()
        },
        None,
    );
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Case { dark, build }),
        Size2D::new(SIZE.0, SIZE.1),
        move |runner| runner.provide_root_context(move || State::create(resolved)),
        1.0,
    );
    test.set_default_fonts(&SANS_FAMILIES.map(std::borrow::Cow::Borrowed));
    test.sync_and_update();
    test.sync_and_update();
    test
}

fn pixels(dark: bool, pressed: bool, look: Look, view: View) -> Vec<u8> {
    let mut test = runner(dark, Rc::new(move || button(look, view)));
    if pressed {
        test.move_cursor(CURSOR);
        test.press_cursor(CURSOR);
        test.sync_and_update();
        test.sync_and_update();
    }
    crate::image::decode(test.render().as_bytes()).2
}

fn differing(a: &[u8], b: &[u8]) -> usize {
    a.iter()
        .zip(b)
        .filter(|(a, b)| a.abs_diff(**b) > crate::controls::TOLERANCE)
        .count()
}

#[test]
fn a_view_label_draws_like_the_label_the_button_would_draw() {
    let pairs = [
        (View::Drawn, View::FreyaLabel),
        (View::Drawn, View::KitText),
        (View::DrawnSymbol, View::KitLabel),
    ];
    for (drawn, view) in pairs {
        for look in [Look::Plain, Look::Prominent, Look::Disabled] {
            for dark in [false, true] {
                for pressed in [false, true] {
                    let expected = pixels(dark, pressed, look, drawn);
                    let actual = pixels(dark, pressed, look, view);
                    assert_eq!(
                        differing(&expected, &actual),
                        0,
                        "{view:?} in a {look:?} button (dark {dark}, pressed {pressed}) differs from {drawn:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn the_states_compared_are_not_all_the_same_picture() {
    let normal = pixels(false, false, Look::Prominent, View::Drawn);
    let pressed = pixels(false, true, Look::Prominent, View::Drawn);
    let disabled = pixels(false, false, Look::Disabled, View::Drawn);
    assert!(differing(&normal, &pressed) > 0);
    assert!(differing(&normal, &disabled) > 0);
}

#[test]
fn a_view_label_follows_state_changes_and_never_stales() {
    struct Counter;

    impl App for Counter {
        fn render(&self) -> impl IntoElement {
            let mut presses = use_state(|| 0);
            let seen_by_parent = *presses.read();
            rect()
                .child(
                    Button::new("Contar")
                        .on_press(move |_| *presses.write() += 1)
                        .label(move || label().text(format!("parent {seen_by_parent}"))),
                )
                .child(
                    Button::new("Contar")
                        .label(move || label().text(format!("scope {}", presses.read()))),
                )
        }
    }

    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Counter),
        Size2D::new(400.0, 200.0),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    test.sync_and_update();
    let shows = |test: &TestingRunner, text: &str| {
        test.find(|_, element| TextRun::try_downcast(element).filter(|run| run.text == text))
            .is_some()
    };
    assert!(shows(&test, "parent 0") && shows(&test, "scope 0"));
    test.click_cursor((20.0, 10.0));
    test.sync_and_update();
    test.sync_and_update();
    assert!(
        shows(&test, "parent 1"),
        "the parent's value reaches the label"
    );
    assert!(
        shows(&test, "scope 1"),
        "a state read inside the label reaches it"
    );
}

#[test]
fn the_string_given_to_new_stays_the_accessible_name() {
    let test = runner(
        false,
        Rc::new(|| {
            Button::new("Guardar")
                .label(|| Label::new("Otro texto", Symbol::Plus))
                .into_element()
        }),
    );
    let named = test.find(|_, element| {
        (element.accessibility().builder.label() == Some("Guardar")).then_some(())
    });
    assert!(named.is_some());
    let renamed = test.find(|_, element| {
        (element.accessibility().builder.label() == Some("Otro texto")).then_some(())
    });
    assert!(renamed.is_none());
}

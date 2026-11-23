use bk::controls::Button;
use bk::elements::{Label, Text};
use bk::styles::ButtonStyle;
use bk::symbol::Symbol;
use ctappearance::AppearanceConfig;
use ctdesign::TEXT_SIZES;
use freya_core::integration::AppComponent;
use freya_core::prelude::Label as TextRun;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Gaps, Size2D};

use crate::support::{self, differing, scheme};

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

fn runner(dark: bool, text_size: f32, build: impl Fn() -> Element + 'static) -> TestingRunner {
    support::runner(
        SIZE,
        AppearanceConfig {
            text_size,
            ..scheme(dark)
        },
        move || {
            rect()
                .expanded()
                .padding(Gaps::new(24.0, 24.0, 24.0, 24.0))
                .child(build())
                .into_element()
        },
    )
}

fn pixels(dark: bool, text_size: f32, pressed: bool, look: Look, view: View) -> Vec<u8> {
    let mut test = runner(dark, text_size, move || button(look, view));
    if pressed {
        test.move_cursor(CURSOR);
        test.press_cursor(CURSOR);
        test.sync_and_update();
        test.sync_and_update();
    }
    crate::image::decode(test.render().as_bytes()).2
}

#[test]
fn a_view_label_draws_like_the_label_the_button_would_draw() {
    let sizes = [TEXT_SIZES[0], TEXT_SIZES[2]];
    let pairs = [
        (View::Drawn, View::FreyaLabel, &sizes[..1]),
        (View::Drawn, View::KitText, &sizes[..]),
        (View::DrawnSymbol, View::KitLabel, &sizes[..]),
    ];
    for (drawn, view, sizes) in pairs {
        for look in [Look::Plain, Look::Prominent, Look::Disabled] {
            for dark in [false, true] {
                for &text_size in sizes {
                    for pressed in [false, true] {
                        let expected = pixels(dark, text_size, pressed, look, drawn);
                        let actual = pixels(dark, text_size, pressed, look, view);
                        assert_eq!(
                            differing(&expected, &actual),
                            0,
                            "{view:?} in a {look:?} button (dark {dark}, text size {text_size}, pressed {pressed}) differs from {drawn:?}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn the_states_compared_are_not_all_the_same_picture() {
    let normal = pixels(false, TEXT_SIZES[0], false, Look::Prominent, View::Drawn);
    let pressed = pixels(false, TEXT_SIZES[0], true, Look::Prominent, View::Drawn);
    let disabled = pixels(false, TEXT_SIZES[0], false, Look::Disabled, View::Drawn);
    let larger = pixels(false, TEXT_SIZES[2], false, Look::Prominent, View::Drawn);
    assert!(differing(&normal, &pressed) > 0);
    assert!(differing(&normal, &disabled) > 0);
    assert!(differing(&normal, &larger) > 0);
}

fn size_at(text_size: f32, view: View) -> (f32, f32) {
    let test = runner(false, text_size, move || button(Look::Plain, view));
    test.find(|node, element| {
        (element.accessibility().builder.label() == Some("Guardar")).then(|| {
            let area = node.layout().area;
            (area.width(), area.height())
        })
    })
    .expect("a node labelled Guardar")
}

#[test]
fn a_larger_text_size_widens_the_button_and_keeps_its_height() {
    for view in [
        View::Drawn,
        View::DrawnSymbol,
        View::FreyaLabel,
        View::KitText,
        View::KitLabel,
    ] {
        let normal = size_at(TEXT_SIZES[0], view);
        let larger = size_at(TEXT_SIZES[2], view);
        assert!(
            larger.0 > normal.0,
            "{view:?}: width {normal:?} -> {larger:?}"
        );
        assert_eq!(
            larger.1, normal.1,
            "{view:?}: height {normal:?} -> {larger:?}"
        );
    }
    let width = |view| size_at(TEXT_SIZES[2], view).0;
    assert_eq!(width(View::FreyaLabel), width(View::Drawn));
    assert_eq!(width(View::KitText), width(View::Drawn));
    assert_eq!(width(View::KitLabel), width(View::DrawnSymbol));
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
    let test = runner(false, TEXT_SIZES[0], || {
        Button::new("Guardar")
            .label(|| Label::new("Otro texto", Symbol::Plus))
            .into_element()
    });
    let named = test.find(|_, element| {
        (element.accessibility().builder.label() == Some("Guardar")).then_some(())
    });
    assert!(named.is_some());
    let renamed = test.find(|_, element| {
        (element.accessibility().builder.label() == Some("Otro texto")).then_some(())
    });
    assert!(renamed.is_none());
}

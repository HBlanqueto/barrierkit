use std::{fs, path::PathBuf};

use bk::buttons::HelpLink;
use bk::choices::RadioGroup;
use bk::compact::Stepper;
use bk::controls::{Button, Checkbox, ControlSize, SegmentedControl, Slider, Switch};
use bk::popup_button::PopUpButton;
use bk::styles::{ButtonStyle, Edge, Shape, SwitchStyle};
use bk::symbol::Symbol;
use freya_core::prelude::*;
use torin::prelude::{Gaps, Size};

use super::appkit::{Builder, layout, render_scene, scene_content, scene_tag};

const SIZES: [(&str, ControlSize); 5] = [
    ("mini", ControlSize::Mini),
    ("small", ControlSize::Small),
    ("regular", ControlSize::Regular),
    ("large", ControlSize::Large),
    ("xl", ControlSize::ExtraLarge),
];

const SEGMENTED_WIDTHS: [f32; 5] = [134.0, 146.0, 158.0, 170.0, 194.0];

const STEPPER_FRAMES: [(f32, f32); 5] = [
    (13.0, 16.0),
    (17.0, 20.0),
    (20.0, 24.0),
    (23.0, 28.0),
    (30.0, 36.0),
];

macro_rules! sizes {
    ($build:ident) => {
        [
            $build::<0> as Builder,
            $build::<1> as Builder,
            $build::<2> as Builder,
            $build::<3> as Builder,
            $build::<4> as Builder,
        ]
    };
}

fn size<const N: usize>() -> ControlSize {
    SIZES[N].1
}

fn push<const N: usize>() -> Element {
    Button::new("Button").size(size::<N>()).into_element()
}

fn push_default<const N: usize>() -> Element {
    Button::new("Default")
        .size(size::<N>())
        .style(bk::styles::ButtonStyle::bordered_prominent)
        .into_element()
}

fn push_disabled<const N: usize>() -> Element {
    Button::new("Button")
        .size(size::<N>())
        .disabled(true)
        .into_element()
}

fn popup<const N: usize>() -> Element {
    PopUpButton::new("Option")
        .item("Option")
        .item("Other")
        .size(size::<N>())
        .into_element()
}

fn popup_disabled<const N: usize>() -> Element {
    PopUpButton::new("Option")
        .item("Option")
        .item("Other")
        .size(size::<N>())
        .disabled(true)
        .into_element()
}

fn pulldown<const N: usize>() -> Element {
    PopUpButton::new("Pull")
        .item("Pull")
        .item("A")
        .pulls_down(true)
        .size(size::<N>())
        .into_element()
}

fn segmented_with<const N: usize>(disabled: bool) -> Element {
    rect()
        .width(Size::px(SEGMENTED_WIDTHS[N]))
        .child(
            SegmentedControl::new("Seg")
                .segment("One")
                .segment("Two")
                .segment("Three")
                .selected(0)
                .size(size::<N>())
                .disabled(disabled),
        )
        .into_element()
}

fn segmented<const N: usize>() -> Element {
    segmented_with::<N>(false)
}

fn check<const N: usize>() -> Element {
    Checkbox::new("Check").size(size::<N>()).into_element()
}

fn check_on<const N: usize>() -> Element {
    Checkbox::new("Check")
        .checked(true)
        .size(size::<N>())
        .into_element()
}

fn check_disabled<const N: usize>() -> Element {
    Checkbox::new("Check")
        .size(size::<N>())
        .disabled(true)
        .into_element()
}

fn check_on_disabled<const N: usize>() -> Element {
    Checkbox::new("Check")
        .checked(true)
        .size(size::<N>())
        .disabled(true)
        .into_element()
}

fn check_mixed<const N: usize>() -> Element {
    Checkbox::new("Check")
        .mixed(true)
        .size(size::<N>())
        .into_element()
}

fn radio_with<const N: usize>(selected: usize, disabled: bool) -> Element {
    RadioGroup::new("Radio")
        .option("Radio")
        .selected(selected)
        .size(size::<N>())
        .disabled(disabled)
        .into_element()
}

fn radio<const N: usize>() -> Element {
    radio_with::<N>(1, false)
}

fn radio_on<const N: usize>() -> Element {
    radio_with::<N>(0, false)
}

fn radio_disabled<const N: usize>() -> Element {
    radio_with::<N>(1, true)
}

fn radio_on_disabled<const N: usize>() -> Element {
    radio_with::<N>(0, true)
}

fn framed((width, height): (f32, f32), child: impl IntoElement) -> Element {
    rect()
        .width(Size::px(width))
        .height(Size::px(height))
        .center()
        .child(child)
        .into_element()
}

fn switch<const N: usize>() -> Element {
    Switch::new("Switch").size(size::<N>()).into_element()
}

fn switch_on<const N: usize>() -> Element {
    Switch::new("Switch")
        .on(true)
        .size(size::<N>())
        .into_element()
}

fn switch_on_disabled<const N: usize>() -> Element {
    Switch::new("Switch")
        .on(true)
        .size(size::<N>())
        .disabled(true)
        .into_element()
}

fn slider_with<const N: usize>(disabled: bool) -> Element {
    Slider::new("Slider")
        .value(0.5)
        .size(size::<N>())
        .disabled(disabled)
        .style(|mut style| {
            style.width = 160.0;
            style
        })
        .into_element()
}

fn slider<const N: usize>() -> Element {
    slider_with::<N>(false)
}

fn slider_disabled<const N: usize>() -> Element {
    slider_with::<N>(true)
}

fn stepper_with<const N: usize>(disabled: bool) -> Element {
    framed(
        STEPPER_FRAMES[N],
        Stepper::new("Stepper", 0, 10)
            .value(5)
            .size(size::<N>())
            .disabled(disabled)
            .style(|mut style| {
                style.value_position = None;
                style
            }),
    )
}

fn stepper<const N: usize>() -> Element {
    stepper_with::<N>(false)
}

fn stepper_disabled<const N: usize>() -> Element {
    stepper_with::<N>(true)
}

fn offset(left: f32, top: f32, child: impl IntoElement) -> Element {
    rect()
        .margin(Gaps::new(top, 0.0, 0.0, left))
        .child(child)
        .into_element()
}

fn bezel(label: &'static str, style: fn(ButtonStyle) -> ButtonStyle) -> Button {
    Button::new(label).style(style)
}

fn chevron(style: fn(ButtonStyle) -> ButtonStyle, disabled: bool) -> Element {
    Button::system_image("", Symbol::ChevronDown)
        .style(style)
        .disabled(disabled)
        .into_element()
}

fn bezel_builders() -> Vec<(&'static str, Builder)> {
    vec![
        ("bezel-push", || Button::new("Button").into_element()),
        ("bezel-flexible-push", || {
            Button::new("Button").into_element()
        }),
        ("bezel-round-rect", || Button::new("Button").into_element()),
        ("bezel-recessed", || Button::new("Button").into_element()),
        ("bezel-accessory-bar", || {
            Button::new("Button").into_element()
        }),
        ("bezel-accessory-bar-action", || {
            Button::new("Button").into_element()
        }),
        ("bezel-small-square", || {
            offset(0.0, 0.5, bezel("Button", ButtonStyle::small_square))
        }),
        ("bezel-textured-rounded", || {
            offset(1.0, 1.0, bezel("Button", ButtonStyle::textured_rounded))
        }),
        ("bezel-toolbar", || {
            offset(1.0, 1.0, bezel("Button", ButtonStyle::textured_rounded))
        }),
        ("bezel-textured-square", || {
            offset(2.0, 2.0, bezel("Button", ButtonStyle::textured_square))
        }),
        ("bezel-inline", || {
            bezel("Button", ButtonStyle::inline).into_element()
        }),
        ("bezel-push-disclosure", || {
            chevron(ButtonStyle::push_disclosure, false)
        }),
        ("bezel-rounded-disclosure", || {
            chevron(ButtonStyle::push_disclosure, false)
        }),
        ("bezel-help", || HelpLink::new().into_element()),
        ("bezel-circular", || {
            offset(
                3.0,
                0.0,
                Button::system_image("", Symbol::Plus).style(ButtonStyle::circular),
            )
        }),
    ]
}

fn bezel_disabled_builders() -> Vec<(&'static str, Builder)> {
    vec![
        ("segmented-disabled", || segmented_with::<2>(true)),
        ("bezel-push-disabled", || {
            Button::new("Button").disabled(true).into_element()
        }),
        ("bezel-small-square-disabled", || {
            offset(
                0.0,
                0.5,
                bezel("Button", ButtonStyle::small_square).disabled(true),
            )
        }),
        ("bezel-textured-rounded-disabled", || {
            offset(
                1.0,
                1.0,
                bezel("Button", ButtonStyle::textured_rounded).disabled(true),
            )
        }),
        ("bezel-textured-square-disabled", || {
            offset(
                2.0,
                2.0,
                bezel("Button", ButtonStyle::textured_square).disabled(true),
            )
        }),
        ("bezel-inline-disabled", || {
            bezel("Button", ButtonStyle::inline)
                .disabled(true)
                .into_element()
        }),
        ("bezel-push-disclosure-disabled", || {
            chevron(ButtonStyle::push_disclosure, true)
        }),
    ]
}

fn bezel_pressed_builders() -> Vec<(&'static str, Builder)> {
    vec![
        ("bezel-push-pressed", || {
            Button::new("Button").into_element()
        }),
        ("bezel-small-square-pressed", || {
            offset(0.0, 0.5, bezel("Button", ButtonStyle::small_square))
        }),
        ("bezel-textured-rounded-pressed", || {
            offset(1.0, 1.0, bezel("Button", ButtonStyle::textured_rounded))
        }),
        ("bezel-textured-square-pressed", || {
            offset(2.0, 2.0, bezel("Button", ButtonStyle::textured_square))
        }),
        ("bezel-inline-pressed", || {
            bezel("Button", ButtonStyle::inline).into_element()
        }),
        ("bezel-push-disclosure-pressed", || {
            chevron(ButtonStyle::push_disclosure, false)
        }),
    ]
}

fn named(builders: Vec<(&'static str, Builder)>) -> Vec<(String, Builder)> {
    builders
        .into_iter()
        .map(|(name, build)| (name.to_string(), build))
        .collect()
}

fn each(prefix: &str, builds: [Builder; 5]) -> Vec<(String, Builder)> {
    SIZES
        .iter()
        .zip(builds)
        .map(|((name, _), build)| (format!("{prefix}-{name}"), build))
        .collect()
}

fn buttons() -> Vec<(String, Builder)> {
    [
        each("push", sizes!(push)),
        each("default", sizes!(push_default)),
        each("push-disabled", sizes!(push_disabled)),
        each("popup", sizes!(popup)),
        each("popup-disabled", sizes!(popup_disabled)),
        each("pulldown", sizes!(pulldown)),
        each("segmented", sizes!(segmented)),
        each("check-off", sizes!(check)),
        each("check-on", sizes!(check_on)),
        each("check-mixed", sizes!(check_mixed)),
        each("check-off-disabled", sizes!(check_disabled)),
        each("check-on-disabled", sizes!(check_on_disabled)),
        each("radio-off", sizes!(radio)),
        each("radio-on", sizes!(radio_on)),
        each("radio-off-disabled", sizes!(radio_disabled)),
        each("radio-on-disabled", sizes!(radio_on_disabled)),
        each("switch-off", sizes!(switch)),
        each("switch-on", sizes!(switch_on)),
        each("switch-on-disabled", sizes!(switch_on_disabled)),
        each("slider", sizes!(slider)),
        each("slider-disabled", sizes!(slider_disabled)),
        each("stepper", sizes!(stepper)),
        each("stepper-disabled", sizes!(stepper_disabled)),
        named(bezel_builders()),
        named(bezel_disabled_builders()),
    ]
    .concat()
}

fn pressed() -> Vec<(String, Builder)> {
    [
        each("push-pressed", sizes!(push)),
        each("default-pressed", sizes!(push_default)),
        each("popup-pressed", sizes!(popup)),
        each("check-off-pressed", sizes!(check)),
        each("check-on-pressed", sizes!(check_on)),
        each("radio-off-pressed", sizes!(radio)),
        each("radio-on-pressed", sizes!(radio_on)),
        named(bezel_pressed_builders()),
    ]
    .concat()
}

fn file(dark: bool, black: bool, middle: &str) -> String {
    format!(
        "kit{}{middle}-{}-{}.png",
        scene_tag(),
        if dark { "dark" } else { "light" },
        if black { "black" } else { "white" }
    )
}

#[test]
#[ignore]
fn appkit_buttons() {
    let layout_path = std::env::var("APPKIT_LAYOUT").expect("APPKIT_LAYOUT");
    let out = PathBuf::from(std::env::var("APPKIT_OUT").expect("APPKIT_OUT"));
    let positions = layout(&layout_path);
    fs::create_dir_all(&out).unwrap();
    for dark in [false, true] {
        for black in [false, true] {
            let placed = buttons()
                .into_iter()
                .filter_map(|(name, build)| positions.get(&name).map(|&(x, y, ..)| (x, y, build)))
                .collect();
            fs::write(
                out.join(file(dark, black, "")),
                render_scene(placed, dark, black, scene_content(), None),
            )
            .unwrap();
        }
    }
}

#[test]
#[ignore]
fn appkit_buttons_pressed() {
    let layout_path = std::env::var("APPKIT_LAYOUT").expect("APPKIT_LAYOUT");
    let out = PathBuf::from(std::env::var("APPKIT_OUT").expect("APPKIT_OUT"));
    let positions = layout(&layout_path);
    fs::create_dir_all(&out).unwrap();
    for (name, build) in pressed() {
        let Some(&(x, y, w, h)) = positions.get(&name) else {
            continue;
        };
        for dark in [false, true] {
            for black in [false, true] {
                fs::write(
                    out.join(file(dark, black, &format!("-pressed-{name}"))),
                    render_scene(
                        vec![(x, y, build)],
                        dark,
                        black,
                        scene_content(),
                        Some((x + w / 2.0, y + h / 2.0)),
                    ),
                )
                .unwrap();
            }
        }
    }
}

fn leading_switch<const N: usize>(on: bool) -> Element {
    Switch::new("Toggle")
        .on(on)
        .size(size::<N>())
        .style(|style: SwitchStyle| SwitchStyle {
            label_position: Some(Edge::Leading),
            label_gap: 8.0,
            ..style
        })
        .into_element()
}

fn swiftui_switch_off<const N: usize>() -> Element {
    leading_switch::<N>(false)
}

fn swiftui_switch_on<const N: usize>() -> Element {
    leading_switch::<N>(true)
}

fn toggle_checkbox<const N: usize>(on: bool) -> Element {
    Checkbox::new("Toggle")
        .checked(on)
        .size(size::<N>())
        .into_element()
}

fn swiftui_checkbox_off<const N: usize>() -> Element {
    toggle_checkbox::<N>(false)
}

fn swiftui_checkbox_on<const N: usize>() -> Element {
    toggle_checkbox::<N>(true)
}

fn toggle_button<const N: usize>(on: bool) -> Element {
    Button::new("Toggle")
        .toggled(on)
        .size(size::<N>())
        .into_element()
}

fn swiftui_toggle_button_off<const N: usize>() -> Element {
    toggle_button::<N>(false)
}

fn swiftui_toggle_button_on<const N: usize>() -> Element {
    toggle_button::<N>(true)
}

fn swiftui_menu<const N: usize>() -> Element {
    PopUpButton::new("One")
        .item("One")
        .item("Two")
        .item("Three")
        .size(size::<N>())
        .into_element()
}

fn swiftui_radio<const N: usize>() -> Element {
    RadioGroup::new("Radio")
        .option("One")
        .option("Two")
        .option("Three")
        .selected(0)
        .size(size::<N>())
        .into_element()
}

fn swiftui_builders() -> Vec<(&'static str, Builder)> {
    vec![
        ("automatic", || Button::new("Button").into_element()),
        ("plain", || {
            bezel("Button", ButtonStyle::plain).into_element()
        }),
        ("link", || bezel("Button", ButtonStyle::link).into_element()),
        ("destructive", || {
            Button::new("Delete")
                .style(ButtonStyle::destructive)
                .into_element()
        }),
        ("cancel", || Button::new("Cancel").into_element()),
        ("bordered-disabled", || {
            Button::new("Button").disabled(true).into_element()
        }),
        ("prominent-disabled", || {
            Button::new("Button")
                .style(ButtonStyle::bordered_prominent)
                .disabled(true)
                .into_element()
        }),
        ("bordered-capsule", || {
            Button::new("Button")
                .style(|style| ButtonStyle {
                    shape: Shape::Capsule,
                    ..style
                })
                .into_element()
        }),
        ("bordered-circle", || {
            Button::system_image("", Symbol::Plus)
                .style(ButtonStyle::circular)
                .into_element()
        }),
        ("bordered-rounded", || Button::new("Button").into_element()),
        ("inline", || swiftui_radio::<2>()),
        ("switch-on-disabled", || {
            Switch::new("Toggle")
                .on(true)
                .disabled(true)
                .style(|style: SwitchStyle| SwitchStyle {
                    label_position: Some(Edge::Leading),
                    label_gap: 8.0,
                    ..style
                })
                .into_element()
        }),
    ]
}

fn swiftui() -> Vec<(String, Builder)> {
    [
        each("bordered", sizes!(push)),
        each("prominent", sizes!(push_default)),
        each("switch-off", sizes!(swiftui_switch_off)),
        each("switch-on", sizes!(swiftui_switch_on)),
        each("checkbox-off", sizes!(swiftui_checkbox_off)),
        each("checkbox-on", sizes!(swiftui_checkbox_on)),
        each("togglebutton-off", sizes!(swiftui_toggle_button_off)),
        each("togglebutton-on", sizes!(swiftui_toggle_button_on)),
        each("menu", sizes!(swiftui_menu)),
        each("radio", sizes!(swiftui_radio)),
        each("slider", sizes!(slider)),
        each("stepper", sizes!(stepper)),
        named(swiftui_builders()),
    ]
    .concat()
}

#[test]
#[ignore]
fn appkit_swiftui() {
    let layout_path = std::env::var("APPKIT_LAYOUT").expect("APPKIT_LAYOUT");
    let out = PathBuf::from(std::env::var("APPKIT_OUT").expect("APPKIT_OUT"));
    let positions = layout(&layout_path);
    fs::create_dir_all(&out).unwrap();
    for dark in [false, true] {
        for black in [false, true] {
            let placed = swiftui()
                .into_iter()
                .filter_map(|(name, build)| positions.get(&name).map(|&(x, y, ..)| (x, y, build)))
                .collect();
            fs::write(
                out.join(file(dark, black, "")),
                render_scene(placed, dark, black, scene_content(), None),
            )
            .unwrap();
        }
    }
}

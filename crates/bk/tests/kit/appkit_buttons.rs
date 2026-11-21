use std::{fs, path::PathBuf};

use bk::choices::RadioGroup;
use bk::compact::Stepper;
use bk::controls::{Button, Checkbox, ControlSize, SegmentedControl, Slider, Switch};
use bk::popup_button::PopUpButton;
use freya_core::prelude::*;
use torin::prelude::Size;

use super::appkit::{Builder, layout, render_scene, scene_content, scene_tag};

const SIZES: [(&str, ControlSize); 5] = [
    ("mini", ControlSize::Mini),
    ("small", ControlSize::Small),
    ("regular", ControlSize::Regular),
    ("large", ControlSize::Large),
    ("xl", ControlSize::ExtraLarge),
];

const SEGMENTED_WIDTHS: [f32; 5] = [134.0, 146.0, 158.0, 170.0, 194.0];

const SWITCH_FRAME: (f32, f32) = (54.0, 24.0);

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
    framed(SWITCH_FRAME, Switch::new("Switch").size(size::<N>()))
}

fn switch_on<const N: usize>() -> Element {
    framed(
        SWITCH_FRAME,
        Switch::new("Switch").on(true).size(size::<N>()),
    )
}

fn switch_on_disabled<const N: usize>() -> Element {
    framed(
        SWITCH_FRAME,
        Switch::new("Switch")
            .on(true)
            .size(size::<N>())
            .disabled(true),
    )
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

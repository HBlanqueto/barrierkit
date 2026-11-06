use freya_core::prelude::*;
use torin::prelude::{Area, Direction, Size};
use vow_design::{BUTTON, HAIRLINE, Rgba, SPACE_2, SPLIT_VIEW, TEXT};

use crate::i18n::text;
use crate::interaction::use_interaction;
use crate::level::LevelSlider;
use crate::popover::show_popover;
use crate::style::{Palette, text};
use crate::theme::color;

pub fn hsb_to_rgb(hue: f32, saturation: f32, brightness: f32) -> Rgba {
    let sector = (hue.rem_euclid(1.0) * 6.0).min(5.999);
    let chroma = brightness * saturation;
    let x = chroma * (1.0 - (sector % 2.0 - 1.0).abs());
    let (r, g, b) = match sector as u32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let base = brightness - chroma;
    Rgba {
        r: r + base,
        g: g + base,
        b: b + base,
        a: 1.0,
    }
}

pub fn rgb_to_hsb(rgba: Rgba) -> (f32, f32, f32) {
    let max = rgba.r.max(rgba.g).max(rgba.b);
    let min = rgba.r.min(rgba.g).min(rgba.b);
    let chroma = max - min;
    let hue = if chroma == 0.0 {
        0.0
    } else if max == rgba.r {
        ((rgba.g - rgba.b) / chroma).rem_euclid(6.0) / 6.0
    } else if max == rgba.g {
        ((rgba.b - rgba.r) / chroma + 2.0) / 6.0
    } else {
        ((rgba.r - rgba.g) / chroma + 4.0) / 6.0
    };
    let saturation = if max == 0.0 { 0.0 } else { chroma / max };
    (hue, saturation, max)
}

pub fn hex(rgba: Rgba) -> String {
    let byte = |channel: f32| (channel.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!(
        "#{:02X}{:02X}{:02X}",
        byte(rgba.r),
        byte(rgba.g),
        byte(rgba.b)
    )
}

#[derive(PartialEq)]
pub struct ColorPicker {
    label: &'static str,
    color: Rgba,
    on_change: Option<Callback<Rgba, ()>>,
}

impl ColorPicker {
    pub fn new(label: &'static str, color: Rgba) -> Self {
        Self {
            label,
            color,
            on_change: None,
        }
    }

    pub fn on_change(mut self, callback: impl Into<Callback<Rgba, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }
}

impl Component for ColorPicker {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let mut area = use_state(Area::default);
        let shown = self.color;
        let on_change = self.on_change.clone();
        let well = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::ColorWell)
            .a11y_alt(self.label)
            .width(Size::px(BUTTON.height * 2.0))
            .height(Size::px(BUTTON.height))
            .corner_radius(BUTTON.radius)
            .background(color(shown))
            .border(
                Border::new()
                    .width(HAIRLINE)
                    .alignment(BorderAlignment::Inner)
                    .fill(color(palette.card_border)),
            )
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .on_press(move |_| {
                let on_change = on_change.clone();
                show_popover(
                    *area.peek(),
                    (SPLIT_VIEW.inspector, TEXT.line_height as f32 * 7.0),
                    move || {
                        ColorPanel {
                            initial: shown,
                            on_change: on_change.clone(),
                        }
                        .into_element()
                    },
                );
            });
        interaction.focus_ring(well, BUTTON.radius)
    }
}

#[derive(PartialEq)]
struct ColorPanel {
    initial: Rgba,
    on_change: Option<Callback<Rgba, ()>>,
}

impl Component for ColorPanel {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let initial = self.initial;
        let mut hsb = use_state(|| rgb_to_hsb(initial));
        let (hue, saturation, brightness) = *hsb.read();
        let current = hsb_to_rgb(hue, saturation, brightness);
        let on_change = self.on_change.clone();
        let mut set = move |next: (f32, f32, f32)| {
            hsb.set(next);
            if let Some(on_change) = &on_change {
                on_change.call(hsb_to_rgb(next.0, next.1, next.2));
            }
        };
        let mut set_saturation = set.clone();
        let mut set_brightness = set.clone();
        rect()
            .width(Size::fill())
            .direction(Direction::Vertical)
            .spacing(SPACE_2)
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::px(BUTTON.height))
                    .corner_radius(BUTTON.radius)
                    .background(color(current))
                    .center()
                    .child(text(
                        hex(current),
                        TEXT,
                        if brightness > 0.6 && saturation < 0.5 {
                            palette.label
                        } else {
                            palette.on_accent
                        },
                    )),
            )
            .child(text(text!("hue"), TEXT, palette.secondary_label))
            .child(
                LevelSlider::new(text!("hue"))
                    .value(hue)
                    .on_change(move |value| set((value, saturation, brightness))),
            )
            .child(text(text!("saturation"), TEXT, palette.secondary_label))
            .child(
                LevelSlider::new(text!("saturation"))
                    .value(saturation)
                    .on_change(move |value| set_saturation((hue, value, brightness))),
            )
            .child(text(text!("brightness"), TEXT, palette.secondary_label))
            .child(
                LevelSlider::new(text!("brightness"))
                    .value(brightness)
                    .on_change(move |value| set_brightness((hue, saturation, value))),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsb_round_trips_through_rgb_and_primaries_land_on_their_hue() {
        let red = hsb_to_rgb(0.0, 1.0, 1.0);
        assert_eq!((red.r, red.g, red.b), (1.0, 0.0, 0.0));
        let blue = hsb_to_rgb(2.0 / 3.0, 1.0, 1.0);
        assert!((blue.b - 1.0).abs() < 1e-5 && blue.r.abs() < 1e-5);
        let (h, s, b) = rgb_to_hsb(hsb_to_rgb(0.3, 0.6, 0.8));
        assert!((h - 0.3).abs() < 1e-4 && (s - 0.6).abs() < 1e-4 && (b - 0.8).abs() < 1e-4);
        assert_eq!(hex(hsb_to_rgb(0.0, 0.0, 1.0)), "#FFFFFF");
    }
}

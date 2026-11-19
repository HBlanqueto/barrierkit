use ctdesign::Rgba;
use freya_core::prelude::*;
use torin::prelude::{Area, Direction, Size};

use crate::controls::{dimmed, shadowed};
use crate::i18n::text;
use crate::interaction::{control_state, use_interaction};
use crate::level::LevelSlider;
use crate::popover::Popover;
use crate::style::{text, with_font};
use crate::styles::Painted;
use crate::styles::{ColorPickerStyle, LayoutDirection, Paints, Theme, resolve};
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
    style: Theme,
}

impl ColorPicker {
    pub fn new(label: &'static str, color: Rgba) -> Self {
        Self {
            label,
            color,
            on_change: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(ColorPickerStyle) -> ColorPickerStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn on_change(mut self, callback: impl Into<Callback<Rgba, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }
}

impl Component for ColorPicker {
    fn render(&self) -> impl IntoElement {
        let style: ColorPickerStyle = resolve(&self.style);
        let interaction = use_interaction();
        let paints = Paints::current(control_state(interaction, false), style.effect);
        let mut area = use_state(Area::default);
        let shown = self.color;
        let on_change = self.on_change.clone();
        let (width, height) = style.size;
        let radius = style
            .shape
            .corner_radius(width, height, LayoutDirection::LeftToRight);
        let panel_size = style.panel_size;
        let resolved = style.clone();
        let panel_style = Theme::tengen().style(move |_: ColorPickerStyle| resolved.clone());
        let well = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::ColorWell)
            .a11y_alt(self.label)
            .width(Size::px(width))
            .height(Size::px(height))
            .corner_radius(radius)
            .background(color(shown))
            .bordered(&paints, style.border.as_ref())
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .on_press(move |_| {
                open_panel(
                    shown,
                    *area.peek(),
                    on_change.clone(),
                    panel_style.clone(),
                    panel_size,
                )
            });
        interaction.focus_ring(
            dimmed(shadowed(well, &style.shadows), style.opacity),
            radius.top_left(),
        )
    }
}

fn open_panel(
    initial: Rgba,
    area: Area,
    on_change: Option<Callback<Rgba, ()>>,
    style: Theme,
    size: (f32, f32),
) {
    Popover::new(size, move || {
        ColorPanel {
            initial,
            on_change: on_change.clone(),
            style: style.clone(),
        }
        .into_element()
    })
    .show(area);
}

#[derive(PartialEq)]
pub struct ColorPanel {
    initial: Rgba,
    on_change: Option<Callback<Rgba, ()>>,
    style: Theme,
}

impl ColorPanel {
    pub fn show(initial: Rgba, area: Area, on_change: impl Into<Callback<Rgba, ()>>) {
        let style: ColorPickerStyle = resolve(&Theme::tengen());
        open_panel(
            initial,
            area,
            Some(on_change.into()),
            Theme::tengen(),
            style.panel_size,
        );
    }
}

impl Component for ColorPanel {
    fn render(&self) -> impl IntoElement {
        let style: ColorPickerStyle = resolve(&self.style);
        let paints = Paints::current(Default::default(), style.effect);
        let caption = |content: &'static str| {
            with_font(label().text(content), style.font)
                .max_lines(1)
                .color(paints.fill(&style.caption))
        };
        let (_, height) = style.size;
        let radius = style
            .shape
            .corner_radius(height, height, LayoutDirection::LeftToRight);
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
            .spacing(style.spacing)
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::px(height))
                    .corner_radius(radius)
                    .background(color(current))
                    .center()
                    .child(text(
                        hex(current),
                        style.font,
                        ctappearance::backdrop::ink_on(current),
                    )),
            )
            .child(caption(text!("hue")))
            .child(
                LevelSlider::new(text!("hue"))
                    .value(hue)
                    .on_change(move |value| set((value, saturation, brightness))),
            )
            .child(caption(text!("saturation")))
            .child(
                LevelSlider::new(text!("saturation"))
                    .value(saturation)
                    .on_change(move |value| set_saturation((hue, value, brightness))),
            )
            .child(caption(text!("brightness")))
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

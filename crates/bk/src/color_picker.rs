use ctdesign::{COLOR_PICKER, HAIRLINE, MULTICOLOR, Rgba, SPACE_3};
use freya_core::prelude::*;
use std::borrow::Cow;
use torin::prelude::{Area, Direction, Gaps, Size};

use crate::controls::{dimmed, opacity_when, shadowed};
use crate::elements::{Overrides, forward_overrides};
use crate::environment::use_disabled;
use crate::i18n::text;
use crate::interaction::{control_state, use_interaction};
use crate::level::LevelSlider;
use crate::popover::Popover;
use crate::style::{text, with_font};
use crate::styles::{ColorPickerStyle, LayoutDirection, Painted, Paints, Theme, resolve};
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
    label: Cow<'static, str>,
    color: Rgba,
    disabled: bool,
    on_change: Option<Callback<Rgba, ()>>,
    style: Theme,
    overrides: Overrides,
}

impl ColorPicker {
    pub fn new(label: impl Into<Cow<'static, str>>, color: Rgba) -> Self {
        Self {
            label: label.into(),
            color,
            disabled: false,
            on_change: None,
            style: Theme::tengen(),
            overrides: Overrides::default(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(ColorPickerStyle) -> ColorPickerStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, callback: impl Into<Callback<Rgba, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }
}

forward_overrides!(ColorPicker);

impl Component for ColorPicker {
    fn render(&self) -> impl IntoElement {
        let style: ColorPickerStyle = resolve(&self.style);
        let interaction = use_interaction();
        let disabled = use_disabled(self.disabled);
        let paints = Paints::current(control_state(interaction, disabled));
        let mut area = use_state(Area::default);
        let shown = self.color;
        let on_change = self.on_change.clone();
        let (width, height) = style.size;
        let radius = style
            .well_shape
            .corner_radius(width, height, LayoutDirection::LeftToRight);
        let shift = if crate::theme::resolved().scheme == ctappearance::ColorScheme::Light {
            -style.ring_shift
        } else {
            style.ring_shift
        };
        let moved = |channel: f32| (channel + shift).clamp(0.0, 1.0);
        let ring = Rgba {
            r: moved(shown.r),
            g: moved(shown.g),
            b: moved(shown.b),
            ..shown
        };
        let panel_size = style.panel_size;
        let resolved = style.clone();
        let panel_style = Theme::tengen().style(move |_: ColorPickerStyle| resolved.clone());
        let well = interaction
            .attach(rect(), !disabled)
            .a11y_role(AccessibilityRole::ColorWell)
            .a11y_alt(&*self.label)
            .width(Size::px(width))
            .height(Size::px(height))
            .corner_radius(radius)
            .painted(&paints, &style.bezel)
            .center()
            .child(
                rect()
                    .width(Size::px(width - 2.0 * style.inset))
                    .height(Size::px(height - 2.0 * style.inset))
                    .corner_radius(crate::style::concentric(radius, style.inset))
                    .background(color(shown))
                    .border(
                        Border::new()
                            .width(HAIRLINE)
                            .alignment(BorderAlignment::Inner)
                            .fill(color(ring)),
                    ),
            )
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .on_press(move |_| {
                if !disabled {
                    open_panel(
                        shown,
                        *area.peek(),
                        on_change.clone(),
                        panel_style.clone(),
                        panel_size,
                    )
                }
            });
        let well = interaction.focus_ring(shadowed(well, &style.shadows), radius.top_left());
        let wheel =
            style.wheel.then(|| {
                rect()
                    .margin(Gaps::new(0.0, 0.0, 0.0, COLOR_PICKER.expanded_gap))
                    .width(Size::px(COLOR_PICKER.wheel_size))
                    .height(Size::px(height))
                    .corner_radius(COLOR_PICKER.wheel_radius)
                    .painted(&paints, &style.wheel_background)
                    .center()
                    .child(
                        rect()
                            .width(Size::px(COLOR_PICKER.wheel_diameter))
                            .height(Size::px(COLOR_PICKER.wheel_diameter))
                            .corner_radius(COLOR_PICKER.wheel_diameter / 2.0)
                            .background(ConicGradient::new().stops(
                                MULTICOLOR.iter().enumerate().map(|(index, stop)| {
                                    GradientStop::new(
                                        color(*stop),
                                        index as f32 * 100.0 / (MULTICOLOR.len() - 1) as f32,
                                    )
                                }),
                            ))
                            .child(
                                rect()
                                    .expanded()
                                    .corner_radius(COLOR_PICKER.wheel_diameter / 2.0)
                                    .background(RadialGradient::new().stops([
                                        GradientStop::new(color(COLOR_PICKER.wheel_center), 0.0),
                                        GradientStop::new(
                                            color(Rgba {
                                                a: 0.0,
                                                ..COLOR_PICKER.wheel_center
                                            }),
                                            70.0,
                                        ),
                                    ])),
                            ),
                    )
            });
        let root = dimmed(
            rect()
                .direction(Direction::Horizontal)
                .child(well)
                .maybe_child(wheel),
            opacity_when(disabled, style.opacity, COLOR_PICKER.disabled_opacity),
        );
        self.overrides.apply(root)
    }
}

fn open_panel(
    initial: Rgba,
    area: Area,
    on_change: Option<Callback<Rgba, ()>>,
    style: Theme,
    size: (f32, f32),
) {
    Popover::new(
        (size.0 + 2.0 * SPACE_3, size.1 + 2.0 * SPACE_3),
        move || {
            rect()
                .padding(SPACE_3)
                .child(ColorPanel {
                    initial,
                    on_change: on_change.clone(),
                    style: style.clone(),
                    overrides: Overrides::default(),
                })
                .into_element()
        },
    )
    .show(area);
}

#[derive(PartialEq)]
pub struct ColorPanel {
    initial: Rgba,
    on_change: Option<Callback<Rgba, ()>>,
    style: Theme,
    overrides: Overrides,
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

forward_overrides!(ColorPanel);

impl Component for ColorPanel {
    fn render(&self) -> impl IntoElement {
        let style: ColorPickerStyle = resolve(&self.style);
        let paints = Paints::current(Default::default());
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
        let root = rect()
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
            );
        self.overrides.apply(root)
    }
}

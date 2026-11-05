use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Gaps, Size};
use vow_design::{BUTTON, DISPLAY, ICON_BUTTON, SEGMENTED, TEXT};

use crate::i18n::text;
use crate::interaction::use_interaction;
use crate::style::{Palette, text};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

#[derive(PartialEq)]
pub struct IconButton {
    symbol: Symbol,
    label: &'static str,
    disabled: bool,
    on_press: Option<Callback<(), ()>>,
}

impl IconButton {
    pub fn new(symbol: Symbol, label: &'static str) -> Self {
        Self {
            symbol,
            label,
            disabled: false,
            on_press: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_press(mut self, callback: impl Into<Callback<(), ()>>) -> Self {
        self.on_press = Some(callback.into());
        self
    }
}

impl Component for IconButton {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let enabled = !self.disabled;
        let on_press = self.on_press.clone();
        let ink = if enabled {
            palette.label
        } else {
            palette.tertiary_label
        };

        let button = interaction
            .attach(rect(), enabled)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label)
            .width(Size::px(ICON_BUTTON.size))
            .height(Size::px(ICON_BUTTON.size))
            .corner_radius(ICON_BUTTON.radius)
            .center()
            .maybe(enabled && interaction.hovered(), |el| {
                el.background(color(palette.control_hover))
            })
            .maybe(enabled && interaction.pressed(), |el| {
                el.scale(BUTTON.pressed_scale)
            })
            .maybe(enabled, |el| {
                el.on_press(move |_| {
                    if let Some(cb) = &on_press {
                        cb.call(());
                    }
                })
            })
            .child(symbol(self.symbol, ICON_BUTTON.icon, ink));
        interaction.focus_ring(button, ICON_BUTTON.radius)
    }
}

#[derive(PartialEq)]
pub struct Stepper {
    label: &'static str,
    value: i64,
    min: i64,
    max: i64,
    on_change: Option<Callback<i64, ()>>,
}

impl Stepper {
    pub fn new(label: &'static str, min: i64, max: i64) -> Self {
        Self {
            label,
            value: min,
            min,
            max,
            on_change: None,
        }
    }

    pub fn value(mut self, value: i64) -> Self {
        self.value = value.clamp(self.min, self.max);
        self
    }

    pub fn on_change(mut self, callback: impl Into<Callback<i64, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }
}

impl Component for Stepper {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let value = crate::controlled::use_controlled(&self.value);
        let current = *value.read();
        let (min, max) = (self.min, self.max);
        let on_change = self.on_change.clone();
        let step = move |delta: i64| {
            let next = (*value.peek() + delta).clamp(min, max);
            crate::controlled::commit(value, &on_change, next);
            if next == min || next == max {
                interaction.request_focus();
            }
        };
        let (step_down, step_up, step_key) = (step.clone(), step.clone(), step);

        let stepper = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::SpinButton)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_numeric_value(current as f64))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .corner_radius(ICON_BUTTON.radius)
            .background(color(palette.control_background))
            .on_key_down(move |event: Event<KeyboardEventData>| match event.key {
                Key::Named(NamedKey::ArrowUp) => step_key(1),
                Key::Named(NamedKey::ArrowDown) => step_key(-1),
                _ => {}
            })
            .child(
                IconButton::new(Symbol::Minus, text!("decrease"))
                    .disabled(current <= min)
                    .on_press(move |_| step_down(-1)),
            )
            .child(
                rect()
                    .min_width(Size::px(ICON_BUTTON.size))
                    .center()
                    .child(text(current.to_string(), TEXT, palette.label)),
            )
            .child(
                IconButton::new(Symbol::Plus, text!("increase"))
                    .disabled(current >= max)
                    .on_press(move |_| step_up(1)),
            );
        interaction.focus_ring(stepper, ICON_BUTTON.radius)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Face {
    Bold,
    Italic,
    Underline,
}

#[derive(PartialEq)]
pub struct ToggleGroup {
    label: &'static str,
    items: Vec<(&'static str, Face, &'static str, bool)>,
    on_toggle: Option<Callback<(usize, bool), ()>>,
}

impl ToggleGroup {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            items: Vec::new(),
            on_toggle: None,
        }
    }

    pub fn item(mut self, text: &'static str, face: Face, label: &'static str, on: bool) -> Self {
        self.items.push((text, face, label, on));
        self
    }

    pub fn on_toggle(mut self, callback: impl Into<Callback<(usize, bool), ()>>) -> Self {
        self.on_toggle = Some(callback.into());
        self
    }
}

impl Component for ToggleGroup {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let states = crate::controlled::use_controlled(
            &self.items.iter().map(|item| item.3).collect::<Vec<_>>(),
        );
        let toggles = self
            .items
            .iter()
            .enumerate()
            .map(|(index, &(text, face, label, _))| {
                let mut states = states;
                let on_toggle = self.on_toggle.clone();
                let on = states.read().get(index).copied().unwrap_or(false);
                Toggle {
                    text,
                    face,
                    label,
                    on,
                    on_press: Callback::new(move |_| {
                        let next = !on;
                        if let Some(slot) = states.write().get_mut(index) {
                            *slot = next;
                        }
                        if let Some(cb) = &on_toggle {
                            cb.call((index, next));
                        }
                    }),
                }
                .into_element()
            });

        rect()
            .a11y_role(AccessibilityRole::Toolbar)
            .a11y_alt(self.label)
            .direction(Direction::Horizontal)
            .padding(Gaps::new_all(SEGMENTED.padding))
            .spacing(SEGMENTED.gap)
            .corner_radius(SEGMENTED.radius)
            .background(color(palette.control_background))
            .children(toggles)
    }
}

#[derive(PartialEq)]
struct Toggle {
    text: &'static str,
    face: Face,
    label: &'static str,
    on: bool,
    on_press: Callback<(), ()>,
}

impl Component for Toggle {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let on = self.on;
        let on_press = self.on_press.clone();
        let ink = if on { palette.on_accent } else { palette.label };
        let face = self.face;

        let toggle = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_toggled(on.into()))
            .width(Size::px(ICON_BUTTON.size))
            .height(Size::px(SEGMENTED.segment_height))
            .corner_radius(SEGMENTED.segment_radius)
            .center()
            .maybe(on, |el| el.background(color(palette.accent)))
            .on_press(move |_| on_press.call(()))
            .child(
                paragraph()
                    .font_family(TEXT.family)
                    .font_size(TEXT.size as f32)
                    .font_weight(match face {
                        Face::Bold => DISPLAY.weight as i32,
                        Face::Italic | Face::Underline => SEGMENTED.font_weight,
                    })
                    .font_slant(match face {
                        Face::Italic => FontSlant::Italic,
                        Face::Bold | Face::Underline => FontSlant::Upright,
                    })
                    .color(color(ink))
                    .span(Span::new(self.text).text_decoration(match face {
                        Face::Underline => TextDecoration::Underline,
                        Face::Bold | Face::Italic => TextDecoration::None,
                    })),
            );
        interaction.focus_ring(toggle, SEGMENTED.segment_radius)
    }
}

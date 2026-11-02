//! Compact controls built from small square buttons: icon button, stepper and toggle group.

use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Gaps, Size};
use vow_design::{BUTTON, ICON_BUTTON, SEGMENTED, TEXT};

use crate::interaction::use_interaction;
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
        let resolved = crate::theme::resolved();
        let base = vow_appearance::palette(resolved.scheme);
        let interaction = use_interaction();
        let enabled = !self.disabled;
        let on_press = self.on_press.clone();
        let ink = if enabled {
            base.ink_strong
        } else {
            base.ink_faint
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
                el.background(color(resolved.surfaces.hover))
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
        let resolved = crate::theme::resolved();
        let base = vow_appearance::palette(resolved.scheme);
        let value = use_state(|| self.value);
        let current = *value.read();
        let (min, max) = (self.min, self.max);
        let step = |delta: i64| {
            let mut value = value;
            let on_change = self.on_change.clone();
            move |_| {
                let next = (*value.read() + delta).clamp(min, max);
                if next != *value.read() {
                    value.set(next);
                    if let Some(cb) = &on_change {
                        cb.call(next);
                    }
                }
            }
        };

        rect()
            .a11y_role(AccessibilityRole::SpinButton)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_numeric_value(current as f64))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .corner_radius(ICON_BUTTON.radius)
            .background(color(resolved.surfaces.control))
            .child(
                IconButton::new(Symbol::Minus, "Menos")
                    .disabled(current <= min)
                    .on_press(step(-1)),
            )
            .child(
                rect().min_width(Size::px(ICON_BUTTON.size)).center().child(
                    label()
                        .text(current.to_string())
                        .font_family(TEXT.family)
                        .font_size(TEXT.size as f32)
                        .font_weight(TEXT.weight as i32)
                        .color(color(base.ink_strong)),
                ),
            )
            .child(
                IconButton::new(Symbol::Plus, "Más")
                    .disabled(current >= max)
                    .on_press(step(1)),
            )
    }
}

#[derive(PartialEq)]
pub struct ToggleGroup {
    label: &'static str,
    items: Vec<(&'static str, &'static str, bool)>,
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

    pub fn item(mut self, text: &'static str, label: &'static str, on: bool) -> Self {
        self.items.push((text, label, on));
        self
    }

    pub fn on_toggle(mut self, callback: impl Into<Callback<(usize, bool), ()>>) -> Self {
        self.on_toggle = Some(callback.into());
        self
    }
}

impl Component for ToggleGroup {
    fn render(&self) -> impl IntoElement {
        let resolved = crate::theme::resolved();
        let states = use_state(|| self.items.iter().map(|item| item.2).collect::<Vec<_>>());
        let toggles = self
            .items
            .iter()
            .enumerate()
            .map(|(index, (text, label, _))| {
                let mut states = states;
                let on_toggle = self.on_toggle.clone();
                let on = states.read().get(index).copied().unwrap_or(false);
                Toggle {
                    text,
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
            .background(color(resolved.surfaces.control))
            .children(toggles)
    }
}

#[derive(PartialEq)]
struct Toggle {
    text: &'static str,
    label: &'static str,
    on: bool,
    on_press: Callback<(), ()>,
}

impl Component for Toggle {
    fn render(&self) -> impl IntoElement {
        let resolved = crate::theme::resolved();
        let base = vow_appearance::palette(resolved.scheme);
        let interaction = use_interaction();
        let on = self.on;
        let on_press = self.on_press.clone();
        let ink = if on { base.on_accent } else { base.ink_strong };

        let toggle = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_toggled(on.into()))
            .width(Size::px(ICON_BUTTON.size))
            .height(Size::px(SEGMENTED.segment_height))
            .corner_radius(SEGMENTED.segment_radius)
            .center()
            .maybe(on, |el| el.background(color(resolved.accent)))
            .on_press(move |_| on_press.call(()))
            .child(
                label()
                    .text(self.text)
                    .font_family(TEXT.family)
                    .font_size(TEXT.size as f32)
                    .font_weight(SEGMENTED.font_weight)
                    .color(color(ink)),
            );
        interaction.focus_ring(toggle, SEGMENTED.segment_radius)
    }
}

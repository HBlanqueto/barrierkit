use crate::i18n::text;

use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Position, Size};
use vow_design::{CAPTION, Rgba, SETTINGS, SWITCH, TEXT};

use crate::backdrop::Tint;
use crate::interaction::{ring, use_interaction};
use crate::style::{Palette, text};
use crate::theme::color;

pub(crate) struct Choice {
    selected: State<Option<usize>>,
    on_change: Option<Callback<usize, ()>>,
}

pub(crate) fn use_choice(initial: Option<usize>, on_change: Option<Callback<usize, ()>>) -> Choice {
    Choice {
        selected: crate::controlled::use_controlled(&initial),
        on_change,
    }
}

impl Choice {
    pub(crate) fn is(&self, index: usize) -> bool {
        *self.selected.read() == Some(index)
    }

    pub(crate) fn select(&self, index: usize) -> Callback<(), ()> {
        let mut selected = self.selected;
        let on_change = self.on_change.clone();
        Callback::new(move |_| {
            if *selected.read() == Some(index) {
                return;
            }
            selected.set(Some(index));
            if let Some(cb) = &on_change {
                cb.call(index);
            }
        })
    }
}

fn choice_group(label: &'static str, spacing: f32, options: impl Iterator<Item = Element>) -> Rect {
    rect()
        .a11y_role(AccessibilityRole::RadioGroup)
        .a11y_alt(label)
        .direction(Direction::Horizontal)
        .spacing(spacing)
        .children(options)
}

#[derive(PartialEq)]
pub struct RadioGroup {
    label: &'static str,
    options: Vec<&'static str>,
    selected: usize,
    disabled: bool,
    on_change: Option<Callback<usize, ()>>,
    tint: Tint,
    key: DiffKey,
}

impl RadioGroup {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            options: Vec::new(),
            selected: 0,
            disabled: false,
            on_change: None,
            tint: Tint::Accent,
            key: DiffKey::None,
        }
    }

    pub fn tint(mut self, tint: Tint) -> Self {
        self.tint = tint;
        self
    }

    pub fn option(mut self, label: &'static str) -> Self {
        self.options.push(label);
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, callback: impl Into<Callback<usize, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for RadioGroup {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for RadioGroup {
    fn render(&self) -> impl IntoElement {
        let choice = use_choice(Some(self.selected), self.on_change.clone());
        let options = self.options.iter().enumerate().map(|(index, text)| {
            RadioOption {
                text,
                selected: choice.is(index),
                disabled: self.disabled,
                on_select: choice.select(index),
                tint: self.tint,
            }
            .into_element()
        });
        choice_group(self.label, SETTINGS.radio_spacing as f32, options)
    }
}

#[derive(PartialEq)]
struct RadioOption {
    text: &'static str,
    selected: bool,
    disabled: bool,
    on_select: Callback<(), ()>,
    tint: Tint,
}

impl Component for RadioOption {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let size = SETTINGS.radio_size as f32;
        let dot = SETTINGS.radio_dot as f32;
        let selected = self.selected;
        let enabled = !self.disabled;
        let on_select = self.on_select.clone();

        let mut circle = rect()
            .width(Size::px(size))
            .height(Size::px(size))
            .corner_radius(size / 2.0)
            .center();
        circle = if selected {
            circle.background(self.tint.fill(&palette)).child(
                rect()
                    .width(Size::px(dot))
                    .height(Size::px(dot))
                    .corner_radius(dot / 2.0)
                    .background(color(palette.on_accent)),
            )
        } else {
            circle.background(color(palette.content_background)).border(
                Border::new()
                    .width(vow_design::HAIRLINE)
                    .alignment(BorderAlignment::Inner)
                    .fill(color(palette.control_border)),
            )
        };
        circle = interaction.focus_ring(circle, size / 2.0);

        interaction
            .attach(rect(), enabled)
            .a11y_role(AccessibilityRole::RadioButton)
            .a11y_alt(self.text)
            .a11y_builder(move |node| node.set_toggled(selected.into()))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(SETTINGS.radio_gap as f32)
            .maybe(enabled, |el| el.on_press(move |_| on_select.call(())))
            .child(circle)
            .child(text(
                self.text,
                TEXT,
                if enabled {
                    palette.label
                } else {
                    palette.tertiary_label
                },
            ))
    }
}

fn swatch_name(id: &'static str) -> &'static str {
    match id {
        "blue" => text!("accent-blue"),
        "purple" => text!("accent-purple"),
        "pink" => text!("accent-pink"),
        "red" => text!("accent-red"),
        "orange" => text!("accent-orange"),
        "yellow" => text!("accent-yellow"),
        "green" => text!("accent-green"),
        "graphite" => text!("accent-graphite"),
        _ => id,
    }
}

#[derive(PartialEq)]
pub struct ColorWell {
    label: &'static str,
    swatches: &'static [(&'static str, Rgba)],
    selected: Option<usize>,
    disabled: bool,
    on_change: Option<Callback<usize, ()>>,
    key: DiffKey,
}

impl ColorWell {
    pub fn new(label: &'static str, swatches: &'static [(&'static str, Rgba)]) -> Self {
        Self {
            label,
            swatches,
            selected: None,
            disabled: false,
            on_change: None,
            key: DiffKey::None,
        }
    }

    pub fn selected(mut self, index: Option<usize>) -> Self {
        self.selected = index;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, callback: impl Into<Callback<usize, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for ColorWell {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for ColorWell {
    fn render(&self) -> impl IntoElement {
        let choice = use_choice(self.selected, self.on_change.clone());
        let swatches = self
            .swatches
            .iter()
            .enumerate()
            .map(|(index, (name, fill))| {
                Swatch {
                    name: swatch_name(name),
                    fill: *fill,
                    selected: choice.is(index),
                    disabled: self.disabled,
                    on_select: choice.select(index),
                }
                .into_element()
            });
        choice_group(self.label, SETTINGS.swatch_gap as f32, swatches)
    }
}

#[derive(PartialEq)]
struct Swatch {
    name: &'static str,
    fill: Rgba,
    selected: bool,
    disabled: bool,
    on_select: Callback<(), ()>,
}

impl Component for Swatch {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let size = SETTINGS.swatch_size as f32;
        let selected = self.selected;
        let enabled = !self.disabled;
        let on_select = self.on_select.clone();

        let mut dot = rect()
            .width(Size::px(size))
            .height(Size::px(size))
            .corner_radius(size / 2.0)
            .background(color(self.fill))
            .border(
                Border::new()
                    .width(vow_design::HAIRLINE)
                    .alignment(BorderAlignment::Inner)
                    .fill(color(palette.separator)),
            );
        if selected {
            dot = ring(dot, size / 2.0, palette.label);
        }
        dot = interaction.focus_ring(dot, size / 2.0);

        interaction
            .attach(rect(), enabled)
            .a11y_role(AccessibilityRole::RadioButton)
            .a11y_alt(self.name)
            .a11y_builder(move |node| node.set_toggled(selected.into()))
            .width(Size::px(SETTINGS.swatch_column as f32))
            .direction(Direction::Vertical)
            .cross_align(Alignment::Center)
            .spacing(SETTINGS.swatch_name_gap as f32)
            .maybe(!enabled, |el| el.opacity(SWITCH.disabled_opacity))
            .maybe(enabled, |el| el.on_press(move |_| on_select.call(())))
            .child(dot)
            .child(text(
                if selected { self.name } else { "" },
                CAPTION,
                palette.secondary_label,
            ))
    }
}

#[derive(PartialEq)]
pub struct ChoiceCards {
    label: &'static str,
    options: Vec<(&'static str, Element)>,
    selected: usize,
    disabled: bool,
    on_change: Option<Callback<usize, ()>>,
    key: DiffKey,
}

impl ChoiceCards {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            options: Vec::new(),
            selected: 0,
            disabled: false,
            on_change: None,
            key: DiffKey::None,
        }
    }

    pub fn option(mut self, caption: &'static str, preview: impl IntoElement) -> Self {
        self.options.push((caption, preview.into_element()));
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, callback: impl Into<Callback<usize, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for ChoiceCards {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for ChoiceCards {
    fn render(&self) -> impl IntoElement {
        let choice = use_choice(Some(self.selected), self.on_change.clone());
        let cards = self
            .options
            .iter()
            .enumerate()
            .map(|(index, (caption, preview))| {
                Card {
                    caption,
                    preview: preview.clone(),
                    selected: choice.is(index),
                    disabled: self.disabled,
                    on_select: choice.select(index),
                }
                .into_element()
            });
        choice_group(self.label, SETTINGS.card_gap as f32, cards)
    }
}

#[derive(PartialEq)]
struct Card {
    caption: &'static str,
    preview: Element,
    selected: bool,
    disabled: bool,
    on_select: Callback<(), ()>,
}

impl Component for Card {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let radius = SETTINGS.card_radius as f32;
        let selected = self.selected;
        let enabled = !self.disabled;
        let on_select = self.on_select.clone();

        let mut card = rect().corner_radius(radius).child(
            rect()
                .width(Size::px(SETTINGS.card_width as f32))
                .height(Size::px(SETTINGS.card_height as f32))
                .corner_radius(radius)
                .overflow(Overflow::Clip)
                .child(self.preview.clone())
                .child(
                    rect()
                        .position(Position::new_absolute().left(0.0).top(0.0))
                        .width(Size::fill())
                        .height(Size::percent(100.0))
                        .corner_radius(radius)
                        .border(
                            Border::new()
                                .width(vow_design::HAIRLINE)
                                .alignment(BorderAlignment::Inner)
                                .fill(color(SETTINGS.card_outline)),
                        ),
                ),
        );
        if selected {
            card = ring(card, radius, palette.accent);
        }
        card = interaction.focus_ring(card, radius);

        let caption = SETTINGS.card_caption_font;
        interaction
            .attach(rect(), enabled)
            .a11y_role(AccessibilityRole::RadioButton)
            .a11y_alt(self.caption)
            .a11y_builder(move |node| node.set_toggled(selected.into()))
            .direction(Direction::Vertical)
            .cross_align(Alignment::Center)
            .spacing(SETTINGS.card_caption_gap as f32)
            .maybe(!enabled, |el| el.opacity(SWITCH.disabled_opacity))
            .maybe(enabled, |el| el.on_press(move |_| on_select.call(())))
            .child(card)
            .child(
                text(
                    self.caption,
                    caption,
                    if selected {
                        palette.label
                    } else {
                        palette.secondary_label
                    },
                )
                .font_weight(if selected {
                    FontWeight::SEMI_BOLD
                } else {
                    FontWeight::from(caption.weight as i32)
                }),
            )
    }
}

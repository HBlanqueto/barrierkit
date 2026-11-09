use crate::i18n::text;

use ctdesign::{MULTICOLOR, Rgba};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Position, Size};

use crate::backdrop::Tint;
use crate::controls::{dimmed, dimmed_label, inner_border, labeled, shadowed};
use crate::interaction::{control_state, ring, use_interaction};
use crate::style::with_font;
use crate::styles::Painted;
use crate::styles::{
    Axis, ChoiceCardsStyle, ColorWellStyle, LayoutDirection, Paints, RadioGroupStyle, StateValue,
    Theme, resolve,
};
use crate::theme::color;

#[derive(Clone)]
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

    // WAI-ARIA APG radio group and tabs: the arrows move the selection, wrapping at the ends, and
    // the newly chosen option takes the focus.
    pub(crate) fn arrow_keys(&self, group: Rect, count: usize, mut keyed: State<bool>) -> Rect {
        let choice = self.clone();
        group.on_key_down(move |event: Event<KeyboardEventData>| {
            let delta = match event.key {
                Key::Named(NamedKey::ArrowLeft | NamedKey::ArrowUp) => -1,
                Key::Named(NamedKey::ArrowRight | NamedKey::ArrowDown) => 1,
                _ => return,
            };
            let current = (*choice.selected.peek()).unwrap_or(0);
            let next = (current as isize + delta).rem_euclid(count.max(1) as isize) as usize;
            keyed.set(true);
            choice.select(next).call(());
        })
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

pub(crate) fn take_focus_when(interaction: crate::interaction::Interaction, take_focus: bool) {
    let take_focus = use_reactive(&take_focus);
    use_side_effect(move || {
        if *take_focus.read() {
            interaction.request_focus();
        }
    });
}

fn choice_group(
    label: &'static str,
    axis: Axis,
    spacing: f32,
    wraps: bool,
    options: impl Iterator<Item = Element>,
) -> Rect {
    rect()
        .a11y_role(AccessibilityRole::RadioGroup)
        .a11y_alt(label)
        .direction(match axis {
            Axis::Horizontal => Direction::Horizontal,
            Axis::Vertical => Direction::Vertical,
        })
        .spacing(spacing)
        .maybe(wraps, |el| el.content(Content::wrap_spacing(spacing)))
        .children(options)
}

fn dimmed_when(element: Rect, disabled: bool, opacity: f32, disabled_opacity: f32) -> Rect {
    dimmed(
        element,
        if disabled {
            opacity * disabled_opacity
        } else {
            opacity
        },
    )
}

#[derive(PartialEq)]
pub struct RadioGroup {
    label: &'static str,
    options: Vec<&'static str>,
    selected: usize,
    disabled: bool,
    on_change: Option<Callback<usize, ()>>,
    style: Theme,
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
            style: Theme::tengen(),
            key: DiffKey::None,
        }
    }

    pub fn tint(self, tint: Tint) -> Self {
        self.style(move |style| RadioGroupStyle {
            fill: StateValue::new(tint.into()),
            ..style
        })
    }

    pub fn style(mut self, restyle: impl Fn(RadioGroupStyle) -> RadioGroupStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
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
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for RadioGroup {
    fn render(&self) -> impl IntoElement {
        let style: RadioGroupStyle = resolve(&self.style);
        let choice = use_choice(Some(self.selected), self.on_change.clone());
        let keyed = use_state(|| false);
        let options = self.options.iter().enumerate().map(|(index, text)| {
            RadioOption {
                text,
                selected: choice.is(index),
                take_focus: choice.is(index) && *keyed.read(),
                disabled: self.disabled,
                on_select: choice.select(index),
                style: self.style.clone(),
            }
            .into_element()
        });
        let group = choice_group(self.label, style.axis, style.spacing, false, options);
        if self.disabled {
            group
        } else {
            choice.arrow_keys(group, self.options.len(), keyed)
        }
    }
}

#[derive(PartialEq)]
struct RadioOption {
    text: &'static str,
    selected: bool,
    take_focus: bool,
    disabled: bool,
    on_select: Callback<(), ()>,
    style: Theme,
}

impl Component for RadioOption {
    fn render(&self) -> impl IntoElement {
        let style: RadioGroupStyle = resolve(&self.style);
        let interaction = use_interaction();
        take_focus_when(interaction, self.take_focus);
        let paints = Paints::current(control_state(interaction, self.disabled), style.effect);
        let direction = style.layout_direction;
        let size = style.button_size;
        let dot = style.dot_size;
        let selected = self.selected;
        let enabled = !self.disabled;
        let on_select = self.on_select.clone();
        let radius = style.button_shape.corner_radius(size, size, direction);
        let paint = if selected {
            &style.fill
        } else {
            &style.button_background
        };

        let circle = rect()
            .width(Size::px(size))
            .height(Size::px(size))
            .corner_radius(radius)
            .center()
            .painted(&paints, paint)
            .map(
                style.button_border.as_ref().filter(|_| !selected),
                |el, border| el.border(inner_border(&paints, border)),
            )
            .maybe(selected, |el| {
                el.child(
                    rect()
                        .width(Size::px(dot))
                        .height(Size::px(dot))
                        .corner_radius(style.dot_shape.corner_radius(dot, dot, direction))
                        .painted(&paints, &style.dot),
                )
            });
        let opacity = if enabled {
            style.opacity
        } else {
            style.opacity * style.disabled_opacity
        };
        let circle = interaction.focus_ring(
            dimmed(shadowed(circle, &style.button_shadows), opacity),
            radius.top_left(),
        );

        let row = interaction
            .attach(rect(), enabled)
            .a11y_role(AccessibilityRole::RadioButton)
            .a11y_alt(self.text)
            .a11y_builder(move |node| node.set_toggled(selected.into()))
            .maybe(enabled, |el| el.on_press(move |_| on_select.call(())));
        match style.label_position {
            None => row
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center)
                .child(circle),
            Some(edge) => {
                let text = with_font(label().text(self.text), style.label_font)
                    .max_lines(1)
                    .color(paints.fill(&style.label));
                labeled(
                    row,
                    circle,
                    dimmed_label(text, opacity),
                    edge,
                    direction,
                    style.label_gap,
                )
            }
        }
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
    multicolor: Option<&'static str>,
    swatches: &'static [(&'static str, Rgba)],
    selected: Option<usize>,
    disabled: bool,
    on_change: Option<Callback<usize, ()>>,
    style: Theme,
    key: DiffKey,
}

impl ColorWell {
    pub fn new(label: &'static str, swatches: &'static [(&'static str, Rgba)]) -> Self {
        Self {
            label,
            multicolor: None,
            swatches,
            selected: None,
            disabled: false,
            on_change: None,
            style: Theme::tengen(),
            key: DiffKey::None,
        }
    }

    pub fn style(mut self, restyle: impl Fn(ColorWellStyle) -> ColorWellStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn multicolor(mut self, name: &'static str) -> Self {
        self.multicolor = Some(name);
        self
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
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for ColorWell {
    fn render(&self) -> impl IntoElement {
        let style: ColorWellStyle = resolve(&self.style);
        let choice = use_choice(self.selected, self.on_change.clone());
        let swatches = self
            .multicolor
            .map(|name| (name, None))
            .into_iter()
            .chain(
                self.swatches
                    .iter()
                    .map(|(name, fill)| (swatch_name(name), Some(*fill))),
            )
            .enumerate()
            .map(|(index, (name, fill))| {
                Swatch {
                    name,
                    fill,
                    selected: choice.is(index),
                    disabled: self.disabled,
                    on_select: choice.select(index),
                    style: self.style.clone(),
                }
                .into_element()
            });
        choice_group(self.label, style.axis, style.spacing, style.wraps, swatches)
    }
}

#[derive(PartialEq)]
struct Swatch {
    name: &'static str,
    fill: Option<Rgba>,
    selected: bool,
    disabled: bool,
    on_select: Callback<(), ()>,
    style: Theme,
}

impl Component for Swatch {
    fn render(&self) -> impl IntoElement {
        let style: ColorWellStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, self.disabled);
        let paints = Paints::current(state, style.effect);
        let size = style.swatch_size;
        let radius = style
            .swatch_shape
            .corner_radius(size, size, LayoutDirection::LeftToRight);
        let selected = self.selected;
        let enabled = !self.disabled;
        let on_select = self.on_select.clone();

        let mut dot =
            shadowed(
                rect()
                    .width(Size::px(size))
                    .height(Size::px(size))
                    .corner_radius(radius)
                    .map(self.fill, |el, fill| el.background(color(fill)))
                    .maybe(self.fill.is_none(), |el| {
                        el.background(ConicGradient::new().stops(
                            MULTICOLOR.iter().enumerate().map(|(index, stop)| {
                                GradientStop::new(
                                    color(*stop),
                                    index as f32 * 100.0 / (MULTICOLOR.len() - 1) as f32,
                                )
                            }),
                        ))
                    })
                    .bordered(&paints, style.swatch_border.as_ref()),
                &style.swatch_shadows,
            );
        if selected {
            dot = ring(
                dot,
                radius.top_left(),
                paints.color(style.selection.resolve(state)),
            );
        }
        dot = interaction.focus_ring(dot, radius.top_left());

        let column = interaction
            .attach(rect(), enabled)
            .a11y_role(AccessibilityRole::RadioButton)
            .a11y_alt(self.name)
            .a11y_builder(move |node| node.set_toggled(selected.into()))
            .width(Size::px(style.column_width))
            .direction(Direction::Vertical)
            .cross_align(Alignment::Center)
            .spacing(style.name_gap)
            .maybe(enabled, |el| el.on_press(move |_| on_select.call(())))
            .child(dot)
            .child(
                with_font(
                    label().text(if selected { self.name } else { "" }),
                    style.name_font,
                )
                .max_lines(1)
                .color(paints.fill(&style.name)),
            );
        dimmed_when(column, self.disabled, style.opacity, style.disabled_opacity)
    }
}

#[derive(PartialEq)]
pub struct ChoiceCards {
    label: &'static str,
    options: Vec<(&'static str, Element)>,
    selected: usize,
    disabled: bool,
    on_change: Option<Callback<usize, ()>>,
    style: Theme,
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
            style: Theme::tengen(),
            key: DiffKey::None,
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(ChoiceCardsStyle) -> ChoiceCardsStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
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
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for ChoiceCards {
    fn render(&self) -> impl IntoElement {
        let style: ChoiceCardsStyle = resolve(&self.style);
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
                    style: self.style.clone(),
                }
                .into_element()
            });
        choice_group(self.label, style.axis, style.spacing, style.wraps, cards)
    }
}

#[derive(PartialEq)]
struct Card {
    caption: &'static str,
    preview: Element,
    selected: bool,
    disabled: bool,
    on_select: Callback<(), ()>,
    style: Theme,
}

impl Component for Card {
    fn render(&self) -> impl IntoElement {
        let style: ChoiceCardsStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, self.disabled);
        let paints = Paints::current(state, style.effect);
        let (width, height) = style.card_size;
        let radius = style
            .card_shape
            .corner_radius(width, height, LayoutDirection::LeftToRight);
        let selected = self.selected;
        let enabled = !self.disabled;
        let on_select = self.on_select.clone();

        let outline = style.card_border.as_ref().map(|border| {
            rect()
                .position(Position::new_absolute().left(0.0).top(0.0))
                .width(Size::fill())
                .height(Size::percent(100.0))
                .corner_radius(radius)
                .border(inner_border(&paints, border))
        });
        let mut card = shadowed(
            rect().corner_radius(radius).child(
                rect()
                    .width(Size::px(width))
                    .height(Size::px(height))
                    .corner_radius(radius)
                    .overflow(Overflow::Clip)
                    .child(self.preview.clone())
                    .map(outline, |el, outline| el.child(outline)),
            ),
            &style.card_shadows,
        );
        if selected {
            card = ring(
                card,
                radius.top_left(),
                paints.color(style.selection.resolve(state)),
            );
        }
        card = interaction.focus_ring(card, radius.top_left());

        let (font, ink) = if selected {
            (style.selected_caption_font, &style.selected_caption)
        } else {
            (style.caption_font, &style.caption)
        };
        let column = interaction
            .attach(rect(), enabled)
            .a11y_role(AccessibilityRole::RadioButton)
            .a11y_alt(self.caption)
            .a11y_builder(move |node| node.set_toggled(selected.into()))
            .direction(Direction::Vertical)
            .cross_align(Alignment::Center)
            .spacing(style.caption_gap)
            .maybe(enabled, |el| el.on_press(move |_| on_select.call(())))
            .child(card)
            .child(
                with_font(label().text(self.caption), font)
                    .max_lines(1)
                    .color(paints.fill(ink)),
            );
        dimmed_when(column, self.disabled, style.opacity, style.disabled_opacity)
    }
}

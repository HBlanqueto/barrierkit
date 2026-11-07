pub use crate::scroll::{ScrollController, ScrollView, use_scroll_controller};

use freya_components::input::{
    Input, InputColorsThemePartial, InputLayoutThemePartial, InputValidator,
};
use freya_components::theming::macros::Preference;

use ctdesign::{
    BODY, BUTTON, CAPTION, CHECKBOX, Rgba, SEGMENTED, SETTINGS, SLIDER, SWITCH, TEXT_FIELD,
    TEXT_LINE_HEIGHT, TOOLGROUP,
};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};

use crate::ProbeExt;
use crate::backdrop::Tint;
use crate::choices::use_choice;
use crate::context_menu::{Spelling, edit_menu};
use crate::controlled::commit;
use crate::i18n::text;
use crate::interaction::use_interaction;
use crate::level::{self, bar, knob_shadow};
use crate::password::PasswordField;
use crate::spelling::{Squiggles, char_index, dictionary, known, misspelled, utf16_range};
use crate::style::Palette;
use crate::symbol::{Symbol, symbol};
use crate::theme;
use crate::theme::color;
use zeroize::Zeroizing;

#[derive(Clone, PartialEq)]
pub struct SecureField {
    field: State<PasswordField>,
    label: &'static str,
    width: Size,
    auto_focus: bool,
    on_submit: Option<Callback<Zeroizing<String>, ()>>,
    key: DiffKey,
}

impl SecureField {
    pub fn new(field: State<PasswordField>, label: &'static str) -> Self {
        Self {
            field,
            label,
            width: Size::fill(),
            auto_focus: false,
            on_submit: None,
            key: DiffKey::None,
        }
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }

    pub fn auto_focus(mut self, auto_focus: bool) -> Self {
        self.auto_focus = auto_focus;
        self
    }

    pub fn on_submit(mut self, callback: impl Into<Callback<Zeroizing<String>, ()>>) -> Self {
        self.on_submit = Some(callback.into());
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for SecureField {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for SecureField {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let mut field = self.field;
        let on_submit = self.on_submit.clone();
        let masked = field.read().masked();
        let margin_y = (TEXT_FIELD.height - BODY.line_height as f32) / 2.0;
        let element = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::PasswordInput)
            .a11y_alt(self.label)
            .a11y_auto_focus(self.auto_focus)
            .width(self.width.clone())
            .height(Size::px(TEXT_FIELD.height))
            .padding(Gaps::new(
                margin_y,
                TEXT_FIELD.padding_x,
                margin_y,
                TEXT_FIELD.padding_x,
            ))
            .corner_radius(TEXT_FIELD.radius)
            .background(color(palette.content_background))
            .border(
                Border::new()
                    .width(ctdesign::HAIRLINE)
                    .alignment(BorderAlignment::Inner)
                    .fill(color(palette.separator)),
            )
            .on_key_down(move |event: Event<KeyboardEventData>| {
                let secret = field.write().handle_key(&event.key, event.modifiers);
                if let Some(secret) = secret {
                    event.prevent_default();
                    if let Some(callback) = &on_submit {
                        callback.call(secret);
                    }
                }
            })
            .child(crate::style::text(masked, BODY, palette.label));
        interaction.focus_ring(element, TEXT_FIELD.radius)
    }
}

#[derive(PartialEq)]
pub struct ToolGroup {
    children: Vec<Element>,
    probe: Option<&'static str>,
    key: DiffKey,
}

impl Default for ToolGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolGroup {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            probe: None,
            key: DiffKey::None,
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(IntoElement::into_element));
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for ToolGroup {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl ProbeExt for ToolGroup {
    fn probe(mut self, id: &'static str) -> Self {
        self.probe = Some(id);
        self
    }
}

impl Component for ToolGroup {
    fn render(&self) -> impl IntoElement {
        crate::compact::use_toolbar_items(true);
        let resolved = crate::theme::resolved();
        let island = resolved.sidebar.style == ctappearance::SidebarStyle::Island;
        let (bg, separator, rim) = if crate::backdrop::tinted().is_some() {
            let palette = Palette::current();
            (
                palette.control_background,
                palette.separator,
                palette.card_border,
            )
        } else {
            let palette = resolved.toolgroup.palette;
            (resolved.toolgroup.bg, palette.separator, palette.rim)
        };

        let mut items: Vec<Element> = Vec::with_capacity(self.children.len());
        let last = self.children.len().saturating_sub(1);
        let slot = TOOLGROUP.height as f32;
        for (i, child) in self.children.iter().cloned().enumerate() {
            items.push(
                rect()
                    .width(Size::px(slot))
                    .height(Size::px(slot))
                    .center()
                    .child(child)
                    .into_element(),
            );
            if island && i != last {
                items.push(
                    rect()
                        .width(Size::px(TOOLGROUP.separator_w as f32))
                        .height(Size::px(TOOLGROUP.separator_h as f32))
                        .background(theme::color(separator))
                        .into_element(),
                );
            }
        }

        let mut root = rect()
            .direction(Direction::Horizontal)
            .height(Size::px(TOOLGROUP.height as f32))
            .cross_align(Alignment::Center)
            .padding(Gaps::new(
                0.0,
                TOOLGROUP.padding_x as f32,
                0.0,
                TOOLGROUP.padding_x as f32,
            ))
            .children(items);
        root = root.map(self.probe, ProbeExt::probe);

        if island {
            return root
                .background(theme::color(bg))
                .border(
                    Border::new()
                        .width(ctdesign::HAIRLINE)
                        .alignment(BorderAlignment::Inner)
                        .fill(theme::color(rim)),
                )
                .corner_radius((TOOLGROUP.height as f32 / 2.0).max(1.0))
                .into_element();
        }

        root.into_element()
    }
}

#[derive(PartialEq)]
pub struct Slider {
    label: &'static str,
    steps: usize,
    value: usize,
    labels: &'static [&'static str],
    disabled: bool,
    on_change: Option<Callback<usize, ()>>,
    probe: Option<&'static str>,
    tint: Tint,
    key: DiffKey,
}

impl Slider {
    pub fn stepped(label: &'static str, steps: usize) -> Self {
        Self {
            label,
            steps: steps.max(2),
            value: 0,
            labels: &[],
            disabled: false,
            on_change: None,
            probe: None,
            tint: Tint::Accent,
            key: DiffKey::None,
        }
    }

    pub fn tint(mut self, tint: Tint) -> Self {
        self.tint = tint;
        self
    }

    pub fn value(mut self, value: usize) -> Self {
        self.value = value.min(self.steps.saturating_sub(1));
        self
    }

    pub fn labels(mut self, labels: &'static [&'static str]) -> Self {
        self.labels = labels;
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

impl KeyExt for Slider {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl ProbeExt for Slider {
    fn probe(mut self, id: &'static str) -> Self {
        self.probe = Some(id);
        self
    }
}

impl Component for Slider {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let value = crate::controlled::use_controlled(&self.value);
        let left = use_state(|| 0.0f64);
        let on_change = self.on_change.clone();

        let width = SETTINGS.strength.width as f32;
        let (knob, knob_height) = (SLIDER.knob_width, SLIDER.knob_height);
        let track = SETTINGS.strength.track as f32;
        let tick = SETTINGS.strength.tick as f32;
        let tick_top = (knob_height + track) / 2.0 + SETTINGS.strength.tick_gap as f32;
        let label_gap = SETTINGS.strength.label_gap as f32;
        let steps = self.steps;
        let segments = steps.saturating_sub(1).max(1) as f32;
        let run = width - knob;
        let fraction = *value.read() as f32 / segments;
        let knob_x = run * fraction;

        let set_from_x = move |x: f64| {
            let x = (x as f32 - knob / 2.0).clamp(0.0, run);
            let step = ((x / run) * segments).round() as usize;
            step.min(steps - 1)
        };

        let update = move |new_value: usize| commit(value, &on_change, new_value);

        let current = *value.read();
        let step_name = self.labels.get(current).copied();
        let mut root = interaction
            .attach(rect(), !self.disabled)
            .width(Size::px(width))
            .height(Size::px(
                knob_height + label_gap + CAPTION.size as f32 * TEXT_LINE_HEIGHT,
            ))
            .a11y_role(AccessibilityRole::Slider)
            .a11y_alt(self.label)
            .a11y_builder(move |node| {
                node.set_numeric_value(current as f64);
                node.set_min_numeric_value(0.0);
                node.set_max_numeric_value((steps - 1) as f64);
                node.set_numeric_value_step(1.0);
                if let Some(name) = step_name {
                    node.set_value(name);
                }
            });

        if !self.disabled {
            root = level::on_drag(root, interaction, left, set_from_x, update.clone()).on_key_down(
                move |event: Event<KeyboardEventData>| match event.key {
                    Key::Named(NamedKey::ArrowLeft | NamedKey::ArrowDown) => {
                        update(current.saturating_sub(1))
                    }
                    Key::Named(NamedKey::ArrowRight | NamedKey::ArrowUp) => {
                        update((current + 1).min(steps - 1))
                    }
                    _ => {}
                },
            );
        }

        let disabled = self.disabled;
        let dim = move |el: Rect| el.maybe(disabled, |el| el.opacity(SWITCH.disabled_opacity));
        let track_rect = dim(bar(
            knob / 2.0,
            (knob_height - track) / 2.0,
            run,
            track,
            color(palette.control_background),
        ));
        let fill_rect = bar(
            -knob / 2.0,
            0.0,
            knob_x + knob / 2.0,
            track,
            self.tint.fill(&palette),
        );

        let ticks = (0..self.steps).map(|i| {
            let x = knob / 2.0 + run * (i as f32 / segments);
            dim(rect()
                .position(Position::new_absolute().left(x - tick / 2.0).top(tick_top))
                .width(Size::px(tick))
                .height(Size::px(tick))
                .corner_radius(tick / 2.0)
                .background(color(palette.control_border)))
            .into_element()
        });
        let knob_rect = level::knob(knob_x, 0.0, knob, knob_height).layer(level::KNOB_ABOVE_FILL);

        let labels = self.labels.iter().take(self.steps).map(|text| {
            let label_node = label()
                .text(*text)
                .font_family(BODY.family)
                .font_size(CAPTION.size as f32)
                .font_weight(CAPTION.weight as i32)
                .line_height(CAPTION.line_height as f32 / CAPTION.size as f32)
                .max_lines(1)
                .color(theme::color(palette.label));
            rect().child(label_node).into_element()
        });
        let labels_row = dim(rect()
            .position(
                Position::new_absolute()
                    .left(0.0)
                    .top(knob_height + label_gap),
            )
            .width(Size::px(width))
            .direction(Direction::Horizontal)
            .main_align(Alignment::SpaceBetween)
            .children(labels));

        // freya 0.4.3 dims each element on its own (render_pipeline.rs): a dimmed knob would show the fill.
        root = root
            .children(ticks)
            .child(track_rect.child(fill_rect))
            .child(knob_rect)
            .child(labels_row);
        root = interaction.focus_ring(root, knob / 2.0);

        root.map(self.probe, ProbeExt::probe)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonStyle {
    Primary,
    #[default]
    Secondary,
    Glass,
    GlassProminent,
    Borderless,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControlSize {
    Small,
    #[default]
    Regular,
    Large,
}

#[derive(PartialEq)]
pub struct Button {
    label: String,
    style: ButtonStyle,
    disabled: bool,
    size: ControlSize,
    on: Option<bool>,
    width: Size,
    on_press: Option<Callback<(), ()>>,
    key: DiffKey,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            style: ButtonStyle::default(),
            disabled: false,
            size: ControlSize::default(),
            on: None,
            width: Size::auto(),
            on_press: None,
            key: DiffKey::None,
        }
    }

    pub fn toggled(mut self, on: bool) -> Self {
        self.on = Some(on);
        self
    }

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }

    pub fn on_press(mut self, callback: impl Into<Callback<(), ()>>) -> Self {
        self.on_press = Some(callback.into());
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for Button {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for Button {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let enabled = !self.disabled;
        let style = match self.on {
            Some(true) => ButtonStyle::Primary,
            _ => self.style,
        };
        let clear = Rgba {
            a: 0.0,
            ..palette.control_background
        };
        let hovered = interaction.hovered();
        let (background, ink) = match (style, enabled) {
            (ButtonStyle::Borderless, false) => (clear, palette.tertiary_label),
            (_, false) => (palette.control_background, palette.tertiary_label),
            (ButtonStyle::Primary | ButtonStyle::GlassProminent, true) => {
                (palette.accent, palette.on_accent)
            }
            (ButtonStyle::Secondary, true) if hovered => (palette.control_hover, palette.label),
            (ButtonStyle::Secondary, true) => (palette.control_background, palette.label),
            (ButtonStyle::Glass, true) if hovered => (palette.control_hover, palette.label),
            (ButtonStyle::Glass, true) => (palette.raised_background, palette.label),
            (ButtonStyle::Borderless, true) if hovered => (palette.control_hover, palette.label),
            (ButtonStyle::Borderless, true) => (clear, palette.label),
        };
        let glass = matches!(style, ButtonStyle::Glass | ButtonStyle::GlassProminent);
        let on_press = self.on_press.clone();
        let (height, padding_x, font_size, radius) = match self.size {
            ControlSize::Small => (
                BUTTON.small_height,
                BUTTON.small_padding_x,
                BUTTON.small_font_size,
                BUTTON.small_radius,
            ),
            ControlSize::Large => (
                BUTTON.large_height,
                BUTTON.padding_x,
                BUTTON.font_size,
                BUTTON.large_height / 2.0,
            ),
            ControlSize::Regular => (
                BUTTON.height,
                BUTTON.padding_x,
                BUTTON.font_size,
                BUTTON.radius,
            ),
        };
        let radius = if glass { height / 2.0 } else { radius };

        let button = interaction
            .attach(rect(), enabled)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label.clone())
            .width(self.width.clone())
            .height(Size::px(height))
            .padding(Gaps::new(0.0, padding_x, 0.0, padding_x))
            .corner_radius(radius)
            .center()
            .background(color(background))
            .maybe(glass, |el| {
                el.border(
                    Border::new()
                        .width(ctdesign::HAIRLINE)
                        .alignment(BorderAlignment::Inner)
                        .fill(color(palette.card_border)),
                )
            })
            .map(self.on, |el, on| {
                el.a11y_builder(move |node| {
                    node.set_toggled(if on {
                        accesskit::Toggled::True
                    } else {
                        accesskit::Toggled::False
                    })
                })
            })
            .maybe(enabled && interaction.pressed(), |el| {
                el.scale(BUTTON.pressed_scale)
            })
            .maybe(enabled, |el| {
                el.on_press(move |event: Event<PressEventData>| {
                    // A focused button consumes Enter/Space so global key handlers (Alert) skip it.
                    if matches!(*event, PressEventData::Keyboard(_)) {
                        event.prevent_default();
                    }
                    if let Some(cb) = &on_press {
                        cb.call(());
                    }
                })
            })
            .child(
                label()
                    .text(self.label.clone())
                    .font_family(BODY.family)
                    .font_size(font_size)
                    .font_weight(BUTTON.font_weight)
                    .max_lines(1)
                    .color(color(ink)),
            );
        interaction.focus_ring(button, radius)
    }
}

#[derive(PartialEq)]
pub struct Switch {
    label: &'static str,
    on: bool,
    disabled: bool,
    on_toggle: Option<Callback<bool, ()>>,
    probe: Option<&'static str>,
    tint: Tint,
    key: DiffKey,
}

impl Switch {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            on: false,
            disabled: false,
            on_toggle: None,
            probe: None,
            tint: Tint::Accent,
            key: DiffKey::None,
        }
    }

    pub fn tint(mut self, tint: Tint) -> Self {
        self.tint = tint;
        self
    }

    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_toggle(mut self, callback: impl Into<Callback<bool, ()>>) -> Self {
        self.on_toggle = Some(callback.into());
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for Switch {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl ProbeExt for Switch {
    fn probe(mut self, id: &'static str) -> Self {
        self.probe = Some(id);
        self
    }
}

impl Component for Switch {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let on = crate::controlled::use_controlled(&self.on);
        let interaction = use_interaction();
        let progress = crate::animation::use_switch_progress(on, SWITCH.transition_ms);
        let is_on = *on.read();

        let disabled = self.disabled;
        let on_toggle = self.on_toggle.clone();

        let (track, border) = if is_on {
            (self.tint.fill(&palette), Color::TRANSPARENT)
        } else {
            (
                color(palette.control_background).into(),
                color(palette.separator),
            )
        };

        let mut root = interaction
            .attach(rect(), !disabled)
            .a11y_role(AccessibilityRole::Switch)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_toggled(is_on.into()))
            .width(Size::px(SWITCH.width))
            .height(Size::px(SWITCH.height))
            .corner_radius(SWITCH.height / 2.0)
            .background(track)
            .border(
                Border::new()
                    .width(ctdesign::HAIRLINE)
                    .alignment(BorderAlignment::Inner)
                    .fill(border),
            )
            .maybe(disabled, |el| el.opacity(SWITCH.disabled_opacity))
            .on_press(move |_| {
                if !disabled {
                    let next = !*on.peek();
                    commit(on, &on_toggle, next);
                }
            })
            .child(level::knob(
                SWITCH.inset + SWITCH.travel * progress,
                SWITCH.inset,
                SWITCH.knob_width,
                SWITCH.knob_height,
            ));
        root = interaction.focus_ring(root, SWITCH.height / 2.0);

        root.map(self.probe, ProbeExt::probe)
    }
}

#[derive(PartialEq)]
pub struct SegmentedControl {
    label: &'static str,
    segments: Vec<&'static str>,
    selected: usize,
    on_change: Option<Callback<usize, ()>>,
    key: DiffKey,
}

impl SegmentedControl {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            segments: Vec::new(),
            selected: 0,
            on_change: None,
            key: DiffKey::None,
        }
    }

    pub fn segment(mut self, label: &'static str) -> Self {
        self.segments.push(label);
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
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

impl KeyExt for SegmentedControl {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for SegmentedControl {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let choice = use_choice(Some(self.selected), self.on_change.clone());
        let keyed = use_state(|| false);
        let segments = self.segments.iter().enumerate().map(|(index, text)| {
            Segment {
                text,
                active: choice.is(index),
                take_focus: choice.is(index) && *keyed.read(),
                on_select: choice.select(index),
            }
            .into_element()
        });

        let group = rect()
            .a11y_role(AccessibilityRole::TabList)
            .a11y_alt(self.label)
            .width(Size::fill())
            .direction(Direction::Horizontal)
            .content(Content::flex())
            .padding(Gaps::new_all(SEGMENTED.padding))
            .spacing(SEGMENTED.gap)
            .corner_radius(SEGMENTED.radius)
            .background(color(palette.control_background))
            .children(segments);
        choice.arrow_keys(group, self.segments.len(), keyed)
    }
}

#[derive(PartialEq)]
struct Segment {
    text: &'static str,
    active: bool,
    take_focus: bool,
    on_select: Callback<(), ()>,
}

impl Component for Segment {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        crate::choices::take_focus_when(interaction, self.take_focus);
        let active = self.active;
        let on_select = self.on_select.clone();

        let mut segment = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Tab)
            .a11y_alt(self.text)
            .a11y_builder(move |node| node.set_selected(active))
            .width(Size::flex(1.0))
            .height(Size::px(SEGMENTED.segment_height))
            .corner_radius(SEGMENTED.segment_radius)
            .center()
            .on_press(move |_| on_select.call(()))
            .child(
                rect()
                    .padding(Gaps::new(
                        0.0,
                        SEGMENTED.segment_padding_x,
                        0.0,
                        SEGMENTED.segment_padding_x,
                    ))
                    .child(
                        label()
                            .text(self.text)
                            .font_family(BODY.family)
                            .font_size(SEGMENTED.font_size)
                            .font_weight(SEGMENTED.font_weight)
                            .color(color(palette.label)),
                    ),
            );
        if active {
            segment = segment
                .background(color(palette.selected_control))
                .shadow(knob_shadow());
        }
        interaction.focus_ring(segment, SEGMENTED.segment_radius)
    }
}

#[derive(PartialEq)]
pub struct TextField {
    value: State<String>,
    placeholder: &'static str,
    width: Size,
    auto_focus: bool,
    key: DiffKey,
}

impl TextField {
    pub fn new(value: State<String>, placeholder: &'static str) -> Self {
        Self {
            value,
            placeholder,
            width: Size::fill(),
            auto_focus: false,
            key: DiffKey::None,
        }
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }

    pub fn auto_focus(mut self, auto_focus: bool) -> Self {
        self.auto_focus = auto_focus;
        self
    }
}

impl KeyExt for TextField {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for TextField {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let input = themed_input(
            self.value,
            self.placeholder,
            palette.content_background,
            palette.separator,
            TEXT_FIELD.radius,
            TEXT_FIELD.padding_x,
        )
        .auto_focus(self.auto_focus);
        let field = use_hook(AccessibilityId::new_unique);
        let mut index = use_state(|| None);
        let holder = use_state(ParagraphHolder::default);
        let text_left = use_hook(|| std::rc::Rc::new(std::cell::Cell::new(TEXT_FIELD.padding_x)));
        let left = text_left.clone();
        let value = self.value;
        let text = value.read().clone();
        let ranges = dictionary()
            .map(|dictionary| {
                misspelled(&text, |word| known(dictionary, word))
                    .iter()
                    .map(|range| utf16_range(&text, range))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        edit_menu(
            text_style(rect().width(self.width.clone())),
            field,
            true,
            (true, true),
            Some(Spelling {
                value,
                index,
                styles: None,
            }),
        )
        .child(
            rect()
                .width(Size::fill())
                .on_secondary_down(move |event: Event<PressEventData>| {
                    if let (PressEventData::Mouse(mouse), Some(inner)) =
                        (event.data(), holder.peek().0.borrow().as_ref())
                    {
                        let x = (mouse.element_location.x as f32 - left.get())
                            * inner.scale_factor as f32;
                        let position = inner
                            .paragraph
                            .get_glyph_position_at_coordinate((x, 0.0))
                            .position;
                        index.set(Some(char_index(&value.peek(), position.max(0) as usize)));
                    }
                })
                .child(input.a11y_id(field))
                .maybe_child((!ranges.is_empty()).then(|| {
                    crate::glass::leaf(Squiggles {
                        holder: holder.read().clone(),
                        input: Some(field),
                        text_left,
                        ranges,
                        color: palette.destructive,
                        offset: (0.0, 0.0),
                        layout: crate::glass::layout(
                            Size::fill(),
                            Size::fill(),
                            Position::new_absolute(),
                        ),
                    })
                })),
        )
    }
}

#[derive(PartialEq)]
pub struct SearchField {
    value: State<String>,
    placeholder: &'static str,
    width: Size,
    auto_focus: bool,
    on_focus_change: Option<Callback<bool, ()>>,
    on_navigate: Option<Callback<NamedKey, ()>>,
    a11y_id: Option<AccessibilityId>,
    tokens: Option<State<Vec<String>>>,
    key: DiffKey,
}

impl SearchField {
    pub fn tokens(mut self, tokens: State<Vec<String>>) -> Self {
        self.tokens = Some(tokens);
        self
    }

    pub fn new(value: State<String>, placeholder: &'static str) -> Self {
        Self {
            value,
            placeholder,
            width: Size::fill(),
            auto_focus: false,
            on_focus_change: None,
            on_navigate: None,
            a11y_id: None,
            tokens: None,
            key: DiffKey::None,
        }
    }

    pub fn a11y_id(mut self, a11y_id: AccessibilityId) -> Self {
        self.a11y_id = Some(a11y_id);
        self
    }

    pub fn auto_focus(mut self, auto_focus: bool) -> Self {
        self.auto_focus = auto_focus;
        self
    }

    pub fn on_focus_change(mut self, callback: impl Into<Callback<bool, ()>>) -> Self {
        self.on_focus_change = Some(callback.into());
        self
    }

    pub fn on_navigate(mut self, callback: impl Into<Callback<NamedKey, ()>>) -> Self {
        self.on_navigate = Some(callback.into());
        self
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }
}

impl KeyExt for SearchField {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for SearchField {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let mut value = self.value;
        let padding = SETTINGS.search_padding_x as f32;
        let icon = SETTINGS.search_icon as f32;
        let gap = SETTINGS.search_gap as f32;
        let a11y_id = use_hook(|| self.a11y_id.unwrap_or_else(AccessibilityId::new_unique));
        let focus = use_focus(a11y_id);
        let on_focus_change = use_reactive(&self.on_focus_change);
        let mut reported = use_state(|| false);
        use_side_effect(move || {
            let focused = focus.read().is_focused();
            if *reported.peek() != focused {
                reported.set(focused);
                if let Some(callback) = &*on_focus_change.peek() {
                    callback.call(focused);
                }
            }
        });
        let mut tokens_width = use_state(|| 0.0f32);
        let tokens = self.tokens;
        let shown_tokens = tokens
            .map(|tokens| tokens.read().clone())
            .unwrap_or_default();
        let leading = padding
            + icon
            + gap
            + if shown_tokens.is_empty() {
                0.0
            } else {
                *tokens_width.read() + gap
            };
        let mut input = themed_input(
            value,
            self.placeholder,
            palette.control_background,
            Rgba {
                a: 0.0,
                ..palette.control_background
            },
            TEXT_FIELD.height / 2.0,
            padding + icon + gap,
        )
        .theme_layout(InputLayoutThemePartial {
            corner_radius: Some(Preference::Specific(CornerRadius::new_all(
                TEXT_FIELD.height / 2.0,
            ))),
            inner_margin: Some(Preference::Specific(Gaps::new(
                (TEXT_FIELD.height - BODY.line_height as f32) / 2.0,
                padding + icon + gap,
                (TEXT_FIELD.height - BODY.line_height as f32) / 2.0,
                leading,
            ))),
        })
        .a11y_id(a11y_id)
        .auto_focus(self.auto_focus);
        let on_navigate = self.on_navigate.clone();
        if on_navigate.is_some() || tokens.is_some() {
            input =
                input.on_pre_key_down(move |event: Event<KeyboardEventData>| match &event.key {
                    Key::Named(
                        key @ (NamedKey::ArrowUp | NamedKey::ArrowDown | NamedKey::Enter),
                    ) if on_navigate.is_some() => {
                        event.stop_propagation();
                        if let Some(on_navigate) = &on_navigate {
                            on_navigate.call(*key);
                        }
                        false
                    }
                    Key::Named(NamedKey::Backspace) if value.peek().is_empty() => {
                        if let Some(mut tokens) = tokens {
                            tokens.write().pop();
                        }
                        event.stop_propagation();
                        false
                    }
                    _ => default_pre_key(&event),
                });
        }
        if !value.read().is_empty() {
            input = input.trailing(
                rect()
                    .a11y_role(AccessibilityRole::Button)
                    .a11y_alt(text!("clear-search"))
                    .on_pointer_down(move |event: Event<PointerEventData>| {
                        if event.is_primary() {
                            value.set(String::new());
                        }
                    })
                    .child(symbol(Symbol::XMark, icon, palette.secondary_label)),
            );
        }

        edit_menu(
            text_style(rect().width(self.width.clone())),
            a11y_id,
            true,
            (true, true),
            None,
        )
        .child(input)
        .child(
            rect()
                .position(
                    Position::new_absolute()
                        .left(padding)
                        .top((TEXT_FIELD.height - icon) / 2.0),
                )
                .child(symbol(
                    Symbol::MagnifyingGlass,
                    icon,
                    palette.tertiary_label,
                )),
        )
        .maybe(!shown_tokens.is_empty(), |el| {
            el.child(
                rect()
                    .position(
                        Position::new_absolute()
                            .left(padding + icon + gap)
                            .top((TEXT_FIELD.height - ctdesign::TOKEN_FIELD.token_height) / 2.0),
                    )
                    .direction(Direction::Horizontal)
                    .spacing(ctdesign::TOKEN_FIELD.gap)
                    .on_sized(move |event: Event<SizedEventData>| {
                        tokens_width.set_if_modified(event.area.width());
                    })
                    .children(
                        shown_tokens.into_iter().map(|token| {
                            crate::token_field::token_chip(token, false).into_element()
                        }),
                    ),
            )
        })
    }
}

// Mirrors Input's default pre-handler (freya-components 0.4.3, input.rs).
pub(crate) fn default_pre_key(event: &Event<KeyboardEventData>) -> bool {
    match &event.key {
        Key::Named(NamedKey::Enter | NamedKey::Escape | NamedKey::Shift) => true,
        Key::Named(NamedKey::Tab) => false,
        _ => {
            event.stop_propagation();
            event.prevent_default();
            true
        }
    }
}

pub(crate) fn text_style(element: Rect) -> Rect {
    element.font_family(BODY.family).font_size(BODY.size as f32)
}

pub(crate) fn themed_input(
    value: State<String>,
    placeholder: &'static str,
    background: Rgba,
    border: Rgba,
    radius: f32,
    padding_x: f32,
) -> Input {
    let (generation, mut typed) = use_input_generation(value);
    let palette = Palette::current();
    let margin_y = (TEXT_FIELD.height - BODY.line_height as f32) / 2.0;
    let color = |c: Rgba| Some(Preference::Specific(color(c)));
    Input::new(value)
        .key(generation)
        .on_validate(move |validator: InputValidator| typed.set(validator.text().clone()))
        .placeholder(placeholder)
        .width(Size::fill())
        .theme_colors(InputColorsThemePartial {
            background: color(background),
            focus_background: color(background),
            border_fill: color(border),
            focus_border_fill: color(palette.accent),
            color: color(palette.label),
            placeholder_color: color(palette.secondary_label),
        })
        .theme_layout(InputLayoutThemePartial {
            corner_radius: Some(Preference::Specific(CornerRadius::new_all(radius))),
            inner_margin: Some(Preference::Specific(Gaps::new(
                margin_y, padding_x, margin_y, padding_x,
            ))),
        })
}

// freya-edit 0.4.3 set() checks the UTF-16 cursor against a byte length: https://docs.rs/crate/freya-edit/0.4.3/source/src/rope_editor.rs#273
fn use_input_generation(value: State<String>) -> (u32, State<String>) {
    let typed = use_state(String::new);
    let mut seen = use_state(|| value.peek().clone());
    let mut generation = use_state(|| 0u32);
    let current = value.read().clone();
    if current != *seen.peek() {
        if current != *typed.peek() && cursor_may_pass_end(&seen.peek(), &current) {
            *generation.write() += 1;
        }
        seen.set(current);
    }
    (*generation.peek(), typed)
}

fn cursor_may_pass_end(old: &str, new: &str) -> bool {
    let units = new.encode_utf16().count();
    old.encode_utf16().count().min(new.len()) > units
}

#[derive(PartialEq)]
pub struct Checkbox {
    label: &'static str,
    checked: bool,
    disabled: bool,
    on_toggle: Option<Callback<bool, ()>>,
    probe: Option<&'static str>,
    width: Size,
    tint: Tint,
    key: DiffKey,
}

impl Checkbox {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            checked: false,
            disabled: false,
            on_toggle: None,
            probe: None,
            width: Size::auto(),
            tint: Tint::Accent,
            key: DiffKey::None,
        }
    }

    pub fn tint(mut self, tint: Tint) -> Self {
        self.tint = tint;
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_toggle(mut self, callback: impl Into<Callback<bool, ()>>) -> Self {
        self.on_toggle = Some(callback.into());
        self
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for Checkbox {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl ProbeExt for Checkbox {
    fn probe(mut self, id: &'static str) -> Self {
        self.probe = Some(id);
        self
    }
}

impl Component for Checkbox {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let checked = crate::controlled::use_controlled(&self.checked);
        let on_toggle = self.on_toggle.clone();
        let disabled = self.disabled;
        let label_text = self.label;

        let size = CHECKBOX.size;

        let interaction = use_interaction();

        let (box_bg, box_border) = if *checked.read() {
            (self.tint.fill(&palette), Color::TRANSPARENT)
        } else {
            (
                color(palette.control_background).into(),
                color(palette.separator),
            )
        };
        let label_color = if self.disabled {
            color(palette.tertiary_label)
        } else {
            color(palette.label)
        };

        let mut checkbox = rect()
            .width(Size::px(size))
            .height(Size::px(size))
            .corner_radius(CHECKBOX.radius)
            .background(box_bg)
            .border(
                Border::new()
                    .width(ctdesign::HAIRLINE)
                    .alignment(BorderAlignment::Inner)
                    .fill(box_border),
            )
            .center();

        if *checked.read() {
            checkbox = checkbox.child(symbol(Symbol::Checkmark, CHECKBOX.mark, palette.on_accent));
        }
        checkbox = interaction.focus_ring(checkbox, CHECKBOX.radius);

        let is_checked = *checked.read();
        let root = interaction
            .attach(rect(), !disabled)
            .width(self.width.clone())
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .a11y_role(AccessibilityRole::CheckBox)
            .a11y_alt(label_text)
            .a11y_builder(move |node| node.set_toggled(is_checked.into()))
            .spacing(SETTINGS.check.gap as f32)
            .on_press(move |_| {
                if !disabled {
                    let next = !*checked.peek();
                    commit(checked, &on_toggle, next);
                }
            })
            .child(checkbox)
            .child(
                label()
                    .text(label_text)
                    .font_family(BODY.family)
                    .font_size(BODY.size as f32)
                    .font_weight(BODY.weight as i32)
                    .line_height(BODY.line_height as f32 / BODY.size as f32)
                    .color(label_color),
            );

        root.map(self.probe, ProbeExt::probe)
    }
}

#[cfg(test)]
mod tests {
    use super::cursor_may_pass_end;

    #[test]
    fn a_shorter_non_ascii_value_may_leave_the_cursor_past_the_end() {
        assert!(cursor_may_pass_end("Seville", "Málaga"));
        assert!(cursor_may_pass_end("añoo año", "año año"));
        assert!(cursor_may_pass_end("hello world", "😀"));
    }

    #[test]
    fn clearing_growing_or_ascii_values_keep_the_cursor_inside() {
        assert!(!cursor_may_pass_end("abc", ""));
        assert!(!cursor_may_pass_end("Málaga", "Malaga"));
        assert!(!cursor_may_pass_end("año", "años"));
    }
}

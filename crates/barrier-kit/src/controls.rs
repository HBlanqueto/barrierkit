//! Named UI controls (buttons, toggles, fields) built on Freya components.

pub use freya_components::menu::Menu;
pub use freya_components::popup::Popup as Popover;
pub use freya_components::scrollviews::ScrollView;

use freya_components::input::{Input, InputColorsThemePartial, InputLayoutThemePartial};
use freya_components::svg_viewer::SvgViewer;
use freya_components::theming::macros::Preference;

use freya_animation::prelude::{AnimNum, Ease, Function, use_animation_transition};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};
use vow_design::{
    BUTTON, CAPTION, CHECKBOX, KNOB_SHADOW, Rgba, SEGMENTED, SETTINGS, SLIDER, SWITCH, TEXT,
    TEXT_FIELD, TEXT_LINE_HEIGHT, TOOLGROUP,
};

use crate::ProbeExt;
use crate::choices::use_choice;
use crate::interaction::use_interaction;
use crate::password::PasswordField;
use crate::platform::inserts_text;

fn secure_field_accepts_text(key: &Key, modifiers: Modifiers) -> bool {
    matches!(key, Key::Character(_)) && inserts_text(modifiers)
}

#[derive(Clone)]
pub struct SecureField {
    field: State<PasswordField>,
    width: f32,
    key: DiffKey,
}

impl SecureField {
    pub fn new() -> Self {
        Self {
            field: use_state(PasswordField::default),
            width: 240.0,
            key: DiffKey::None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl Default for SecureField {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyExt for SecureField {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl PartialEq for SecureField {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Component for SecureField {
    fn render(&self) -> impl IntoElement {
        let mut field = self.field;
        let masked = field.read().masked();
        rect()
            .width(Size::px(self.width))
            .height(Size::px(TEXT.size as f32 * TEXT_LINE_HEIGHT))
            .padding(Gaps::new(6.0, 8.0, 6.0, 8.0))
            .on_key_down(move |event: Event<KeyboardEventData>| {
                let mut field = field.write();
                match &event.key {
                    Key::Named(NamedKey::Backspace) => field.backspace(),
                    Key::Character(text)
                        if secure_field_accepts_text(&event.key, event.modifiers) =>
                    {
                        for ch in text.chars() {
                            field.insert(ch);
                        }
                    }
                    _ => {}
                }
            })
            .child(label().text(masked))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolGroupStyle {
    #[default]
    Island,
    Flat,
}

#[derive(PartialEq)]
pub struct ToolGroup {
    children: Vec<Element>,
    style: ToolGroupStyle,
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
        let style = match crate::theme::resolved().sidebar.style {
            vow_appearance::SidebarStyle::Island => ToolGroupStyle::Island,
            vow_appearance::SidebarStyle::Flat => ToolGroupStyle::Flat,
        };
        Self {
            children: Vec::new(),
            style,
            probe: None,
            key: DiffKey::None,
        }
    }

    pub fn flat(mut self) -> Self {
        self.style = ToolGroupStyle::Flat;
        self
    }

    pub fn island(mut self) -> Self {
        self.style = ToolGroupStyle::Island;
        self
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
        let resolved = crate::theme::resolved();
        let palette = resolved.toolgroup.palette;

        let mut items: Vec<Element> = Vec::with_capacity(self.children.len());
        let last = self.children.len().saturating_sub(1);
        for (i, child) in self.children.iter().cloned().enumerate() {
            items.push(child);
            if self.style == ToolGroupStyle::Island && i != last {
                items.push(
                    rect()
                        .width(Size::px(TOOLGROUP.separator_w as f32))
                        .height(Size::px(TOOLGROUP.separator_h as f32))
                        .background(rgba(palette.separator))
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
        if let Some(id) = self.probe {
            root = root.a11y_builder(move |node| node.set_author_id(id));
        }

        if self.style == ToolGroupStyle::Island {
            return root
                .background(rgba(resolved.toolgroup.bg))
                .corner_radius((TOOLGROUP.height as f32 / 2.0).max(1.0))
                .into_element();
        }

        root.into_element()
    }
}

#[derive(PartialEq)]
pub struct ToolButton {
    label: String,
    icon: &'static str,
    disabled: bool,
    on_press: Option<Callback<(), ()>>,
    probe: Option<&'static str>,
    key: DiffKey,
}

impl ToolButton {
    pub fn new(label: impl Into<String>, icon: &'static str) -> Self {
        Self {
            label: label.into(),
            icon,
            disabled: false,
            on_press: None,
            probe: None,
            key: DiffKey::None,
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

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for ToolButton {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl ProbeExt for ToolButton {
    fn probe(mut self, id: &'static str) -> Self {
        self.probe = Some(id);
        self
    }
}

impl Component for ToolButton {
    fn render(&self) -> impl IntoElement {
        let resolved = crate::theme::resolved();
        let palette = resolved.toolgroup.palette;
        let mut hovered = use_state(|| false);

        let island = resolved.sidebar.style == vow_appearance::SidebarStyle::Island;
        let (width, height, radius) = if island {
            (
                TOOLGROUP.button_w as f32,
                TOOLGROUP.button_h as f32,
                (TOOLGROUP.button_h as f32 / 2.0).max(1.0),
            )
        } else {
            (
                TOOLGROUP.flat_button_w as f32,
                TOOLGROUP.flat_button_h as f32,
                TOOLGROUP.flat_radius as f32,
            )
        };

        let fg = if self.disabled {
            palette.disabled_ink
        } else {
            palette.ink
        };
        let bg = if !self.disabled && *hovered.read() {
            rgba(resolved.sidebar.palette.hover_bg)
        } else {
            Color::TRANSPARENT
        };

        let mut root = rect()
            .width(Size::px(width))
            .height(Size::px(height))
            .corner_radius(radius)
            .background(bg)
            .cross_align(Alignment::Center)
            .center()
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label.clone())
            .a11y_focusable(true)
            .on_pointer_enter(move |_| hovered.set(true))
            .on_pointer_leave(move |_| hovered.set(false))
            .child(svg_viewer(self.icon, fg, TOOLGROUP.icon));

        if !self.disabled
            && let Some(callback) = self.on_press.clone()
        {
            root = root.on_press(move |_| callback.call(()));
        }
        if let Some(id) = self.probe {
            root = root.a11y_builder(move |node| node.set_author_id(id));
        }

        root
    }
}

#[derive(PartialEq)]
pub struct Slider {
    steps: usize,
    value: usize,
    labels: &'static [&'static str],
    disabled: bool,
    on_change: Option<Callback<usize, ()>>,
    probe: Option<&'static str>,
    knob_id: Option<&'static str>,
    step_ids: &'static [&'static str],
    key: DiffKey,
}

impl Slider {
    pub fn stepped(steps: usize) -> Self {
        Self {
            steps: steps.max(2),
            value: 0,
            labels: &[],
            disabled: false,
            on_change: None,
            probe: None,
            knob_id: None,
            step_ids: &[],
            key: DiffKey::None,
        }
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

    pub fn sub_ids(mut self, knob: &'static str, steps: &'static [&'static str]) -> Self {
        self.knob_id = Some(knob);
        self.step_ids = steps;
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
        let resolved = crate::theme::resolved();
        let base = vow_appearance::palette(resolved.scheme);
        let value = use_state(|| self.value);
        let dragging = use_state(|| false);
        let on_change = self.on_change.clone();

        let width = SETTINGS.strength.width as f32;
        let knob = SLIDER.knob;
        let track = SLIDER.track;
        let tick = SETTINGS.strength.tick as f32;
        let label_gap = SETTINGS.strength.label_gap as f32;
        let steps = self.steps;
        let segments = steps.saturating_sub(1).max(1) as f32;
        let run = width - knob;
        let fraction = *value.read() as f32 / segments;
        let knob_x = run * fraction;

        let set_from_x = move |x: f32| {
            let x = (x - knob / 2.0).clamp(0.0, run);
            let step = ((x / run) * segments).round() as usize;
            step.min(steps - 1)
        };

        let update = {
            let mut value = value;
            let on_change = on_change.clone();
            move |new_value: usize| {
                if new_value != *value.read() {
                    value.set(new_value);
                    if let Some(cb) = &on_change {
                        cb.call(new_value);
                    }
                }
            }
        };

        let track_color = if self.disabled {
            rgba(resolved.surfaces.control)
        } else {
            rgba(SLIDER.track_color)
        };
        let fill_color = rgba(resolved.accent);
        let knob_color = Color::WHITE;
        let knob_shadow = rgba(KNOB_SHADOW.color);

        let mut root = rect()
            .width(Size::px(width))
            .height(Size::px(
                knob + label_gap + CAPTION.size as f32 * TEXT_LINE_HEIGHT,
            ))
            .a11y_role(AccessibilityRole::Slider);

        if !self.disabled {
            let mut update_mouse = update.clone();
            let mut dragging_mouse = dragging;
            root = root.on_mouse_down(move |event: Event<MouseEventData>| {
                dragging_mouse.set(true);
                let new_value = set_from_x(event.element_location.x as f32);
                update_mouse(new_value);
            });
            let mut update_move = update.clone();
            root = root.on_pointer_move(move |event: Event<PointerEventData>| {
                if *dragging.read() {
                    let new_value = set_from_x(event.element_location().x as f32);
                    update_move(new_value);
                }
            });
            let mut dragging_up = dragging;
            root = root.on_mouse_up(move |_: Event<MouseEventData>| {
                dragging_up.set(false);
            });
            root = root.on_pointer_leave(move |_: Event<PointerEventData>| {
                dragging_up.set(false);
            });
        }

        if self.disabled {
            root = root.opacity(0.4);
        }

        let track_rect = rect()
            .position(
                Position::new_absolute()
                    .left(knob / 2.0)
                    .top((knob - track) / 2.0),
            )
            .width(Size::px(run))
            .height(Size::px(track))
            .corner_radius((track / 2.0).max(1.0))
            .background(track_color);

        let fill_rect = rect()
            .position(Position::new_absolute().left(0.0).top((knob - track) / 2.0))
            .width(Size::px(knob_x + knob / 2.0))
            .height(Size::px(track))
            .corner_radius((track / 2.0).max(1.0))
            .background(fill_color);

        let mut ticks: Vec<Element> = Vec::with_capacity(self.steps);
        for i in 0..self.steps {
            let x = knob / 2.0 + run * (i as f32 / segments);
            let color = if i <= *value.read() && !self.disabled {
                base.on_accent
            } else {
                base.line_strong
            };
            ticks.push(
                rect()
                    .position(
                        Position::new_absolute()
                            .left(x - tick / 2.0)
                            .top((knob - tick) / 2.0),
                    )
                    .width(Size::px(tick))
                    .height(Size::px(tick))
                    .corner_radius((tick / 2.0).max(1.0))
                    .background(rgba(color))
                    .into_element(),
            );
        }

        let mut knob_rect = rect()
            .position(Position::new_absolute().left(knob_x).top(0.0))
            .width(Size::px(knob))
            .height(Size::px(knob))
            .corner_radius((knob / 2.0).max(1.0))
            .background(knob_color)
            .shadow((
                0.0,
                KNOB_SHADOW.y_offset as f32,
                KNOB_SHADOW.blur as f32,
                0.0,
                knob_shadow,
            ));
        if let Some(id) = self.knob_id {
            knob_rect = knob_rect.a11y_builder(move |node| node.set_author_id(id));
        }

        const STEP_LABEL_WIDTHS: [f32; 4] = [24.5, 38.5, 34.5, 40.0];
        let mut labels: Vec<Element> = Vec::new();
        for (i, text) in self.labels.iter().enumerate().take(self.steps) {
            let color = if i == *value.read() {
                base.ink_strong
            } else {
                base.ink_muted
            };
            let label_node = label()
                .text(*text)
                .font_family(TEXT.family)
                .font_size(CAPTION.size as f32)
                .font_weight(CAPTION.weight as i32)
                .line_height(CAPTION.line_height as f32 / CAPTION.size as f32)
                .width(Size::fill())
                .color(rgba(color));
            let width = if self.steps == 4 {
                Size::px(STEP_LABEL_WIDTHS[i])
            } else {
                Size::auto()
            };
            let mut wrapper = rect().width(width).height(Size::auto()).child(label_node);
            if let Some(id) = self.step_ids.get(i).copied() {
                wrapper = wrapper.a11y_builder(move |node| node.set_author_id(id));
            }
            labels.push(wrapper.into_element());
        }
        let labels_row = rect()
            .position(
                Position::new_absolute()
                    .left(knob / 2.0)
                    .top(knob + label_gap),
            )
            .width(Size::px(run))
            .direction(Direction::Horizontal)
            .main_align(Alignment::SpaceBetween)
            .children(labels);

        let mut children = vec![fill_rect.into_element(), track_rect.into_element()];
        children.extend(ticks);
        children.push(knob_rect.into_element());
        children.push(labels_row.into_element());
        root = root.children(children);

        if let Some(id) = self.probe {
            root = root.a11y_builder(move |node| node.set_author_id(id));
        }

        root
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonStyle {
    Primary,
    #[default]
    Secondary,
}

#[derive(PartialEq)]
pub struct Button {
    label: String,
    style: ButtonStyle,
    disabled: bool,
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
            width: Size::auto(),
            on_press: None,
            key: DiffKey::None,
        }
    }

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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
        let resolved = crate::theme::resolved();
        let base = vow_appearance::palette(resolved.scheme);
        let interaction = use_interaction();
        let enabled = !self.disabled;
        let (background, ink) = match (self.style, enabled) {
            (_, false) => (resolved.surfaces.control, base.ink_faint),
            (ButtonStyle::Primary, true) => (resolved.accent, base.on_accent),
            (ButtonStyle::Secondary, true) if interaction.hovered() => {
                (resolved.surfaces.hover, base.ink_strong)
            }
            (ButtonStyle::Secondary, true) => (resolved.surfaces.control, base.ink_strong),
        };
        let on_press = self.on_press.clone();

        let button = interaction
            .attach(rect(), enabled)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label.clone())
            .width(self.width.clone())
            .height(Size::px(BUTTON.height))
            .padding(Gaps::new(0.0, BUTTON.padding_x, 0.0, BUTTON.padding_x))
            .corner_radius(BUTTON.radius)
            .center()
            .background(rgba(background))
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
            .child(
                label()
                    .text(self.label.clone())
                    .font_family(TEXT.family)
                    .font_size(BUTTON.font_size)
                    .font_weight(BUTTON.font_weight)
                    .max_lines(1)
                    .color(rgba(ink)),
            );
        interaction.focus_ring(button, BUTTON.radius)
    }
}

#[derive(PartialEq)]
pub struct Switch {
    label: &'static str,
    on: bool,
    disabled: bool,
    on_toggle: Option<Callback<bool, ()>>,
    probe: Option<&'static str>,
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
            key: DiffKey::None,
        }
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
        let resolved = crate::theme::resolved();
        let base = vow_appearance::palette(resolved.scheme);
        let on = use_state(|| self.on);
        let interaction = use_interaction();
        let reduced_motion = use_reactive(&resolved.reduced_motion);
        let animation = use_animation_transition(on, move |from, to| {
            let duration = if *reduced_motion.peek() {
                0
            } else {
                SWITCH.transition_ms
            };
            AnimNum::new(if from { 1.0 } else { 0.0 }, if to { 1.0 } else { 0.0 })
                .time(duration)
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        let is_on = *on.read();
        let progress = if *reduced_motion.read() {
            if is_on { 1.0 } else { 0.0 }
        } else {
            animation.get().value()
        };

        let disabled = self.disabled;
        let mut flip = {
            let mut on = on;
            let on_toggle = self.on_toggle.clone();
            move || {
                if disabled {
                    return;
                }
                let new = !*on.read();
                on.set(new);
                if let Some(cb) = &on_toggle {
                    cb.call(new);
                }
            }
        };

        let (track, border) = if is_on {
            (rgba(resolved.accent), Color::TRANSPARENT)
        } else {
            (rgba(resolved.surfaces.control), rgba(base.line))
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
                    .width(1.0)
                    .alignment(BorderAlignment::Inner)
                    .fill(border),
            )
            .maybe(disabled, |el| el.opacity(SWITCH.disabled_opacity))
            .on_press(move |_| flip())
            .child(
                rect()
                    .position(
                        Position::new_absolute()
                            .left(SWITCH.inset + SWITCH.travel * progress)
                            .top(SWITCH.inset),
                    )
                    .width(Size::px(SWITCH.knob))
                    .height(Size::px(SWITCH.knob))
                    .corner_radius(SWITCH.knob / 2.0)
                    .background(rgba(SWITCH.knob_color))
                    .shadow((
                        0.0,
                        KNOB_SHADOW.y_offset as f32,
                        KNOB_SHADOW.blur as f32,
                        0.0,
                        rgba(KNOB_SHADOW.color),
                    )),
            );
        root = interaction.focus_ring(root, SWITCH.height / 2.0);

        if let Some(id) = self.probe {
            root = root.a11y_builder(move |node| node.set_author_id(id));
        }

        root
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
        let resolved = crate::theme::resolved();
        let choice = use_choice(Some(self.selected), self.on_change.clone());
        let segments = self.segments.iter().enumerate().map(|(index, text)| {
            Segment {
                text,
                active: choice.is(index),
                on_select: choice.select(index),
            }
            .into_element()
        });

        rect()
            .a11y_role(AccessibilityRole::TabList)
            .a11y_alt(self.label)
            .width(Size::fill())
            .direction(Direction::Horizontal)
            .content(Content::flex())
            .padding(Gaps::new_all(SEGMENTED.padding))
            .spacing(SEGMENTED.gap)
            .corner_radius(SEGMENTED.radius)
            .background(rgba(resolved.surfaces.control))
            .children(segments)
    }
}

#[derive(PartialEq)]
struct Segment {
    text: &'static str,
    active: bool,
    on_select: Callback<(), ()>,
}

impl Component for Segment {
    fn render(&self) -> impl IntoElement {
        let resolved = crate::theme::resolved();
        let base = vow_appearance::palette(resolved.scheme);
        let interaction = use_interaction();
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
                            .font_family(TEXT.family)
                            .font_size(SEGMENTED.font_size)
                            .font_weight(SEGMENTED.font_weight)
                            .color(rgba(if active {
                                base.ink_strong
                            } else {
                                base.ink_muted
                            })),
                    ),
            );
        if active {
            segment = segment.background(rgba(resolved.surfaces.surface)).shadow((
                0.0,
                KNOB_SHADOW.y_offset as f32,
                KNOB_SHADOW.blur as f32,
                0.0,
                rgba(KNOB_SHADOW.color),
            ));
        }
        interaction.focus_ring(segment, SEGMENTED.segment_radius)
    }
}

const SEARCH_ICON: &str = r#"<svg viewBox="0 0 16 16"><circle cx="6.8" cy="6.8" r="4.2" fill="none" stroke="currentColor" stroke-width="1.4"/><path d="M10 10l3.2 3.2" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/></svg>"#;
const CLEAR_ICON: &str = r#"<svg viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/></svg>"#;

#[derive(PartialEq)]
pub struct TextField {
    value: State<String>,
    placeholder: &'static str,
    width: Size,
    key: DiffKey,
}

impl TextField {
    pub fn new(value: State<String>, placeholder: &'static str) -> Self {
        Self {
            value,
            placeholder,
            width: Size::fill(),
            key: DiffKey::None,
        }
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
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
        let resolved = crate::theme::resolved();
        let base = vow_appearance::palette(resolved.scheme);
        let input = themed_input(
            self.value,
            self.placeholder,
            resolved.surfaces.surface,
            base.line,
            TEXT_FIELD.radius,
            TEXT_FIELD.padding_x,
        );
        text_style(rect().width(self.width.clone())).child(input)
    }
}

#[derive(PartialEq)]
pub struct SearchField {
    value: State<String>,
    placeholder: &'static str,
    width: Size,
    key: DiffKey,
}

impl SearchField {
    pub fn new(value: State<String>, placeholder: &'static str) -> Self {
        Self {
            value,
            placeholder,
            width: Size::fill(),
            key: DiffKey::None,
        }
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
        let resolved = crate::theme::resolved();
        let base = vow_appearance::palette(resolved.scheme);
        let mut value = self.value;
        let padding = SETTINGS.search_padding_x as f32;
        let icon = SETTINGS.search_icon as f32;
        let gap = SETTINGS.search_gap as f32;
        let mut input = themed_input(
            value,
            self.placeholder,
            resolved.surfaces.control,
            Rgba {
                a: 0.0,
                ..resolved.surfaces.control
            },
            TEXT_FIELD.height / 2.0,
            padding + icon + gap,
        );
        if !value.read().is_empty() {
            input = input.trailing(
                rect()
                    .a11y_role(AccessibilityRole::Button)
                    .a11y_alt("Borrar búsqueda")
                    .on_pointer_down(move |event: Event<PointerEventData>| {
                        if event.is_primary() {
                            value.set(String::new());
                        }
                    })
                    .child(svg_viewer(CLEAR_ICON, base.ink_muted, icon as u32)),
            );
        }

        text_style(rect().width(self.width.clone()))
            .child(input)
            .child(
                rect()
                    .position(
                        Position::new_absolute()
                            .left(padding)
                            .top((TEXT_FIELD.height - icon) / 2.0),
                    )
                    .child(svg_viewer(SEARCH_ICON, base.ink_faint, icon as u32)),
            )
    }
}

fn text_style(element: Rect) -> Rect {
    element.font_family(TEXT.family).font_size(TEXT.size as f32)
}

fn themed_input(
    value: State<String>,
    placeholder: &'static str,
    background: Rgba,
    border: Rgba,
    radius: f32,
    padding_x: f32,
) -> Input {
    let resolved = crate::theme::resolved();
    let base = vow_appearance::palette(resolved.scheme);
    let margin_y = (TEXT_FIELD.height - TEXT.line_height as f32) / 2.0;
    let color = |c: Rgba| Some(Preference::Specific(rgba(c)));
    Input::new(value)
        .placeholder(placeholder)
        .width(Size::fill())
        .theme_colors(InputColorsThemePartial {
            background: color(background),
            focus_background: color(background),
            border_fill: color(border),
            focus_border_fill: color(resolved.accent),
            color: color(base.ink_strong),
            placeholder_color: color(base.ink_faint),
        })
        .theme_layout(InputLayoutThemePartial {
            corner_radius: Some(Preference::Specific(CornerRadius::new_all(radius))),
            inner_margin: Some(Preference::Specific(Gaps::new(
                margin_y, padding_x, margin_y, padding_x,
            ))),
        })
}

#[derive(PartialEq)]
pub struct Checkbox {
    label: &'static str,
    checked: bool,
    disabled: bool,
    on_toggle: Option<Callback<bool, ()>>,
    probe: Option<&'static str>,
    width: Size,
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
            key: DiffKey::None,
        }
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
        let resolved = crate::theme::resolved();
        let base = vow_appearance::palette(resolved.scheme);
        let checked = use_state(|| self.checked);
        let on_toggle = self.on_toggle.clone();
        let disabled = self.disabled;
        let label_text = self.label;

        let size = CHECKBOX.size;
        let check_svg = r#"<svg viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="m2.5 6.2 2.3 2.3 4.7-4.8"/></svg>"#;

        let interaction = use_interaction();
        let mut toggle = {
            let mut checked = checked;
            let on_toggle = on_toggle;
            move || {
                if !disabled {
                    let new = !*checked.read();
                    checked.set(new);
                    if let Some(cb) = &on_toggle {
                        cb.call(new);
                    }
                }
            }
        };

        let box_bg = if *checked.read() {
            rgba(CHECKBOX.on)
        } else {
            rgba(resolved.surfaces.surface)
        };
        let box_border = rgba(CHECKBOX.border);
        let label_color = if self.disabled {
            rgba(base.ink_faint)
        } else {
            rgba(base.ink_strong)
        };

        let mut checkbox = rect()
            .width(Size::px(size))
            .height(Size::px(size))
            .corner_radius(CHECKBOX.radius)
            .background(box_bg)
            .border(
                Border::new()
                    .width(1.0)
                    .alignment(BorderAlignment::Inner)
                    .fill(if *checked.read() {
                        Color::TRANSPARENT
                    } else {
                        box_border
                    }),
            )
            .center();

        if *checked.read() {
            checkbox = checkbox.child(svg_viewer(check_svg, base.on_accent, size as u32 - 4));
        }
        checkbox = interaction.focus_ring(checkbox, CHECKBOX.radius);

        let is_checked = *checked.read();
        let mut root = interaction
            .attach(rect(), !disabled)
            .width(self.width.clone())
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .a11y_role(AccessibilityRole::CheckBox)
            .a11y_alt(label_text)
            .a11y_builder(move |node| node.set_toggled(is_checked.into()))
            .on_press(move |_| toggle())
            .child(checkbox)
            .child(
                rect()
                    .width(Size::px(SETTINGS.check.gap as f32))
                    .height(Size::px(1.0))
                    .background(Color::TRANSPARENT),
            )
            .child(
                label()
                    .text(label_text)
                    .font_family(TEXT.family)
                    .font_size(TEXT.size as f32)
                    .font_weight(TEXT.weight as i32)
                    .line_height(TEXT.line_height as f32 / TEXT.size as f32)
                    .color(label_color),
            );

        if let Some(id) = self.probe {
            root = root.a11y_builder(move |node| node.set_author_id(id));
        }

        root
    }
}

fn svg_viewer(template: &'static str, color: Rgba, size: u32) -> SvgViewer {
    SvgViewer::new(Bytes::from_static(template.as_bytes()))
        .width(Size::px(size as f32))
        .height(Size::px(size as f32))
        .color(rgba(color))
        .show_loader(false)
}

fn rgba(color: vow_design::Rgba) -> Color {
    let [r, g, b, a] = color.to_rgba8();
    Color::from_argb(a, r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyboard_types::{Key, Modifiers};

    #[test]
    fn shift_h_is_accepted_as_secure_field_text() {
        assert!(secure_field_accepts_text(
            &Key::Character("H".into()),
            Modifiers::SHIFT
        ));
    }

    #[test]
    fn ctrl_a_is_rejected_as_secure_field_text() {
        assert!(!secure_field_accepts_text(
            &Key::Character("a".into()),
            Modifiers::CONTROL
        ));
    }
}

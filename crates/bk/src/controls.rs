pub use crate::scroll::{ScrollController, ScrollView, use_scroll_controller};

use freya_components::input::{
    Input, InputColorsThemePartial, InputLayoutThemePartial, InputValidator,
};
use freya_components::theming::macros::Preference;

use ctdesign::{BODY, Font, Rgba, TEXT_FIELD, TEXT_LINE_HEIGHT};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, CursorPoint, Direction, Gaps, Position, Size};

use crate::ProbeExt;
use crate::backdrop::Tint;
use crate::choices::use_choice;
use crate::context_menu::{Spelling, edit_menu};
use crate::controlled::commit;
use crate::i18n::text;
use crate::interaction::use_interaction;
use crate::level;
use crate::password::PasswordField;
use crate::spelling::{Squiggles, char_index, dictionary, known, misspelled, utf16_range};
use crate::style::{Palette, with_font};
use crate::styles::{
    Axis, ButtonStyle, CheckboxStyle, ControlState, Edge, LayoutDirection, Paints,
    SearchFieldStyle, SecureFieldStyle, SegmentedControlStyle, ShadowStyle, Shape, ShapeStyle,
    SliderStyle, StateValue, SwitchStyle, TextFieldStyle, Theme, ToolGroupStyle, resolve,
};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;
use zeroize::Zeroizing;

#[derive(Clone, PartialEq)]
pub struct SecureField {
    field: State<PasswordField>,
    label: &'static str,
    width: Size,
    auto_focus: bool,
    on_submit: Option<Callback<Zeroizing<String>, ()>>,
    style: Theme,
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
            style: Theme::tengen(),
            key: DiffKey::None,
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(SecureFieldStyle) -> SecureFieldStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
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
        let interaction = use_interaction();
        let style: SecureFieldStyle = resolve(&self.style);
        let state = ControlState {
            hovered: interaction.hovered(),
            pressed: interaction.pressed(),
            focused: interaction.focused(),
            disabled: false,
        };
        let paints = Paints::current(state, style.effect);
        let mut field = self.field;
        let on_submit = self.on_submit.clone();
        let masked = field.read().masked();
        let margin_y = (style.height - style.font.line_height as f32) / 2.0;
        let radius =
            style
                .shape
                .corner_radius(style.height, style.height, LayoutDirection::LeftToRight);
        let element = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::PasswordInput)
            .a11y_alt(self.label)
            .a11y_auto_focus(self.auto_focus)
            .width(self.width.clone())
            .height(Size::px(style.height))
            .padding(Gaps::new(
                margin_y,
                style.padding_x,
                margin_y,
                style.padding_x,
            ))
            .corner_radius(radius)
            .background(paints.fill(&style.background))
            .map(paints.blur(&style.background), Rect::blur)
            .map(style.border.as_ref(), |el, border| {
                el.border(inner_border(&paints, border))
            })
            .on_key_down(move |event: Event<KeyboardEventData>| {
                let secret = field.write().handle_key(&event.key, event.modifiers);
                if let Some(secret) = secret {
                    event.prevent_default();
                    if let Some(callback) = &on_submit {
                        callback.call(secret);
                    }
                }
            })
            .child(crate::style::text(
                masked,
                style.font,
                paints.color(style.text.resolve(state)),
            ));
        interaction.focus_ring(dimmed(element, style.opacity), radius.top_left)
    }
}

#[derive(PartialEq)]
pub struct ToolGroup {
    children: Vec<Element>,
    probe: Option<&'static str>,
    style: Theme,
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
            style: Theme::tengen(),
            key: DiffKey::None,
        }
    }

    pub fn style(mut self, restyle: impl Fn(ToolGroupStyle) -> ToolGroupStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
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
        crate::compact::use_toolbar_items(true);
        let style: ToolGroupStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default(), style.effect);
        let vertical = style.axis == Axis::Vertical;
        let slot = style.item_size;
        let (along, across) = style.separator_size;
        let (separator_width, separator_height) = if vertical {
            (across, along)
        } else {
            (along, across)
        };

        let mut items: Vec<Element> = Vec::with_capacity(self.children.len());
        let last = self.children.len().saturating_sub(1);
        for (i, child) in self.children.iter().cloned().enumerate() {
            items.push(
                rect()
                    .width(Size::px(slot))
                    .height(Size::px(slot))
                    .center()
                    .child(child)
                    .into_element(),
            );
            if let Some(separator) = style.separator.as_ref().filter(|_| i != last) {
                items.push(
                    rect()
                        .width(Size::px(separator_width))
                        .height(Size::px(separator_height))
                        .background(paints.fill(separator))
                        .into_element(),
                );
            }
        }

        let padding = style.padding;
        let root = if vertical {
            rect()
                .direction(Direction::Vertical)
                .width(Size::px(slot))
                .padding(Gaps::new(padding, 0.0, padding, 0.0))
        } else {
            rect()
                .direction(Direction::Horizontal)
                .height(Size::px(slot))
                .padding(Gaps::new(0.0, padding, 0.0, padding))
        };
        let root = root
            .cross_align(Alignment::Center)
            .children(items)
            .map(self.probe, ProbeExt::probe)
            .background(paints.fill(&style.background))
            .map(paints.blur(&style.background), Rect::blur)
            .map(style.border.as_ref(), |el, border| {
                el.border(inner_border(&paints, border))
            })
            .corner_radius(
                style
                    .shape
                    .corner_radius(slot, slot, LayoutDirection::LeftToRight),
            );
        dimmed(shadowed(root, &style.shadows), style.opacity)
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
    style: Theme,
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
            style: Theme::tengen(),
            key: DiffKey::None,
        }
    }

    pub fn tint(self, tint: Tint) -> Self {
        self.style(move |style| SliderStyle {
            fill: StateValue::new(tint.into()),
            ..style
        })
    }

    pub fn style(mut self, restyle: impl Fn(SliderStyle) -> SliderStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
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
        let interaction = use_interaction();
        let style: SliderStyle = resolve(&self.style);
        let paints = Paints::current(
            ControlState {
                hovered: interaction.hovered(),
                pressed: interaction.pressed(),
                focused: interaction.focused(),
                disabled: self.disabled,
            },
            style.effect,
        );
        let value = crate::controlled::use_controlled(&self.value);
        let origin = use_state(CursorPoint::default);
        let on_change = self.on_change.clone();

        let vertical = style.axis == Axis::Vertical;
        let right_to_left = style.layout_direction == LayoutDirection::RightToLeft;
        let reversed = vertical || right_to_left;
        let length = style.width;
        let (knob, knob_thickness) = style.knob_size;
        let track = style.track_height;
        let tick = style.tick_size;
        let steps = self.steps;
        let segments = steps.saturating_sub(1).max(1) as f32;
        let run = length - knob;
        let current = *value.read();
        let knob_at =
            run * crate::animation::use_animated(current as f32 / segments, style.animation);

        let set_from_point = move |point: CursorPoint| {
            let along = if vertical { point.y } else { point.x };
            let at = (along as f32 - knob / 2.0).clamp(0.0, run);
            let at = if reversed { run - at } else { at };
            let step = ((at / run) * segments).round() as usize;
            step.min(steps - 1)
        };

        let update = move |new_value: usize| commit(value, &on_change, new_value);

        let labels_first =
            matches!(style.labels, Some(Edge::Top | Edge::Leading)) != (vertical && right_to_left);
        let label_line = style.label_font.size as f32 * TEXT_LINE_HEIGHT;
        let (width, height, direction) = match (vertical, style.labels) {
            (true, _) => (Size::auto(), Size::px(length), Direction::Horizontal),
            (false, Some(_)) => (
                Size::px(length),
                Size::px(knob_thickness + style.label_gap + label_line),
                Direction::Vertical,
            ),
            (false, None) => (
                Size::px(length),
                Size::px(knob_thickness),
                Direction::Vertical,
            ),
        };
        let step_name = self.labels.get(current).copied();
        let mut root = interaction
            .attach(rect(), !self.disabled)
            .width(width)
            .height(height)
            .direction(direction)
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
            let forward = if right_to_left && !vertical {
                NamedKey::ArrowLeft
            } else {
                NamedKey::ArrowRight
            };
            root = level::on_drag(root, interaction, origin, set_from_point, update.clone())
                .on_key_down(move |event: Event<KeyboardEventData>| match event.key {
                    Key::Named(NamedKey::ArrowUp) => update((current + 1).min(steps - 1)),
                    Key::Named(key) if key == forward => update((current + 1).min(steps - 1)),
                    Key::Named(
                        NamedKey::ArrowLeft | NamedKey::ArrowRight | NamedKey::ArrowDown,
                    ) => update(current.saturating_sub(1)),
                    _ => {}
                });
        }

        let part = |main: f32,
                    size: f32,
                    parent: f32,
                    cross: f32,
                    thickness: f32,
                    shape: Shape,
                    paint: &StateValue<ShapeStyle>| {
            let main = if reversed { parent - main - size } else { main };
            let (left, top, width, height) = if vertical {
                (cross, main, thickness, size)
            } else {
                (main, cross, size, thickness)
            };
            rect()
                .position(Position::new_absolute().left(left).top(top))
                .width(Size::px(width))
                .height(Size::px(height))
                .corner_radius(shape.corner_radius(width, height, style.layout_direction))
                .background(paints.fill(paint))
                .map(paints.blur(paint), Rect::blur)
        };
        let dim = |el: Rect, opacity: f32| el.maybe(opacity < 1.0, |el| el.opacity(opacity));
        // freya 0.4.3 dims each element on its own (render_pipeline.rs): a dimmed knob would show the fill.
        let parts_opacity = if self.disabled {
            style.opacity * style.disabled_opacity
        } else {
            style.opacity
        };

        let track_rect = dim(
            part(
                knob / 2.0,
                run,
                length,
                (knob_thickness - track) / 2.0,
                track,
                style.track_shape,
                &style.track,
            ),
            parts_opacity,
        );
        let fill_rect = part(
            -knob / 2.0,
            knob_at + knob / 2.0,
            run,
            0.0,
            track,
            style.track_shape,
            &style.fill,
        );
        let tick_cross = if labels_first {
            (knob_thickness - track) / 2.0 - style.tick_gap - tick
        } else {
            (knob_thickness + track) / 2.0 + style.tick_gap
        };
        let ticks = (0..steps).map(|i| {
            let x = knob / 2.0 + run * (i as f32 / segments);
            dim(
                part(
                    x - tick / 2.0,
                    tick,
                    length,
                    tick_cross,
                    tick,
                    style.tick_shape,
                    &style.tick,
                ),
                parts_opacity,
            )
            .into_element()
        });
        let knob_rect = style
            .knob_shadows
            .iter()
            .fold(
                part(
                    knob_at,
                    knob,
                    length,
                    0.0,
                    knob_thickness,
                    style.knob_shape,
                    &style.knob,
                ),
                |el, shadow| el.shadow(*shadow),
            )
            .map(style.knob_border.as_ref(), |el, (paint, width)| {
                el.border(
                    Border::new()
                        .width(*width)
                        .alignment(BorderAlignment::Inner)
                        .fill(color(paints.color(paint))),
                )
            })
            .layer(level::KNOB_ABOVE_FILL);
        let knob_rect = dim(knob_rect, style.opacity);
        let (band_width, band_height) = if vertical {
            (knob_thickness, length)
        } else {
            (length, knob_thickness)
        };
        let band = rect()
            .width(Size::px(band_width))
            .height(Size::px(band_height))
            .children(ticks)
            .child(track_rect.child(fill_rect))
            .child(knob_rect);

        let labels_row = style.labels.map(|_| {
            let names = self.labels.iter().take(steps).map(|text| {
                let label_node = with_font(label().text(*text), style.label_font)
                    .max_lines(1)
                    .color(paints.fill(&style.label));
                rect().child(label_node).into_element()
            });
            let names: Vec<Element> = if reversed {
                names.rev().collect()
            } else {
                names.collect()
            };
            let gap = style.label_gap;
            let margin = match (vertical, labels_first) {
                (false, false) => Gaps::new(gap, 0.0, 0.0, 0.0),
                (false, true) => Gaps::new(0.0, 0.0, gap, 0.0),
                (true, false) => Gaps::new(0.0, 0.0, 0.0, gap),
                (true, true) => Gaps::new(0.0, gap, 0.0, 0.0),
            };
            let row = if vertical {
                rect()
                    .height(Size::px(length))
                    .direction(Direction::Vertical)
            } else {
                rect()
                    .width(Size::px(length))
                    .height(Size::px(label_line))
                    .direction(Direction::Horizontal)
            };
            dim(
                row.main_align(Alignment::SpaceBetween)
                    .margin(margin)
                    .children(names),
                parts_opacity,
            )
        });

        root = if labels_first {
            root.map(labels_row, |el, row| el.child(row)).child(band)
        } else {
            root.child(band).map(labels_row, |el, row| el.child(row))
        };
        root = interaction.focus_ring(root, knob / 2.0);

        root.map(self.probe, ProbeExt::probe)
    }
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
    disabled: bool,
    on: Option<bool>,
    width: Size,
    on_press: Option<Callback<(), ()>>,
    style: Theme,
    key: DiffKey,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
            on: None,
            width: Size::auto(),
            on_press: None,
            style: Theme::tengen(),
            key: DiffKey::None,
        }
    }

    pub fn toggled(mut self, on: bool) -> Self {
        self.on = Some(on);
        self
    }

    pub fn style(mut self, restyle: impl Fn(ButtonStyle) -> ButtonStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn size(self, size: ControlSize) -> Self {
        self.style(move |style| style.sized(size))
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
        let interaction = use_interaction();
        let enabled = !self.disabled;
        let style: ButtonStyle = resolve(&self.style);
        let style = if self.on == Some(true) {
            style.bordered_prominent()
        } else {
            style
        };
        let paints = Paints::current(
            ControlState {
                hovered: interaction.hovered(),
                pressed: interaction.pressed(),
                focused: interaction.focused(),
                disabled: self.disabled,
            },
            style.effect,
        );
        let pressed = enabled && interaction.pressed();
        let scale = crate::animation::use_animated(
            if pressed { style.pressed_scale } else { 1.0 },
            style.animation,
        );
        let on_press = self.on_press.clone();
        let radius =
            style
                .shape
                .corner_radius(style.height, style.height, LayoutDirection::LeftToRight);
        let opacity = if enabled {
            style.opacity
        } else {
            style.opacity * style.disabled_opacity
        };
        let font = style.font;

        let button = interaction
            .attach(rect(), enabled)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label.clone())
            .width(self.width.clone())
            .height(Size::px(style.height))
            .padding(Gaps::new(0.0, style.padding_x, 0.0, style.padding_x))
            .corner_radius(radius)
            .center()
            .background(paints.fill(&style.background))
            .map(paints.blur(&style.background), Rect::blur)
            .map(style.border.as_ref(), |el, border| {
                el.border(inner_border(&paints, border))
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
            .maybe(scale != 1.0, |el| el.scale(scale))
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
                    .font_family(font.family)
                    .font_size(font.size as f32)
                    .font_weight(font.weight as i32)
                    .max_lines(1)
                    .color(paints.fill(&style.label)),
            );
        let button = dimmed(shadowed(button, &style.shadows), opacity);
        interaction.focus_ring(button, radius.top_left)
    }
}

#[derive(PartialEq)]
pub struct Switch {
    label: &'static str,
    on: bool,
    disabled: bool,
    on_toggle: Option<Callback<bool, ()>>,
    probe: Option<&'static str>,
    style: Theme,
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
            style: Theme::tengen(),
            key: DiffKey::None,
        }
    }

    pub fn tint(self, tint: Tint) -> Self {
        self.style(move |style| SwitchStyle {
            fill: StateValue::new(tint.into()),
            ..style
        })
    }

    pub fn style(mut self, restyle: impl Fn(SwitchStyle) -> SwitchStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
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
        let on = crate::controlled::use_controlled(&self.on);
        let interaction = use_interaction();
        let style: SwitchStyle = resolve(&self.style);
        let state = ControlState {
            hovered: interaction.hovered(),
            pressed: interaction.pressed(),
            focused: interaction.focused(),
            disabled: self.disabled,
        };
        let paints = Paints::current(state, style.effect);
        let is_on = *on.read();
        let progress = crate::animation::use_animated(f32::from(u8::from(is_on)), style.animation);

        let disabled = self.disabled;
        let on_toggle = self.on_toggle.clone();
        let direction = style.layout_direction;
        let (width, height) = (style.width, style.height);
        let (knob_width, knob_height) = style.knob_size;
        let travel = width - knob_width - 2.0 * style.inset;
        let along = if direction == LayoutDirection::RightToLeft {
            1.0 - progress
        } else {
            progress
        };
        let opacity = if disabled {
            style.opacity * style.disabled_opacity
        } else {
            style.opacity
        };

        let knob = rect()
            .position(
                Position::new_absolute()
                    .left(style.inset + travel * along)
                    .top((height - knob_height) / 2.0),
            )
            .width(Size::px(knob_width))
            .height(Size::px(knob_height))
            .corner_radius(
                style
                    .knob_shape
                    .corner_radius(knob_width, knob_height, direction),
            )
            .background(paints.fill(&style.knob))
            .map(paints.blur(&style.knob), Rect::blur)
            .map(style.knob_border.as_ref(), |el, border| {
                el.border(inner_border(&paints, border))
            });
        let track_paint = if is_on { &style.fill } else { &style.track };
        let radius = style.track_shape.corner_radius(width, height, direction);
        let track = rect()
            .width(Size::px(width))
            .height(Size::px(height))
            .corner_radius(radius)
            .background(paints.fill(track_paint))
            .map(paints.blur(track_paint), Rect::blur)
            .map(
                style.track_border.as_ref().filter(|_| !is_on),
                |el, border| el.border(inner_border(&paints, border)),
            )
            .child(shadowed(knob, &style.knob_shadows));
        let track = interaction.focus_ring(dimmed(track, opacity), radius.top_left);
        let control = match style.label_position {
            None => track,
            Some(edge) => {
                let text = with_font(label().text(self.label), style.label_font)
                    .color(paints.fill(&style.label));
                labeled(
                    rect(),
                    track,
                    dimmed_label(text, opacity),
                    edge,
                    direction,
                    style.label_gap,
                )
            }
        };

        interaction
            .attach(control, !disabled)
            .a11y_role(AccessibilityRole::Switch)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_toggled(is_on.into()))
            .on_press(move |_| {
                if !disabled {
                    let next = !*on.peek();
                    commit(on, &on_toggle, next);
                }
            })
            .map(self.probe, ProbeExt::probe)
    }
}

#[derive(PartialEq)]
pub struct SegmentedControl {
    label: &'static str,
    segments: Vec<&'static str>,
    selected: usize,
    on_change: Option<Callback<usize, ()>>,
    style: Theme,
    key: DiffKey,
}

impl SegmentedControl {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            segments: Vec::new(),
            selected: 0,
            on_change: None,
            style: Theme::tengen(),
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

    pub fn style(
        mut self,
        restyle: impl Fn(SegmentedControlStyle) -> SegmentedControlStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
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
        let style: SegmentedControlStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default(), style.effect);
        let choice = use_choice(Some(self.selected), self.on_change.clone());
        let keyed = use_state(|| false);
        let segments = self.segments.iter().enumerate().map(|(index, text)| {
            Segment {
                text,
                active: choice.is(index),
                take_focus: choice.is(index) && *keyed.read(),
                on_select: choice.select(index),
                style: self.style.clone(),
            }
            .into_element()
        });
        let count = self.segments.len() as f32;
        let (direction, height) = match style.axis {
            Axis::Horizontal => (Direction::Horizontal, style.segment_height),
            Axis::Vertical => (
                Direction::Vertical,
                count * style.segment_height + (count - 1.0).max(0.0) * style.gap,
            ),
        };
        let height = height + 2.0 * style.padding;

        let group = rect()
            .a11y_role(AccessibilityRole::TabList)
            .a11y_alt(self.label)
            .width(Size::fill())
            .direction(direction)
            .content(Content::flex())
            .padding(Gaps::new_all(style.padding))
            .spacing(style.gap)
            .corner_radius(
                style
                    .shape
                    .corner_radius(height, height, LayoutDirection::LeftToRight),
            )
            .background(paints.fill(&style.background))
            .map(paints.blur(&style.background), Rect::blur)
            .map(style.border.as_ref(), |el, border| {
                el.border(inner_border(&paints, border))
            })
            .children(segments);
        choice.arrow_keys(dimmed(group, style.opacity), self.segments.len(), keyed)
    }
}

#[derive(PartialEq)]
struct Segment {
    text: &'static str,
    active: bool,
    take_focus: bool,
    on_select: Callback<(), ()>,
    style: Theme,
}

impl Component for Segment {
    fn render(&self) -> impl IntoElement {
        let style: SegmentedControlStyle = resolve(&self.style);
        let interaction = use_interaction();
        let paints = Paints::current(
            ControlState {
                hovered: interaction.hovered(),
                pressed: interaction.pressed(),
                focused: interaction.focused(),
                disabled: false,
            },
            style.effect,
        );
        crate::choices::take_focus_when(interaction, self.take_focus);
        let active = self.active;
        let on_select = self.on_select.clone();
        let (paint, ink) = if active {
            (&style.selected, &style.selected_label)
        } else {
            (&style.segment, &style.label)
        };
        let font = style.font;
        let height = style.segment_height;
        let radius =
            style
                .segment_shape
                .corner_radius(height, height, LayoutDirection::LeftToRight);

        let segment = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Tab)
            .a11y_alt(self.text)
            .a11y_builder(move |node| node.set_selected(active))
            .width(match style.axis {
                Axis::Horizontal => Size::flex(1.0),
                Axis::Vertical => Size::fill(),
            })
            .height(Size::px(height))
            .corner_radius(radius)
            .center()
            .on_press(move |_| on_select.call(()))
            .child(
                rect()
                    .padding(Gaps::new(
                        0.0,
                        style.segment_padding_x,
                        0.0,
                        style.segment_padding_x,
                    ))
                    .child(
                        label()
                            .text(self.text)
                            .font_family(font.family)
                            .font_size(font.size as f32)
                            .font_weight(font.weight as i32)
                            .color(paints.fill(ink)),
                    ),
            )
            .background(paints.fill(paint))
            .map(paints.blur(paint), Rect::blur);
        let segment = if active {
            shadowed(segment, &style.selected_shadows)
        } else {
            segment
        };
        interaction.focus_ring(segment, radius.top_left)
    }
}

#[derive(PartialEq)]
pub struct TextField {
    value: State<String>,
    placeholder: &'static str,
    width: Size,
    auto_focus: bool,
    style: Theme,
    key: DiffKey,
}

impl TextField {
    pub fn new(value: State<String>, placeholder: &'static str) -> Self {
        Self {
            value,
            placeholder,
            width: Size::fill(),
            auto_focus: false,
            style: Theme::tengen(),
            key: DiffKey::None,
        }
    }

    pub fn style(mut self, restyle: impl Fn(TextFieldStyle) -> TextFieldStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
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
        let style: TextFieldStyle = resolve(&self.style);
        let field = use_hook(AccessibilityId::new_unique);
        let focus = use_focus(field);
        let mut hovered = use_state(|| false);
        let state = ControlState {
            hovered: *hovered.read(),
            focused: !matches!(*focus.read(), Focus::Not),
            ..ControlState::default()
        };
        let paints = Paints::current(state, style.effect);
        let ink = |paint: &StateValue<ShapeStyle>| paints.color(paint.resolve(state));
        let margin_y = (style.height - style.font.line_height as f32) / 2.0;
        let radius =
            style
                .shape
                .corner_radius(style.height, style.height, LayoutDirection::LeftToRight);
        let input = input(
            self.value,
            self.placeholder,
            [
                ink(&style.background),
                ink(&style.border),
                ink(&style.border),
                ink(&style.text),
                ink(&style.placeholder),
            ],
            radius,
            Gaps::new(margin_y, style.padding_x, margin_y, style.padding_x),
        )
        .auto_focus(self.auto_focus);
        let mut index = use_state(|| None);
        let holder = use_state(ParagraphHolder::default);
        let padding_x = style.padding_x;
        let text_left = use_hook(|| std::rc::Rc::new(std::cell::Cell::new(padding_x)));
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
            dimmed(
                text_style(rect().width(self.width.clone()), style.font),
                style.opacity,
            ),
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
                .map(paints.blur(&style.background), |el, blur| {
                    el.corner_radius(radius).blur(blur)
                })
                .on_pointer_enter(move |_| hovered.set(true))
                .on_pointer_leave(move |_| hovered.set(false))
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
                        color: ink(&style.misspelling),
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
    style: Theme,
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
            style: Theme::tengen(),
            key: DiffKey::None,
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(SearchFieldStyle) -> SearchFieldStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
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
        let style: SearchFieldStyle = resolve(&self.style);
        let mut value = self.value;
        let padding = style.padding_x;
        let icon = style.icon_size;
        let gap = style.icon_gap;
        let height = style.height;
        let a11y_id = use_hook(|| self.a11y_id.unwrap_or_else(AccessibilityId::new_unique));
        let focus = use_focus(a11y_id);
        let mut hovered = use_state(|| false);
        let state = ControlState {
            hovered: *hovered.read(),
            focused: !matches!(*focus.read(), Focus::Not),
            ..ControlState::default()
        };
        let paints = Paints::current(state, style.effect);
        let ink = |paint: &StateValue<ShapeStyle>| paints.color(paint.resolve(state));
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
        let margin_y = (height - style.font.line_height as f32) / 2.0;
        let radius = style
            .shape
            .corner_radius(height, height, LayoutDirection::LeftToRight);
        let mut input = input(
            value,
            self.placeholder,
            [
                ink(&style.background),
                ink(&style.border),
                ink(&style.border),
                ink(&style.text),
                ink(&style.placeholder),
            ],
            radius,
            Gaps::new(margin_y, padding + icon + gap, margin_y, leading),
        )
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
                    .child(symbol(Symbol::XMark, icon, ink(&style.clear_icon))),
            );
        }

        edit_menu(
            dimmed(
                text_style(rect().width(self.width.clone()), style.font),
                style.opacity,
            )
            .map(paints.blur(&style.background), |el, blur| {
                el.corner_radius(radius).blur(blur)
            })
            .on_pointer_enter(move |_| hovered.set(true))
            .on_pointer_leave(move |_| hovered.set(false)),
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
                        .top((height - icon) / 2.0),
                )
                .child(symbol(Symbol::MagnifyingGlass, icon, ink(&style.icon))),
        )
        .maybe(!shown_tokens.is_empty(), |el| {
            el.child(
                rect()
                    .position(
                        Position::new_absolute()
                            .left(padding + icon + gap)
                            .top((height - ctdesign::TOKEN_FIELD.token_height) / 2.0),
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

pub(crate) fn text_style(element: Rect, font: Font) -> Rect {
    element
        .font_family(font.family)
        .font_size(font.size as f32)
        .font_weight(font.weight as i32)
}

pub(crate) fn themed_input(
    value: State<String>,
    placeholder: &'static str,
    background: Rgba,
    border: Rgba,
    radius: f32,
    padding_x: f32,
) -> Input {
    let palette = Palette::current();
    let margin_y = (TEXT_FIELD.height - BODY.line_height as f32) / 2.0;
    input(
        value,
        placeholder,
        [
            background,
            border,
            palette.accent,
            palette.label,
            palette.secondary_label,
        ],
        CornerRadius::new_all(radius),
        Gaps::new(margin_y, padding_x, margin_y, padding_x),
    )
}

fn input(
    value: State<String>,
    placeholder: &'static str,
    [background, border, focus_border, text, placeholder_color]: [Rgba; 5],
    corner_radius: CornerRadius,
    inner_margin: Gaps,
) -> Input {
    let (generation, mut typed) = use_input_generation(value);
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
            focus_border_fill: color(focus_border),
            color: color(text),
            placeholder_color: color(placeholder_color),
        })
        .theme_layout(InputLayoutThemePartial {
            corner_radius: Some(Preference::Specific(corner_radius)),
            inner_margin: Some(Preference::Specific(inner_margin)),
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
    style: Theme,
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
            style: Theme::tengen(),
            key: DiffKey::None,
        }
    }

    pub fn tint(self, tint: Tint) -> Self {
        self.style(move |style| CheckboxStyle {
            fill: StateValue::new(tint.into()),
            ..style
        })
    }

    pub fn style(mut self, restyle: impl Fn(CheckboxStyle) -> CheckboxStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
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
        let checked = crate::controlled::use_controlled(&self.checked);
        let on_toggle = self.on_toggle.clone();
        let disabled = self.disabled;
        let label_text = self.label;
        let interaction = use_interaction();
        let style: CheckboxStyle = resolve(&self.style);
        let state = ControlState {
            hovered: interaction.hovered(),
            pressed: interaction.pressed(),
            focused: interaction.focused(),
            disabled,
        };
        let paints = Paints::current(state, style.effect);
        let is_checked = *checked.read();
        let opacity = if disabled {
            style.opacity * style.disabled_opacity
        } else {
            style.opacity
        };

        let size = style.box_size;
        let radius = style
            .box_shape
            .corner_radius(size, size, style.layout_direction);
        let paint = if is_checked {
            &style.fill
        } else {
            &style.box_background
        };
        let checkbox = rect()
            .width(Size::px(size))
            .height(Size::px(size))
            .corner_radius(radius)
            .background(paints.fill(paint))
            .map(paints.blur(paint), Rect::blur)
            .map(
                style.box_border.as_ref().filter(|_| !is_checked),
                |el, border| el.border(inner_border(&paints, border)),
            )
            .center()
            .maybe(is_checked, |el| {
                el.child(symbol(
                    Symbol::Checkmark,
                    style.mark_size,
                    paints.color(style.mark.resolve(state)),
                ))
            });
        let checkbox = interaction.focus_ring(
            dimmed(shadowed(checkbox, &style.box_shadows), opacity),
            radius.top_left,
        );

        let row = interaction
            .attach(rect(), !disabled)
            .width(self.width.clone())
            .a11y_role(AccessibilityRole::CheckBox)
            .a11y_alt(label_text)
            .a11y_builder(move |node| node.set_toggled(is_checked.into()))
            .on_press(move |_| {
                if !disabled {
                    let next = !*checked.peek();
                    commit(checked, &on_toggle, next);
                }
            });
        let root = match style.label_position {
            None => row
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center)
                .child(checkbox),
            Some(edge) => {
                let text = with_font(label().text(label_text), style.label_font)
                    .color(paints.fill(&style.label));
                labeled(
                    row,
                    checkbox,
                    dimmed_label(text, opacity),
                    edge,
                    style.layout_direction,
                    style.label_gap,
                )
            }
        };

        root.map(self.probe, ProbeExt::probe)
    }
}

fn inner_border(paints: &Paints, (paint, width): &(ShapeStyle, f32)) -> Border {
    Border::new()
        .width(*width)
        .alignment(BorderAlignment::Inner)
        .fill(color(paints.color(paint)))
}

fn shadowed(element: Rect, shadows: &[ShadowStyle]) -> Rect {
    shadows
        .iter()
        .fold(element, |element, shadow| element.shadow(*shadow))
}

fn dimmed(element: Rect, opacity: f32) -> Rect {
    element.maybe(opacity < 1.0, |element| element.opacity(opacity))
}

fn dimmed_label(text: Label, opacity: f32) -> Element {
    if opacity < 1.0 {
        rect().opacity(opacity).child(text).into_element()
    } else {
        text.into_element()
    }
}

fn labeled(
    row: Rect,
    control: Rect,
    label: Element,
    edge: Edge,
    direction: LayoutDirection,
    gap: f32,
) -> Rect {
    let label_first = match edge {
        Edge::Top => true,
        Edge::Bottom => false,
        Edge::Leading => direction == LayoutDirection::LeftToRight,
        Edge::Trailing => direction == LayoutDirection::RightToLeft,
    };
    let row = row
        .direction(if matches!(edge, Edge::Top | Edge::Bottom) {
            Direction::Vertical
        } else {
            Direction::Horizontal
        })
        .cross_align(Alignment::Center)
        .spacing(gap);
    if label_first {
        row.child(label).child(control)
    } else {
        row.child(control).child(label)
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

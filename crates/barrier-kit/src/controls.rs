//! BarrierKit controls with macOS-style names, built on `freya-components`
//! or on the core `rect`/`label` primitives (ADR 0034). Measurements and
//! colours come from the theme and `vow-design`.

pub use freya_components::button::Button;
pub use freya_components::input::Input as TextField;
pub use freya_components::menu::Menu;
pub use freya_components::popup::Popup as Popover;
pub use freya_components::scrollviews::ScrollView;
pub use freya_components::segmented_button::SegmentedButton as SegmentedControl;

pub use freya_components::switch::Switch;

use freya_components::svg_viewer::SvgViewer;

use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Gaps, Position, Size};
use vow_design::{CAPTION, CHECKBOX, Rgba, SETTINGS, SLIDER, TEXT, TEXT_LINE_HEIGHT, TOOLGROUP};

use crate::ProbeExt;
use crate::password::PasswordField;
use crate::platform::inserts_text;

/// True when a key event should be inserted into a [`SecureField`].
fn secure_field_accepts_text(key: &Key, modifiers: Modifiers) -> bool {
    matches!(key, Key::Character(_)) && inserts_text(modifiers)
}

/// A masked password field that keeps the same zeroizing buffer as the lock
/// screen (`password::PasswordField`). The masked text is the only value the
/// accessibility tree and the canvas see; the secret is never logged.
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

/// Visual style for a [`ToolGroup`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolGroupStyle {
    /// Capsule with a tinted background and vertical separators.
    #[default]
    Island,
    /// Transparent background with vertical separators only.
    Flat,
}

/// A macOS-style toolbar capsule group.
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

    /// Use the flat style (no background capsule).
    pub fn flat(mut self) -> Self {
        self.style = ToolGroupStyle::Flat;
        self
    }

    /// Use the island style (background capsule without shadow).
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

/// A Tengen toolbar icon button.
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
    /// Create a tool button with a mandatory accessibility label and an SVG icon.
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

/// A stepped slider with a fixed number of positions.
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
    /// Create a slider with `steps` positions (0..steps-1).
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

    /// Set the current position.
    pub fn value(mut self, value: usize) -> Self {
        self.value = value.min(self.steps.saturating_sub(1));
        self
    }

    /// Set the labels shown under each step.
    pub fn labels(mut self, labels: &'static [&'static str]) -> Self {
        self.labels = labels;
        self
    }

    /// Disable interaction and fade the control.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Callback invoked when the value changes.
    pub fn on_change(mut self, callback: impl Into<Callback<usize, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }

    /// Set accessibility ids for the knob and each step label.
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
        let knob_shadow = rgba(Rgba::rgba8(0x0d, 0x12, 0x19, 0.3));

        let mut root = rect()
            .width(Size::px(width))
            .height(Size::px(knob + label_gap + CAPTION.size as f32 * TEXT_LINE_HEIGHT))
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
            .position(
                Position::new_absolute()
                    .left(0.0)
                    .top((knob - track) / 2.0),
            )
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
            .shadow((0.0, 1.0, 3.0, 0.0, knob_shadow));
        if let Some(id) = self.knob_id {
            knob_rect = knob_rect.a11y_builder(move |node| node.set_author_id(id));
        }

        let mut labels: Vec<Element> = Vec::new();
        for (i, text) in self.labels.iter().enumerate().take(self.steps) {
            let color = if i == *value.read() {
                base.ink_strong
            } else {
                base.ink_muted
            };
            let mut label_node = label()
                .text(*text)
                .font_family(TEXT.family)
                .font_size(CAPTION.size as f32)
                .font_weight(CAPTION.weight as i32)
                .color(rgba(color));
            if let Some(id) = self.step_ids.get(i).copied() {
                label_node = label_node.a11y_builder(move |node| node.set_author_id(id));
            }
            labels.push(label_node.into_element());
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

/// A Tengen checkbox with a text label.
#[derive(PartialEq)]
pub struct Checkbox {
    label: &'static str,
    checked: bool,
    disabled: bool,
    on_toggle: Option<Callback<bool, ()>>,
    probe: Option<&'static str>,
    key: DiffKey,
}

impl Checkbox {
    /// Create a checkbox with the given label.
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            checked: false,
            disabled: false,
            on_toggle: None,
            probe: None,
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

        let toggle = {
            let mut checked = checked;
            let on_toggle = on_toggle;
            move |_| {
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
            checkbox = checkbox.child(
                svg_viewer(check_svg, base.on_accent, size as u32 - 4),
            );
        }

        let mut root = rect()
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .a11y_role(AccessibilityRole::CheckBox)
            .on_press(toggle)
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
                    .color(label_color),
            );

        if self.disabled {
            root = root.opacity(0.4);
        }

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

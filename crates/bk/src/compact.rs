use freya_core::prelude::*;
use std::rc::Rc;

use torin::prelude::{Alignment, Content, Direction, Gaps, Size};

use crate::controls::{dimmed, dimmed_label, inner_border, labeled, shadowed};
use crate::i18n::text;
use crate::interaction::{Interaction, use_interaction};
use crate::style::with_font;
use crate::styles::{
    Axis, ControlState, IconButtonStyle, LayoutDirection, Paints, StepperStyle, Theme,
    ToggleGroupStyle, ToolbarSearchStyle, resolve,
};
use crate::symbol::{Symbol, symbol};

#[derive(Clone, Copy)]
struct ToolbarItems(State<bool>);

// Tahoe toolbar items are glass capsules as tall as a tool group (macOS 26 UI kit: 36 × 36 in a 52 bar).
pub(crate) fn use_toolbar_items(in_toolbar: bool) {
    let mut items = use_hook(|| {
        let items = State::create(in_toolbar);
        provide_context(ToolbarItems(items));
        items
    });
    items.set_if_modified(in_toolbar);
}

pub(crate) fn in_toolbar() -> bool {
    try_consume_context::<ToolbarItems>().is_some_and(|items| *items.0.read())
}

fn control_state(interaction: Interaction, disabled: bool) -> ControlState {
    ControlState {
        hovered: interaction.hovered(),
        pressed: interaction.pressed(),
        focused: interaction.focused(),
        disabled,
    }
}

#[derive(PartialEq)]
pub struct IconButton {
    symbol: Symbol,
    label: &'static str,
    disabled: bool,
    on_press: Option<Callback<(), ()>>,
    style: Theme,
}

impl IconButton {
    pub fn new(symbol: Symbol, label: &'static str) -> Self {
        Self {
            symbol,
            label,
            disabled: false,
            on_press: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(IconButtonStyle) -> IconButtonStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
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
        let interaction = use_interaction();
        let style: IconButtonStyle = resolve(&self.style);
        let enabled = !self.disabled;
        let state = control_state(interaction, self.disabled);
        let paints = Paints::current(state, style.effect);
        let pressed = enabled && interaction.pressed();
        let scale = crate::animation::use_animated(
            if pressed { style.pressed_scale } else { 1.0 },
            style.animation,
        );
        let on_press = self.on_press.clone();
        let size = style.size;
        let radius = style
            .shape
            .corner_radius(size, size, LayoutDirection::LeftToRight);
        let opacity = if enabled {
            style.opacity
        } else {
            style.opacity * style.disabled_opacity
        };

        let button = interaction
            .attach(rect(), enabled)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label)
            .width(Size::px(size))
            .height(Size::px(size))
            .corner_radius(radius)
            .center()
            .background(paints.fill(&style.background))
            .map(paints.blur(&style.background), Rect::blur)
            .map(style.border.as_ref(), |el, border| {
                el.border(inner_border(&paints, border))
            })
            .maybe(scale != 1.0, |el| el.scale(scale))
            .map(on_press.filter(|_| enabled), |el, on_press| {
                el.on_press(move |_| on_press.call(()))
            })
            .child(symbol(
                self.symbol,
                style.icon_size,
                paints.color(style.icon.resolve(state)),
            ));
        interaction.focus_ring(
            dimmed(shadowed(button, &style.shadows), opacity),
            radius.top_left,
        )
    }
}

#[derive(PartialEq)]
pub struct Stepper {
    label: &'static str,
    value: i64,
    min: i64,
    max: i64,
    on_change: Option<Callback<i64, ()>>,
    style: Theme,
}

impl Stepper {
    pub fn new(label: &'static str, min: i64, max: i64) -> Self {
        Self {
            label,
            value: min,
            min,
            max,
            on_change: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(StepperStyle) -> StepperStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
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
        let interaction = use_interaction();
        let style: StepperStyle = resolve(&self.style);
        let state = control_state(interaction, false);
        let paints = Paints::current(state, style.effect);
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

        let up = (current < max).then(|| Rc::new(move || step_up(1)) as Rc<dyn Fn()>);
        let down = (current > min).then(|| Rc::new(move || step_down(-1)) as Rc<dyn Fn()>);
        let (width, height) = style.arrows_size;
        let radius = style
            .shape
            .corner_radius(width, height, LayoutDirection::LeftToRight);
        let arrows =
            interaction.focus_ring(stepper_arrows(&style, state, up, down), radius.top_left);

        let row = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::SpinButton)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_numeric_value(current as f64))
            .on_key_down(move |event: Event<KeyboardEventData>| match event.key {
                Key::Named(NamedKey::ArrowUp) => step_key(1),
                Key::Named(NamedKey::ArrowDown) => step_key(-1),
                _ => {}
            });
        match style.value_position {
            None => row
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center)
                .child(arrows),
            Some(edge) => {
                let text = with_font(label().text(current.to_string()), style.value_font)
                    .max_lines(1)
                    .color(paints.fill(&style.value));
                labeled(
                    row,
                    arrows,
                    dimmed_label(text, style.opacity),
                    edge,
                    style.layout_direction,
                    style.value_gap,
                )
            }
        }
    }
}

pub(crate) fn stepper_arrows(
    style: &StepperStyle,
    state: ControlState,
    up: Option<Rc<dyn Fn()>>,
    down: Option<Rc<dyn Fn()>>,
) -> Rect {
    let paints = Paints::current(state, style.effect);
    let arrow = |kind: Symbol, label: &'static str, press: Option<Rc<dyn Fn()>>| {
        let state = ControlState {
            disabled: press.is_none(),
            ..state
        };
        let ink = Paints::current(state, style.effect).color(style.chevron.resolve(state));
        rect()
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(label)
            .width(Size::fill())
            .height(Size::flex(1.0))
            .center()
            .maybe(press.is_some(), |el| {
                el.on_press(move |_| {
                    if let Some(press) = &press {
                        press()
                    }
                })
            })
            .child(symbol(kind, style.chevron_size, ink))
    };
    let (width, height) = style.arrows_size;
    let arrows = rect()
        .width(Size::px(width))
        .height(Size::px(height))
        .content(Content::flex())
        .corner_radius(
            style
                .shape
                .corner_radius(width, height, LayoutDirection::LeftToRight),
        )
        .background(paints.fill(&style.background))
        .map(paints.blur(&style.background), Rect::blur)
        .map(style.border.as_ref(), |el, border| {
            el.border(inner_border(&paints, border))
        })
        .child(arrow(Symbol::ChevronUp, text!("increase"), up))
        .child(
            rect()
                .width(Size::fill())
                .height(Size::px(style.separator_height))
                .background(paints.fill(&style.separator)),
        )
        .child(arrow(Symbol::ChevronDown, text!("decrease"), down));
    dimmed(shadowed(arrows, &style.shadows), style.opacity)
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
    style: Theme,
}

impl ToggleGroup {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            items: Vec::new(),
            on_toggle: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(ToggleGroupStyle) -> ToggleGroupStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
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
        let style: ToggleGroupStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default(), style.effect);
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
                    style: self.style.clone(),
                }
                .into_element()
            });
        let (item_width, item_height) = style.item_size;
        let (direction, thickness) = match style.axis {
            Axis::Horizontal => (Direction::Horizontal, item_height),
            Axis::Vertical => (Direction::Vertical, item_width),
        };
        let thickness = thickness + 2.0 * style.padding;

        let group = rect()
            .a11y_role(AccessibilityRole::Toolbar)
            .a11y_alt(self.label)
            .direction(direction)
            .padding(Gaps::new_all(style.padding))
            .spacing(style.gap)
            .corner_radius(style.shape.corner_radius(
                thickness,
                thickness,
                LayoutDirection::LeftToRight,
            ))
            .background(paints.fill(&style.background))
            .map(paints.blur(&style.background), Rect::blur)
            .map(style.border.as_ref(), |el, border| {
                el.border(inner_border(&paints, border))
            })
            .children(toggles);
        dimmed(group, style.opacity)
    }
}

#[derive(PartialEq)]
struct Toggle {
    text: &'static str,
    face: Face,
    label: &'static str,
    on: bool,
    on_press: Callback<(), ()>,
    style: Theme,
}

impl Component for Toggle {
    fn render(&self) -> impl IntoElement {
        let style: ToggleGroupStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state, style.effect);
        let on = self.on;
        let on_press = self.on_press.clone();
        let (paint, ink) = if on {
            (&style.selected, &style.selected_label)
        } else {
            (&style.item, &style.label)
        };
        let face = self.face;
        let font = match face {
            Face::Bold => style.bold_font,
            Face::Italic | Face::Underline => style.font,
        };
        let (width, height) = style.item_size;
        let radius = style
            .item_shape
            .corner_radius(width, height, LayoutDirection::LeftToRight);

        let toggle = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_toggled(on.into()))
            .width(Size::px(width))
            .height(Size::px(height))
            .corner_radius(radius)
            .center()
            .background(paints.fill(paint))
            .map(paints.blur(paint), Rect::blur)
            .on_press(move |_| on_press.call(()))
            .child(
                paragraph()
                    .font_family(font.family)
                    .font_size(font.size as f32)
                    .font_weight(font.weight as i32)
                    .font_slant(match face {
                        Face::Italic => FontSlant::Italic,
                        Face::Bold | Face::Underline => FontSlant::Upright,
                    })
                    .color(paints.fill(ink))
                    .span(Span::new(self.text).text_decoration(match face {
                        Face::Underline => TextDecoration::Underline,
                        Face::Bold | Face::Italic => TextDecoration::None,
                    })),
            );
        let toggle = if on {
            shadowed(toggle, &style.selected_shadows)
        } else {
            toggle
        };
        interaction.focus_ring(dimmed(toggle, style.opacity), radius.top_left)
    }
}

#[derive(PartialEq)]
pub struct ToolbarSearch {
    value: State<String>,
    placeholder: &'static str,
    collapsed: bool,
    style: Theme,
}

impl ToolbarSearch {
    pub fn new(value: State<String>, placeholder: &'static str) -> Self {
        Self {
            value,
            placeholder,
            collapsed: false,
            style: Theme::tengen(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(ToolbarSearchStyle) -> ToolbarSearchStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }
}

impl Component for ToolbarSearch {
    fn render(&self) -> impl IntoElement {
        let style: ToolbarSearchStyle = resolve(&self.style);
        let mut opened = use_state(|| false);
        let open = !self.collapsed || *opened.read() || !self.value.read().is_empty();
        if !open {
            return rect()
                .child(
                    IconButton::new(Symbol::MagnifyingGlass, self.placeholder)
                        .on_press(move |_| opened.set(true)),
                )
                .into_element();
        }
        rect()
            .width(Size::px(style.width))
            .child(
                crate::controls::SearchField::new(self.value, self.placeholder)
                    .auto_focus(self.collapsed)
                    .on_focus_change(move |focused: bool| {
                        if !focused {
                            opened.set(false);
                        }
                    }),
            )
            .into_element()
    }
}

#[derive(PartialEq)]
pub struct ToolbarOverflow {
    items: Vec<crate::menu::MenuItem>,
    on_activate: Option<Callback<String, ()>>,
    style: Theme,
}

impl ToolbarOverflow {
    pub fn new(items: Vec<crate::menu::MenuItem>) -> Self {
        Self {
            items,
            on_activate: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(IconButtonStyle) -> IconButtonStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn on_activate(mut self, callback: impl Into<Callback<String, ()>>) -> Self {
        self.on_activate = Some(callback.into());
        self
    }
}

impl Component for ToolbarOverflow {
    fn render(&self) -> impl IntoElement {
        let mut area = use_state(torin::prelude::Area::default);
        let items = self.items.clone();
        let on_activate = self.on_activate.clone();
        rect()
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .child(
                IconButton {
                    style: self.style.clone(),
                    ..IconButton::new(Symbol::ChevronDoubleRight, text!("more-items"))
                }
                .on_press(move |_| {
                    let anchor = *area.peek();
                    let on_activate = on_activate.clone();
                    crate::popup_button::open_popover_menu(
                        items.clone(),
                        crate::platform::PopupAnchor::BottomLeft,
                        (
                            anchor.origin.x.round() as i32,
                            anchor.origin.y.round() as i32,
                            anchor.size.width.round() as u32,
                            anchor.size.height.round() as u32,
                        ),
                        None,
                        move |id| {
                            if let Some(on_activate) = &on_activate {
                                on_activate.call(id);
                            }
                        },
                    );
                }),
            )
    }
}

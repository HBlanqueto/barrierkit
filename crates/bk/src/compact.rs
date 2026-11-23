use ctdesign::{ControlSize, TOOLGROUP};
use freya_core::prelude::*;
use std::borrow::Cow;
use std::rc::Rc;

use torin::prelude::{Alignment, Content, Direction, Gaps, Size};

use crate::controls::{dimmed, dimmed_label, labeled, opacity_when, shadowed};
use crate::elements::{Overrides, forward_overrides};
use crate::environment::{use_disabled, use_is_enabled};
use crate::i18n::text;
use crate::interaction::{control_state, use_interaction};
use crate::style::{text_style, with_font};
use crate::styles::Painted;
use crate::styles::{
    Axis, ControlState, IconButtonStyle, LayoutDirection, Paints, StepperStyle, Theme,
    ToggleGroupStyle, ToolbarSearchStyle, resolve,
};
use crate::symbol::{Symbol, stroked_symbol};
use crate::titlebar::WindowToolbarStyle;

#[derive(Clone, Copy)]
struct ToolbarItems(State<Option<WindowToolbarStyle>>);

// Toolbar items are glass capsules, 36 high in the unified bar, 24 in the compact one and 28 in the expanded one.
pub(crate) fn use_toolbar_items(style: Option<WindowToolbarStyle>) {
    let mut items = use_hook(|| {
        let items = State::create(style);
        provide_context(ToolbarItems(items));
        items
    });
    items.set_if_modified(style);
}

pub(crate) fn use_toolbar_items_or(style: WindowToolbarStyle) {
    use_hook(|| {
        if try_consume_context::<ToolbarItems>().is_none() {
            provide_context(ToolbarItems(State::create(Some(style))));
        }
    });
}

pub(crate) fn toolbar_style() -> Option<WindowToolbarStyle> {
    try_consume_context::<ToolbarItems>().and_then(|items| *items.0.read())
}

// The compact and expanded toolbars draw their items on a platter narrower than their slots: slot width, height, inset.
pub(crate) fn slim_toolbar_items() -> Option<(f32, f32, f32)> {
    match toolbar_style()? {
        WindowToolbarStyle::Unified => None,
        WindowToolbarStyle::UnifiedCompact => Some((
            TOOLGROUP.compact_width as f32,
            TOOLGROUP.compact_height as f32,
            TOOLGROUP.compact_inset as f32,
        )),
        WindowToolbarStyle::Expanded => Some((
            TOOLGROUP.expanded_width as f32,
            TOOLGROUP.expanded_height as f32,
            TOOLGROUP.expanded_inset as f32,
        )),
    }
}

#[derive(PartialEq)]
pub struct IconButton {
    symbol: Symbol,
    label: Cow<'static, str>,
    disabled: bool,
    on_press: Option<Callback<(), ()>>,
    style: Theme,
    overrides: Overrides,
}

impl IconButton {
    pub fn new(symbol: Symbol, label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            symbol,
            label: label.into(),
            disabled: false,
            on_press: None,
            style: Theme::tengen(),
            overrides: Overrides::default(),
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

forward_overrides!(IconButton);

impl Component for IconButton {
    fn render(&self) -> impl IntoElement {
        let interaction = use_interaction();
        let style: IconButtonStyle = resolve(&self.style);
        let disabled = use_disabled(self.disabled);
        let enabled = !disabled;
        let state = control_state(interaction, disabled);
        let paints = Paints::current(state, style.effect);
        let pressed = enabled && interaction.pressed();
        let scale = crate::animation::use_animated(
            if pressed { style.pressed_scale } else { 1.0 },
            style.animation,
        );
        let on_press = self.on_press.clone();
        let (width, height) = style.size;
        let radius = style
            .shape
            .corner_radius(width, height, LayoutDirection::LeftToRight);
        let opacity = opacity_when(disabled, style.opacity, style.disabled_opacity);

        let button = interaction
            .attach(rect(), enabled)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(&*self.label)
            .width(Size::px(width))
            .height(Size::px(height))
            .corner_radius(radius)
            .center()
            .painted(&paints, &style.background)
            .bordered(&paints, style.border.as_ref())
            .maybe(scale != 1.0, |el| el.scale(scale))
            .map(on_press.filter(|_| enabled), |el, on_press| {
                el.on_press(move |_| on_press.call(()))
            })
            .child(stroked_symbol(
                self.symbol,
                style.icon_size,
                style.stroke,
                paints.color(style.icon.resolve(state)),
            ));
        let root = interaction.focus_ring(
            dimmed(shadowed(button, &style.shadows), opacity),
            radius.top_left(),
        );
        self.overrides.apply(root)
    }
}

#[derive(PartialEq)]
pub struct Stepper {
    label: Cow<'static, str>,
    value: i64,
    min: i64,
    max: i64,
    disabled: bool,
    on_change: Option<Callback<i64, ()>>,
    style: Theme,
    overrides: Overrides,
}

impl Stepper {
    pub fn new(label: impl Into<Cow<'static, str>>, min: i64, max: i64) -> Self {
        Self {
            label: label.into(),
            value: min,
            min,
            max,
            disabled: false,
            on_change: None,
            style: Theme::tengen(),
            overrides: Overrides::default(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(StepperStyle) -> StepperStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn size(self, size: ControlSize) -> Self {
        self.style(move |style: StepperStyle| style.sized(size))
    }

    pub fn value(mut self, value: i64) -> Self {
        self.value = value.clamp(self.min, self.max);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, callback: impl Into<Callback<i64, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }
}

forward_overrides!(Stepper);

impl Component for Stepper {
    fn render(&self) -> impl IntoElement {
        let interaction = use_interaction();
        let style: StepperStyle = resolve(&self.style);
        let disabled = use_disabled(self.disabled);
        let state = control_state(interaction, disabled);
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

        let up = (!disabled && current < max).then(|| Rc::new(move || step_up(1)) as Rc<dyn Fn()>);
        let down =
            (!disabled && current > min).then(|| Rc::new(move || step_down(-1)) as Rc<dyn Fn()>);
        let (width, height) = style.arrows_size;
        let radius = style
            .shape
            .corner_radius(width, height, LayoutDirection::LeftToRight);
        let arrows =
            interaction.focus_ring(stepper_arrows(&style, state, up, down), radius.top_left());

        let row = interaction
            .attach(rect(), !disabled)
            .a11y_role(AccessibilityRole::SpinButton)
            .a11y_alt(&*self.label)
            .a11y_builder(move |node| node.set_numeric_value(current as f64))
            .on_key_down(move |event: Event<KeyboardEventData>| match event.key {
                Key::Named(NamedKey::ArrowUp) => step_key(1),
                Key::Named(NamedKey::ArrowDown) => step_key(-1),
                _ => {}
            });
        let root = match style.value_position {
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
        };
        self.overrides.apply(root)
    }
}

pub(crate) fn stepper_arrows(
    style: &StepperStyle,
    state: ControlState,
    up: Option<Rc<dyn Fn()>>,
    down: Option<Rc<dyn Fn()>>,
) -> Rect {
    let paints = Paints::current(state, style.effect);
    let arrow = |kind: Symbol, label: &'static str, offset: f32, press: Option<Rc<dyn Fn()>>| {
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
            .child(rect().offset_y(offset).child(stroked_symbol(
                kind,
                style.chevron_size,
                style.chevron_stroke,
                ink,
            )))
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
        .painted(&paints, &style.background)
        .bordered(&paints, style.border.as_ref())
        .child(arrow(
            Symbol::ChevronUp,
            text!("increase"),
            style.chevron_offsets.0,
            up,
        ))
        .child(
            rect()
                .width(Size::fill())
                .height(Size::px(style.separator_height))
                .margin(Gaps::new(
                    0.0,
                    style.separator_inset,
                    0.0,
                    style.separator_inset,
                ))
                .offset_y(style.separator_drop)
                .child(
                    rect()
                        .width(Size::fill())
                        .height(Size::fill())
                        .background(paints.fill(&style.separator)),
                ),
        )
        .child(arrow(
            Symbol::ChevronDown,
            text!("decrease"),
            style.chevron_offsets.1,
            down,
        ));
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
    label: Cow<'static, str>,
    items: Vec<(Cow<'static, str>, Face, Cow<'static, str>, bool)>,
    on_toggle: Option<Callback<(usize, bool), ()>>,
    style: Theme,
    overrides: Overrides,
}

impl ToggleGroup {
    pub fn new(label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            label: label.into(),
            items: Vec::new(),
            on_toggle: None,
            style: Theme::tengen(),
            overrides: Overrides::default(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(ToggleGroupStyle) -> ToggleGroupStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn item(
        mut self,
        text: impl Into<Cow<'static, str>>,
        face: Face,
        label: impl Into<Cow<'static, str>>,
        on: bool,
    ) -> Self {
        self.items.push((text.into(), face, label.into(), on));
        self
    }

    pub fn on_toggle(mut self, callback: impl Into<Callback<(usize, bool), ()>>) -> Self {
        self.on_toggle = Some(callback.into());
        self
    }
}

forward_overrides!(ToggleGroup);

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
            .map(|(index, (text, face, label, _))| {
                let mut states = states;
                let on_toggle = self.on_toggle.clone();
                let on = states.read().get(index).copied().unwrap_or(false);
                Toggle {
                    text: text.clone(),
                    face: *face,
                    label: label.clone(),
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
            .a11y_alt(&*self.label)
            .direction(direction)
            .padding(Gaps::new_all(style.padding))
            .spacing(style.gap)
            .corner_radius(style.shape.corner_radius(
                thickness,
                thickness,
                LayoutDirection::LeftToRight,
            ))
            .painted(&paints, &style.background)
            .bordered(&paints, style.border.as_ref())
            .children(toggles);
        self.overrides.apply(dimmed(group, style.opacity))
    }
}

#[derive(PartialEq)]
struct Toggle {
    text: Cow<'static, str>,
    face: Face,
    label: Cow<'static, str>,
    on: bool,
    on_press: Callback<(), ()>,
    style: Theme,
}

impl Component for Toggle {
    fn render(&self) -> impl IntoElement {
        let style: ToggleGroupStyle = resolve(&self.style);
        let interaction = use_interaction();
        let disabled = !use_is_enabled();
        let state = control_state(interaction, disabled);
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
            .attach(rect(), !disabled)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(&*self.label)
            .a11y_builder(move |node| node.set_toggled(on.into()))
            .width(Size::px(width))
            .height(Size::px(height))
            .corner_radius(radius)
            .center()
            .painted(&paints, paint)
            .maybe(!disabled, |el| el.on_press(move |_| on_press.call(())))
            .child(
                paragraph()
                    .with(|el| text_style(el, font))
                    .font_slant(match face {
                        Face::Italic => FontSlant::Italic,
                        Face::Bold | Face::Underline => FontSlant::Upright,
                    })
                    .color(paints.fill(ink))
                    .span(Span::new(self.text.clone()).text_decoration(match face {
                        Face::Underline => TextDecoration::Underline,
                        Face::Bold | Face::Italic => TextDecoration::None,
                    })),
            );
        let toggle = if on {
            shadowed(toggle, &style.selected_shadows)
        } else {
            toggle
        };
        interaction.focus_ring(dimmed(toggle, style.opacity), radius.top_left())
    }
}

#[derive(PartialEq)]
pub struct ToolbarSearch {
    value: State<String>,
    placeholder: Cow<'static, str>,
    collapsed: bool,
    style: Theme,
    overrides: Overrides,
}

impl ToolbarSearch {
    pub fn new(value: State<String>, placeholder: impl Into<Cow<'static, str>>) -> Self {
        Self {
            value,
            placeholder: placeholder.into(),
            collapsed: false,
            style: Theme::tengen(),
            overrides: Overrides::default(),
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

forward_overrides!(ToolbarSearch);

impl Component for ToolbarSearch {
    fn render(&self) -> impl IntoElement {
        let style: ToolbarSearchStyle = resolve(&self.style);
        let mut opened = use_state(|| false);
        let open = !self.collapsed || *opened.read() || !self.value.read().is_empty();
        if !open {
            return self
                .overrides
                .apply(
                    rect().child(
                        IconButton::new(Symbol::MagnifyingGlass, self.placeholder.clone())
                            .on_press(move |_| opened.set(true)),
                    ),
                )
                .into_element();
        }
        let root = rect().width(Size::px(style.width)).child(
            crate::controls::SearchField::new(self.value, self.placeholder.clone())
                .auto_focus(self.collapsed)
                .on_focus_change(move |focused: bool| {
                    if !focused {
                        opened.set(false);
                    }
                }),
        );
        self.overrides.apply(root)
    }
}

#[derive(PartialEq)]
pub struct ToolbarOverflow {
    items: Vec<crate::menu::MenuItem>,
    on_activate: Option<Callback<String, ()>>,
    style: Theme,
    overrides: Overrides,
}

impl ToolbarOverflow {
    pub fn new(items: Vec<crate::menu::MenuItem>) -> Self {
        Self {
            items,
            on_activate: None,
            style: Theme::tengen(),
            overrides: Overrides::default(),
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

forward_overrides!(ToolbarOverflow);

impl Component for ToolbarOverflow {
    fn render(&self) -> impl IntoElement {
        let mut area = use_state(torin::prelude::Area::default);
        let items = self.items.clone();
        let on_activate = self.on_activate.clone();
        let root = rect()
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
                        crate::platform::anchor_rect(anchor),
                        None,
                        move |id| {
                            if let Some(on_activate) = &on_activate {
                                on_activate.call(id);
                            }
                        },
                    );
                }),
            );
        self.overrides.apply(root)
    }
}

use ctdesign::Rgba;
use freya_core::prelude::*;
use std::borrow::Cow;
use torin::prelude::{Area, Gaps, Position, Size};

use crate::context_menu::edit_menu;
use crate::controls::{
    ControlSize, default_pre_key, dimmed, input, outer_hairline, shadowed, text_style,
};
use crate::elements::{Overrides, forward_overrides};
use crate::environment::use_disabled;
use crate::focus::use_focus_target;
use crate::menu::MenuItem;
use crate::platform::PopupAnchor;
use crate::popup_button::open_popover_menu;
use crate::styles::Painted;
use crate::styles::{
    ComboBoxStyle, ControlState, LayoutDirection, Paints, ShapeStyle, StateValue, TextFieldStyle,
    Theme, resolve,
};
use crate::symbol::{Symbol, stroked_symbol};

#[derive(PartialEq)]
pub struct ComboBox {
    value: State<String>,
    placeholder: Cow<'static, str>,
    items: Vec<Cow<'static, str>>,
    width: Size,
    disabled: bool,
    style: Theme,
    overrides: Overrides,
}

impl ComboBox {
    pub fn new(value: State<String>, placeholder: impl Into<Cow<'static, str>>) -> Self {
        Self {
            value,
            placeholder: placeholder.into(),
            items: Vec::new(),
            width: Size::fill(),
            disabled: false,
            style: Theme::tengen(),
            overrides: Overrides::default(),
        }
    }

    pub fn size(self, size: ControlSize) -> Self {
        self.style(move |style: ComboBoxStyle| style.sized(size))
    }

    pub fn style(mut self, restyle: impl Fn(ComboBoxStyle) -> ComboBoxStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn item(mut self, label: impl Into<Cow<'static, str>>) -> Self {
        self.items.push(label.into());
        self
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

forward_overrides!(ComboBox);

impl Component for ComboBox {
    fn render(&self) -> impl IntoElement {
        let style: ComboBoxStyle = resolve(&self.style);
        let field_style: TextFieldStyle = resolve(&Theme::tengen());
        let value = self.value;
        let mut area = use_state(Area::default);
        let items = self.items.clone();
        let field = use_hook(AccessibilityId::new_unique);
        use_focus_target(field);
        let focus = use_focus(field);
        let disabled = use_disabled(self.disabled);
        let mut hovered = use_state(|| false);
        let state = ControlState {
            hovered: *hovered.read(),
            focused: !matches!(*focus.read(), Focus::Not),
            disabled,
            ..ControlState::default()
        };
        let field_paints = Paints::current(state, field_style.effect);
        let ink = |paint: &StateValue<ShapeStyle>| field_paints.color(paint.resolve(state));
        let button_state = ControlState {
            disabled,
            ..ControlState::default()
        };
        let paints = Paints::current(button_state, style.effect);
        let margin_y = (style.height - style.font.line_height as f32) / 2.0;
        let trailing = style.button_inset + style.button_size.0 + style.text_gap;
        let radius = field_style.shape.corner_radius(
            style.height,
            style.height,
            LayoutDirection::LeftToRight,
        );
        let input = input(
            value,
            self.placeholder.clone(),
            [
                ink(&field_style.background),
                Rgba::TRANSPARENT,
                ink(&field_style.border),
                ink(&field_style.text),
                ink(&field_style.placeholder),
            ],
            radius,
            Gaps::new(margin_y, trailing, margin_y, field_style.padding_x),
        )
        .height(Size::px(style.height))
        .a11y_id(field);
        let open = move || {
            let field = *area.peek();
            let entries = items
                .iter()
                .enumerate()
                .map(|(index, label)| MenuItem::action(index.to_string(), label.clone()))
                .collect();
            let items = items.clone();
            open_popover_menu(
                entries,
                PopupAnchor::BottomLeft,
                crate::platform::anchor_rect(field),
                None,
                move |id| {
                    if let Some(label) = id.parse::<usize>().ok().and_then(|i| items.get(i)) {
                        let mut value = value;
                        value.set(label.to_string());
                    }
                },
            );
        };
        let opener = open.clone();
        let input = input.on_pre_key_down(move |event: Event<KeyboardEventData>| {
            if disabled {
                event.stop_propagation();
                return false;
            }
            if event.key == Key::Named(NamedKey::ArrowDown) {
                event.stop_propagation();
                opener();
                return false;
            }
            default_pre_key(&event)
        });
        let (button_width, button_height) = style.button_size;
        let button = rect()
            .width(Size::px(button_width))
            .height(Size::px(button_height))
            .corner_radius(style.button_shape.corner_radius(
                button_width,
                button_height,
                LayoutDirection::LeftToRight,
            ))
            .center()
            .painted(&paints, &style.button)
            .bordered(&paints, style.button_border.as_ref())
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(&*self.placeholder)
            .on_press(move |_| {
                if !disabled {
                    open()
                }
            })
            .child(stroked_symbol(
                Symbol::ChevronDown,
                style.chevron_size,
                style.chevron_stroke,
                paints.color(style.chevron.resolve(button_state)),
            ));
        let combo_box = dimmed(
            text_style(rect().width(self.width.clone()), style.font),
            field_style.opacity,
        )
        .corner_radius(radius)
        .border(outer_hairline(ink(&field_style.border)))
        .map(field_paints.blur(&field_style.background), |el, blur| {
            el.corner_radius(radius).backdrop_blur(blur)
        })
        .on_pointer_enter(move |_| hovered.set(true))
        .on_pointer_leave(move |_| hovered.set(false));
        let root = edit_menu(combo_box, field, true, (true, true), None)
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .child(input)
            .maybe(disabled, |el| {
                el.child(
                    rect()
                        .position(Position::new_absolute().top(0.0).left(0.0))
                        .width(Size::fill())
                        .height(Size::fill())
                        .on_pointer_down(|event: Event<PointerEventData>| event.stop_propagation()),
                )
            })
            .child(
                rect()
                    .position(
                        Position::new_absolute()
                            .right(style.button_inset)
                            .top(((area.read().size.height - button_height) / 2.0).max(0.0)),
                    )
                    .child(dimmed(
                        shadowed(button, &style.button_shadows),
                        style.opacity,
                    )),
            );
        self.overrides.apply(root)
    }
}

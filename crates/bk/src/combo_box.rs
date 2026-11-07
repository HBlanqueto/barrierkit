use freya_core::prelude::*;
use torin::prelude::{Area, Gaps, Position, Size};

use crate::context_menu::edit_menu;
use crate::controls::{default_pre_key, dimmed, inner_border, input, shadowed, text_style};
use crate::menu::MenuItem;
use crate::platform::PopupAnchor;
use crate::popup_button::open_popover_menu;
use crate::styles::{
    ComboBoxStyle, ControlState, LayoutDirection, Paints, ShapeStyle, StateValue, TextFieldStyle,
    Theme, resolve,
};
use crate::symbol::{Symbol, symbol};

#[derive(PartialEq)]
pub struct ComboBox {
    value: State<String>,
    placeholder: &'static str,
    items: Vec<&'static str>,
    width: Size,
    style: Theme,
}

impl ComboBox {
    pub fn new(value: State<String>, placeholder: &'static str) -> Self {
        Self {
            value,
            placeholder,
            items: Vec::new(),
            width: Size::fill(),
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(ComboBoxStyle) -> ComboBoxStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn item(mut self, label: &'static str) -> Self {
        self.items.push(label);
        self
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }
}

impl Component for ComboBox {
    fn render(&self) -> impl IntoElement {
        let style: ComboBoxStyle = resolve(&self.style);
        let field_style: TextFieldStyle = resolve(&Theme::tengen());
        let value = self.value;
        let mut area = use_state(Area::default);
        let items = self.items.clone();
        let field = use_hook(AccessibilityId::new_unique);
        let focus = use_focus(field);
        let mut hovered = use_state(|| false);
        let state = ControlState {
            hovered: *hovered.read(),
            focused: !matches!(*focus.read(), Focus::Not),
            ..ControlState::default()
        };
        let field_paints = Paints::current(state, field_style.effect);
        let ink = |paint: &StateValue<ShapeStyle>| field_paints.color(paint.resolve(state));
        let paints = Paints::current(ControlState::default(), style.effect);
        let margin_y = (field_style.height - field_style.font.line_height as f32) / 2.0;
        let trailing = style.button_inset + style.button_size.0 + style.text_gap;
        let radius = field_style.shape.corner_radius(
            field_style.height,
            field_style.height,
            LayoutDirection::LeftToRight,
        );
        let input = input(
            value,
            self.placeholder,
            [
                ink(&field_style.background),
                ink(&field_style.border),
                ink(&field_style.border),
                ink(&field_style.text),
                ink(&field_style.placeholder),
            ],
            radius,
            Gaps::new(margin_y, trailing, margin_y, field_style.padding_x),
        )
        .a11y_id(field);
        let open = move || {
            let field = *area.peek();
            let entries = items
                .iter()
                .enumerate()
                .map(|(index, label)| MenuItem::action(index.to_string(), *label))
                .collect();
            let items = items.clone();
            open_popover_menu(
                entries,
                PopupAnchor::BottomLeft,
                (
                    field.origin.x.round() as i32,
                    field.origin.y.round() as i32,
                    field.size.width.round() as u32,
                    field.size.height.round() as u32,
                ),
                None,
                move |id| {
                    if let Some(label) = id.parse::<usize>().ok().and_then(|i| items.get(i)) {
                        let mut value = value;
                        value.set((*label).to_owned());
                    }
                },
            );
        };
        let opener = open.clone();
        let input = input.on_pre_key_down(move |event: Event<KeyboardEventData>| {
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
            .background(paints.fill(&style.button))
            .map(paints.blur(&style.button), Rect::blur)
            .map(style.button_border.as_ref(), |el, border| {
                el.border(inner_border(&paints, border))
            })
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.placeholder)
            .on_press(move |_| open())
            .child(symbol(
                Symbol::ChevronDown,
                style.chevron_size,
                paints.color(style.chevron.resolve(ControlState::default())),
            ));
        let combo_box = dimmed(
            text_style(rect().width(self.width.clone()), field_style.font),
            field_style.opacity,
        )
        .map(field_paints.blur(&field_style.background), |el, blur| {
            el.corner_radius(radius).blur(blur)
        })
        .on_pointer_enter(move |_| hovered.set(true))
        .on_pointer_leave(move |_| hovered.set(false));
        edit_menu(combo_box, field, true, (true, true), None)
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .child(input)
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
            )
    }
}

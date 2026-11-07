use ctdesign::{BODY, BUTTON, Font, ICON_BUTTON, SPACE_1};

use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct WindowTabsStyle {
    pub tab_height: f32,
    pub padding: f32,
    pub spacing: f32,
    pub tab_padding_x: f32,
    pub symbol_size: f32,
    pub new_tab_width: f32,
    pub tab_shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub tab_background: StateValue<ShapeStyle>,
    pub selected_tab_background: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub symbol: StateValue<ShapeStyle>,
    pub effect: ColorEffect,
    pub font: Font,
}

impl ControlStyle for WindowTabsStyle {
    fn tengen() -> Self {
        Self {
            tab_height: BUTTON.height,
            padding: SPACE_1,
            spacing: SPACE_1,
            tab_padding_x: SPACE_1,
            symbol_size: ICON_BUTTON.icon,
            new_tab_width: ICON_BUTTON.size,
            tab_shape: Shape::Capsule,
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            tab_background: StateValue::new(ShapeStyle::Clear)
                .hovered(ShapeStyle::Role(Role::ControlHover)),
            selected_tab_background: StateValue::new(ShapeStyle::Role(Role::RaisedBackground)),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            symbol: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            effect: system_effect,
            font: BODY,
        }
    }
}

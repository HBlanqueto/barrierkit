use ctdesign::{BODY, BUTTON, Font, ICON_BUTTON, SPACE_1};

use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct WindowTabsStyle {
    /// Tengen, with `padding`, `spacing`, `tab_padding_x`: `BUTTON.height`; `SPACE_1` around
    /// the bar, between tabs and on each side of a tab's content. The bar is `tab_height` plus
    /// twice `padding` high.
    pub tab_height: f32,
    /// Tengen, with `tab_height`, `spacing`, `tab_padding_x`: `BUTTON.height`; `SPACE_1` around
    /// the bar, between tabs and on each side of a tab's content. The bar is `tab_height` plus
    /// twice `padding` high.
    pub padding: f32,
    /// Tengen, with `tab_height`, `padding`, `tab_padding_x`: `BUTTON.height`; `SPACE_1` around
    /// the bar, between tabs and on each side of a tab's content. The bar is `tab_height` plus
    /// twice `padding` high.
    pub spacing: f32,
    /// Tengen, with `tab_height`, `padding`, `spacing`: `BUTTON.height`; `SPACE_1` around the
    /// bar, between tabs and on each side of a tab's content. The bar is `tab_height` plus
    /// twice `padding` high.
    pub tab_padding_x: f32,
    /// Tengen: `ICON_BUTTON.icon`, the × and +.
    pub symbol_size: f32,
    /// Tengen: `ICON_BUTTON.size`, the + button.
    pub new_tab_width: f32,
    /// Tengen: `Capsule`.
    pub tab_shape: Shape,
    /// Tengen: `Role(ControlBackground)`, the bar.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Clear`, hovered `Role(ControlHover)`.
    pub tab_background: StateValue<ShapeStyle>,
    /// Tengen: `Role(RaisedBackground)`.
    pub selected_tab_background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`, the × and +.
    pub symbol: StateValue<ShapeStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `BODY`.
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

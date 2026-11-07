use ctdesign::{BODY, DISCLOSURE, Font, HAIRLINE, SPACE_1};

use super::button::rounded;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct OutlineViewStyle {
    pub row_height: f32,
    pub indent: f32,
    pub spacing: f32,
    pub icon_size: f32,
    pub chevron_size: f32,
    pub column_width: f32,
    pub column_padding_x: f32,
    pub row_shape: Shape,
    pub row_background: StateValue<ShapeStyle>,
    pub selected_row_background: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub selected_label: StateValue<ShapeStyle>,
    pub icon: StateValue<ShapeStyle>,
    pub selected_icon: StateValue<ShapeStyle>,
    pub disclosure: StateValue<ShapeStyle>,
    pub column_chevron: StateValue<ShapeStyle>,
    pub selected_chevron: StateValue<ShapeStyle>,
    pub column_separator: (ShapeStyle, f32),
    pub effect: ColorEffect,
    pub font: Font,
}

impl ControlStyle for OutlineViewStyle {
    fn tengen() -> Self {
        Self {
            row_height: DISCLOSURE.row_height,
            indent: DISCLOSURE.indent,
            spacing: SPACE_1,
            icon_size: DISCLOSURE.icon,
            chevron_size: DISCLOSURE.chevron,
            column_width: DISCLOSURE.column_width,
            column_padding_x: SPACE_1,
            row_shape: rounded(DISCLOSURE.row_radius),
            row_background: StateValue::new(ShapeStyle::Clear),
            selected_row_background: StateValue::new(ShapeStyle::Accent),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            selected_label: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            icon: StateValue::new(ShapeStyle::Accent),
            selected_icon: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            disclosure: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            column_chevron: StateValue::new(ShapeStyle::Role(Role::TertiaryLabel)),
            selected_chevron: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            column_separator: (ShapeStyle::Role(Role::Separator), HAIRLINE),
            effect: system_effect,
            font: BODY,
        }
    }
}

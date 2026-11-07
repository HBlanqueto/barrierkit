use ctdesign::{BODY, BUTTON, Font, HAIRLINE, SETTINGS};

use super::button::rounded;
use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct ComboButtonStyle {
    pub height: f32,
    pub padding_x: f32,
    pub menu_padding_x: f32,
    pub chevron_size: f32,
    pub separator_size: (f32, f32),
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub segment: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub chevron: StateValue<ShapeStyle>,
    pub separator: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
    pub font: Font,
}

impl ControlStyle for ComboButtonStyle {
    fn tengen() -> Self {
        Self {
            height: BUTTON.height,
            padding_x: BUTTON.padding_x,
            menu_padding_x: BUTTON.menu_padding_x,
            chevron_size: SETTINGS.dropdown_chevron_glyph as f32,
            separator_size: (HAIRLINE, BUTTON.height / 2.0),
            shape: rounded(BUTTON.radius),
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            segment: StateValue::new(ShapeStyle::Clear)
                .hovered(ShapeStyle::Role(Role::ControlHover)),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            chevron: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            separator: StateValue::new(ShapeStyle::Role(Role::Separator)),
            border: None,
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
        }
    }
}

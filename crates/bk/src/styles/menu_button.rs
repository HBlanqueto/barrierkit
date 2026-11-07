use ctdesign::{BODY, BUTTON, Font, SETTINGS, SPACE_1};

use super::button::rounded;
use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct MenuButtonStyle {
    pub height: f32,
    pub padding_x: f32,
    pub chevron_gap: f32,
    pub chevron_size: f32,
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub chevron: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
    pub font: Font,
}

impl ControlStyle for MenuButtonStyle {
    fn tengen() -> Self {
        Self {
            height: BUTTON.height,
            padding_x: BUTTON.padding_x,
            chevron_gap: SPACE_1,
            chevron_size: SETTINGS.dropdown_chevron_glyph as f32,
            shape: rounded(BUTTON.radius),
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground))
                .hovered(ShapeStyle::Role(Role::ControlHover)),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            chevron: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            border: None,
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
        }
    }
}

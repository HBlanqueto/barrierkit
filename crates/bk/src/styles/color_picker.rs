use ctdesign::{BODY, BUTTON, Font, HAIRLINE, SPACE_2, SPLIT_VIEW};

use super::button::rounded;
use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct ColorPickerStyle {
    pub size: (f32, f32),
    pub panel_size: (f32, f32),
    pub spacing: f32,
    pub shape: Shape,
    pub caption: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
    pub font: Font,
}

impl ControlStyle for ColorPickerStyle {
    fn tengen() -> Self {
        Self {
            size: (BUTTON.height * 2.0, BUTTON.height),
            panel_size: (SPLIT_VIEW.inspector, BODY.line_height as f32 * 7.0),
            spacing: SPACE_2,
            shape: rounded(BUTTON.radius),
            caption: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            border: Some((ShapeStyle::Role(Role::CardBorder), HAIRLINE)),
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
        }
    }
}

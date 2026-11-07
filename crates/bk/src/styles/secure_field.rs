use ctdesign::{BODY, Font, HAIRLINE, TEXT_FIELD};

use super::text_field::field_shape;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct SecureFieldStyle {
    pub height: f32,
    pub padding_x: f32,
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub text: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub opacity: f32,
    pub effect: ColorEffect,
    pub font: Font,
}

impl ControlStyle for SecureFieldStyle {
    fn tengen() -> Self {
        Self {
            height: TEXT_FIELD.height,
            padding_x: TEXT_FIELD.padding_x,
            shape: field_shape(),
            background: StateValue::new(ShapeStyle::Role(Role::ContentBackground)),
            text: StateValue::new(ShapeStyle::Role(Role::Label)),
            border: Some((ShapeStyle::Role(Role::Separator), HAIRLINE)),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
        }
    }
}

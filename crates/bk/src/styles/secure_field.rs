use ctdesign::{BODY, Font, HAIRLINE, TEXT_FIELD};

use super::text_field::field_shape;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct SecureFieldStyle {
    /// Tengen: `TEXT_FIELD.height`.
    pub height: f32,
    /// Tengen: `TEXT_FIELD.padding_x`.
    pub padding_x: f32,
    /// Tengen: `RoundedRectangle { radius: TEXT_FIELD.radius, style: Circular }`.
    pub shape: Shape,
    /// Tengen: `Role(ContentBackground)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`, the `•`.
    pub text: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(Separator), HAIRLINE))`.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `BODY`; the text sits (`height` − `line_height`) / 2 from the top.
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

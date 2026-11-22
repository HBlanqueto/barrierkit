use ctdesign::{BODY, Font, HAIRLINE, TEXT_FIELD};

use super::text_field::field_shape;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct SecureFieldStyle {
    /// Tengen: `TEXT_FIELD.height`.
    pub height: f32,
    /// Tengen: `TEXT_FIELD.padding_x`.
    pub padding_x: f32,
    /// Tengen: `RoundedRectangle { radius: TEXT_FIELD.radius, style: Continuous }`.
    pub shape: Shape,
    /// Tengen: `Role(TextBackground)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`, the dots.
    pub text: StateValue<ShapeStyle>,
    /// Tengen: `(TEXT_FIELD.secure_dot, TEXT_FIELD.secure_pitch)`: the diameter of each dot of a
    /// hidden secret and the distance between their centers.
    pub dot: (f32, f32),
    /// Tengen: `(TEXT_FIELD.secure_inset, TEXT_FIELD.secure_drop)`: the space before the first dot
    /// and how far the dots sit below the center of the field.
    pub dot_offset: (f32, f32),
    /// Tengen: `Some((Role(ControlBorder), HAIRLINE))`, drawn outside.
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
            background: StateValue::new(ShapeStyle::Role(Role::TextBackground)),
            text: StateValue::new(ShapeStyle::Role(Role::Label)),
            dot: (TEXT_FIELD.secure_dot, TEXT_FIELD.secure_pitch),
            dot_offset: (TEXT_FIELD.secure_inset, TEXT_FIELD.secure_drop),
            border: Some((ShapeStyle::Role(Role::ControlBorder), HAIRLINE)),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
        }
    }
}

use ctdesign::{BODY, Font, TEXT_FIELD};

use super::{
    ColorEffect, ControlStyle, CornerStyle, Role, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct TextFieldStyle {
    pub height: f32,
    pub padding_x: f32,
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub border: StateValue<ShapeStyle>,
    pub text: StateValue<ShapeStyle>,
    pub placeholder: StateValue<ShapeStyle>,
    pub misspelling: StateValue<ShapeStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
    pub font: Font,
}

pub(super) fn field_shape() -> Shape {
    Shape::RoundedRectangle {
        radius: TEXT_FIELD.radius,
        style: CornerStyle::Circular,
    }
}

impl ControlStyle for TextFieldStyle {
    fn tengen() -> Self {
        Self {
            height: TEXT_FIELD.height,
            padding_x: TEXT_FIELD.padding_x,
            shape: field_shape(),
            background: StateValue::new(ShapeStyle::Role(Role::ContentBackground)),
            border: StateValue::new(ShapeStyle::Role(Role::Separator)).focused(ShapeStyle::Accent),
            text: StateValue::new(ShapeStyle::Role(Role::Label)),
            placeholder: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            misspelling: StateValue::new(ShapeStyle::Role(Role::Destructive)),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
        }
    }
}

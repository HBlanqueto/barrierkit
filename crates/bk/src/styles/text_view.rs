use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct TextViewStyle {
    pub text: StateValue<ShapeStyle>,
    pub cursor: StateValue<ShapeStyle>,
    pub effect: ColorEffect,
}

impl ControlStyle for TextViewStyle {
    fn tengen() -> Self {
        Self {
            text: StateValue::new(ShapeStyle::Role(Role::Label)),
            cursor: StateValue::new(ShapeStyle::Role(Role::Label)),
            effect: system_effect,
        }
    }
}

use ctdesign::HAIRLINE;

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct DividerStyle {
    pub thickness: f32,
    pub line: StateValue<ShapeStyle>,
    pub effect: ColorEffect,
}

impl ControlStyle for DividerStyle {
    fn tengen() -> Self {
        Self {
            thickness: HAIRLINE,
            line: StateValue::new(ShapeStyle::Role(Role::Separator)),
            effect: system_effect,
        }
    }
}

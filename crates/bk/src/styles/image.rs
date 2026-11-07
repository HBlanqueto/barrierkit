use ctdesign::BODY;

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct ImageStyle {
    pub size: f32,
    pub foreground: StateValue<ShapeStyle>,
    pub effect: ColorEffect,
}

impl ControlStyle for ImageStyle {
    fn tengen() -> Self {
        Self {
            size: BODY.size as f32,
            foreground: StateValue::new(ShapeStyle::Role(Role::Label)),
            effect: system_effect,
        }
    }
}

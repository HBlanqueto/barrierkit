use ctdesign::BODY;

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct ImageStyle {
    /// Tengen: `size`: `BODY.size`, a square.
    pub size: f32,
    /// Tengen: `foreground` `Role(Label)`, the symbol.
    pub foreground: StateValue<ShapeStyle>,
    /// Tengen: `system_effect`.
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

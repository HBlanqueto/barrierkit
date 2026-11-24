use ctdesign::{BODY, SYMBOL};

use crate::styles::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct ImageStyle {
    /// Tengen: `size`: `BODY.size`, a square.
    pub size: f32,
    /// Tengen: `SYMBOL.stroke`, the line width of a symbol on its grid.
    pub stroke: f32,
    /// Tengen: `foreground` `Role(Label)`, the symbol.
    pub foreground: StateValue<ShapeStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
}

impl ControlStyle for ImageStyle {
    fn tengen() -> Self {
        Self {
            size: BODY.size as f32,
            stroke: SYMBOL.stroke,
            foreground: StateValue::new(ShapeStyle::Role(Role::Label)),
            effect: system_effect,
        }
    }
}

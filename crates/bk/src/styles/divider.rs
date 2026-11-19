use ctdesign::HAIRLINE;

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct DividerStyle {
    /// Tengen: `thickness`: `HAIRLINE`; the line fills the width.
    pub thickness: f32,
    /// Tengen: `line` `Role(Separator)`.
    pub line: StateValue<ShapeStyle>,
    /// Tengen: `system_effect`.
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

use ctappearance::Resolved;
use ctdesign::{BODY, Font};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct TextStyle {
    pub foreground: StateValue<ShapeStyle>,
    pub effect: ColorEffect,
    pub font: Font,
}

impl ControlStyle for TextStyle {
    fn tengen() -> Self {
        Self {
            foreground: StateValue::new(ShapeStyle::Role(Role::Label)),
            effect: system_effect,
            font: BODY,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

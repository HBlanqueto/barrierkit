use ctappearance::Resolved;
use ctdesign::{BODY, Font};

use super::{ColorEffect, ControlStyle, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct LinkStyle {
    pub label: StateValue<ShapeStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
    pub font: Font,
}

impl ControlStyle for LinkStyle {
    fn tengen() -> Self {
        Self {
            label: StateValue::new(ShapeStyle::Color(crate::theme::resolved().link)),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

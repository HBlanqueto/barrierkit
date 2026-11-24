use ctdesign::{BODY, Font};

use crate::styles::{ControlStyle, ShapeStyle, StateValue};

/// States: hovered, pressed, focused.
#[derive(Clone, Debug)]
pub struct LinkStyle {
    /// Tengen: `Color` of the resolved link color, which keeps 4.5:1 on the window.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `BODY`.
    pub font: Font,
}

impl ControlStyle for LinkStyle {
    fn tengen() -> Self {
        Self {
            label: StateValue::new(ShapeStyle::Color(crate::theme::resolved().link)),
            opacity: 1.0,
            font: BODY,
        }
    }
}

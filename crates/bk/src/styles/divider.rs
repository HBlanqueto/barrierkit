use ctdesign::HAIRLINE;

use crate::styles::{ControlStyle, Role, ShapeStyle, StateValue};

#[derive(Clone, Debug)]
pub struct DividerStyle {
    /// Tengen: `thickness`: `HAIRLINE`; the line fills the width.
    pub thickness: f32,
    /// Tengen: `line` `Role(Separator)`.
    pub line: StateValue<ShapeStyle>,
}

impl ControlStyle for DividerStyle {
    fn tengen() -> Self {
        Self {
            thickness: HAIRLINE,
            line: StateValue::new(ShapeStyle::Role(Role::Separator)),
        }
    }
}

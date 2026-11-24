use ctdesign::SPLIT_PANES;

use crate::styles::{ControlStyle, Role, ShapeStyle, StateValue};

#[derive(Clone, Debug)]
pub struct SplitPanesStyle {
    /// Tengen: `divider_width`: `SPLIT_PANES.divider`.
    pub divider_width: f32,
    /// Tengen: `divider` `Role(Separator)`.
    pub divider: StateValue<ShapeStyle>,
}

impl ControlStyle for SplitPanesStyle {
    fn tengen() -> Self {
        Self {
            divider_width: SPLIT_PANES.divider,
            divider: StateValue::new(ShapeStyle::Role(Role::Separator)),
        }
    }
}

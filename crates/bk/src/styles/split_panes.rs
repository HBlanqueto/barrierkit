use ctdesign::SPLIT_PANES;

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct SplitPanesStyle {
    /// Tengen: `divider_width`: `SPLIT_PANES.divider`.
    pub divider_width: f32,
    /// Tengen: `divider` `Role(Separator)`.
    pub divider: StateValue<ShapeStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
}

impl ControlStyle for SplitPanesStyle {
    fn tengen() -> Self {
        Self {
            divider_width: SPLIT_PANES.divider,
            divider: StateValue::new(ShapeStyle::Role(Role::Separator)),
            effect: system_effect,
        }
    }
}

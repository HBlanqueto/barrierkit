use ctdesign::SPLIT_PANES;

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct SplitPanesStyle {
    pub divider_width: f32,
    pub divider: StateValue<ShapeStyle>,
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

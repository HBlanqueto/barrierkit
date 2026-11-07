use ctappearance::Resolved;
use ctdesign::LEVEL_INDICATOR;

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct RelevancyIndicatorStyle {
    pub height: f32,
    pub line_width: f32,
    pub gap: f32,
    pub line: StateValue<ShapeStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
}

impl ControlStyle for RelevancyIndicatorStyle {
    fn tengen() -> Self {
        Self {
            height: LEVEL_INDICATOR.relevancy_height,
            line_width: LEVEL_INDICATOR.relevancy_line,
            gap: LEVEL_INDICATOR.relevancy_gap,
            line: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            opacity: 1.0,
            effect: system_effect,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

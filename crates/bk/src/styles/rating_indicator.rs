use ctdesign::LEVEL_INDICATOR;

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct RatingIndicatorStyle {
    pub star_size: f32,
    pub star: StateValue<ShapeStyle>,
    pub empty_star: StateValue<ShapeStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
}

impl ControlStyle for RatingIndicatorStyle {
    fn tengen() -> Self {
        Self {
            star_size: LEVEL_INDICATOR.star,
            star: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            empty_star: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            opacity: 1.0,
            effect: system_effect,
        }
    }
}

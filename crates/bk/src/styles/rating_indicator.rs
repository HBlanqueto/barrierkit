use ctdesign::LEVEL_INDICATOR;

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct RatingIndicatorStyle {
    /// Tengen: `LEVEL_INDICATOR.star`.
    pub star_size: f32,
    /// Tengen, with `empty_star`: `star`, `empty_star` `Role(SecondaryLabel)`.
    pub star: StateValue<ShapeStyle>,
    /// Tengen, with `star`: `star`, `empty_star` `Role(SecondaryLabel)`.
    pub empty_star: StateValue<ShapeStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
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

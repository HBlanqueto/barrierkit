use ctdesign::LEVEL_INDICATOR;

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct RatingIndicatorStyle {
    /// Tengen: `(LEVEL_INDICATOR.star, LEVEL_INDICATOR.star_pitch)`: the size of each star and the
    /// width and height of the square it is centered in.
    pub star_size: (f32, f32),
    /// Tengen: `LEVEL_INDICATOR.star_drop`: how far below the center of its square each star sits.
    pub star_drop: f32,
    /// Tengen, with `empty_star`: `star` `Role(SecondaryLabel)`, `empty_star` `Role(SystemFill)`.
    pub star: StateValue<ShapeStyle>,
    /// Tengen, with `star`: `star` `Role(SecondaryLabel)`, `empty_star` `Role(SystemFill)`.
    pub empty_star: StateValue<ShapeStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
}

impl ControlStyle for RatingIndicatorStyle {
    fn tengen() -> Self {
        Self {
            star_size: (LEVEL_INDICATOR.star, LEVEL_INDICATOR.star_pitch),
            star_drop: LEVEL_INDICATOR.star_drop,
            star: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            empty_star: StateValue::new(ShapeStyle::Role(Role::SystemFill)),
            opacity: 1.0,
            effect: system_effect,
        }
    }
}

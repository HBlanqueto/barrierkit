use ctdesign::DURATION_BASE;

use crate::styles::{Animation, ControlStyle, ShapeStyle, StateValue};

#[derive(Clone, Debug)]
pub struct FullscreenBarStyle {
    /// Tengen: `Color` of the band under the overlaid title bar, as `TopGlassStyle`.
    pub tint: StateValue<ShapeStyle>,
    /// Tengen: `Custom { duration_ms: DURATION_BASE, curve }` with a cubic ease-out, how the
    /// bar comes down and goes up.
    pub animation: Animation,
}

impl ControlStyle for FullscreenBarStyle {
    fn tengen() -> Self {
        Self {
            tint: StateValue::new(ShapeStyle::Color(crate::glass::glass_tint())),
            animation: Animation::Custom {
                duration_ms: DURATION_BASE,
                curve: crate::animation::ease_out,
            },
        }
    }
}

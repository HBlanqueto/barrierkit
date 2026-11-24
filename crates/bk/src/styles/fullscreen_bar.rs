use ctdesign::{DURATION_BASE, FULLSCREEN_GLASS, FullscreenGlassMetrics};

use crate::styles::{Animation, ControlStyle, ShapeStyle, StateValue};

#[derive(Clone, Debug)]
pub struct FullscreenBarStyle {
    /// Tengen: `FULLSCREEN_GLASS`. Its `height` is how deep `FullscreenBar::ProgressiveGlass` is;
    /// the compositor takes the rest of the metrics.
    pub glass: FullscreenGlassMetrics,
    /// Tengen: `Color` of the solid a window without compositor glass draws, as `TopGlassStyle`.
    pub tint: StateValue<ShapeStyle>,
    /// Tengen: `Custom { duration_ms: DURATION_BASE, curve }` with a cubic ease-out, how the
    /// bar comes down and goes up.
    pub animation: Animation,
}

impl ControlStyle for FullscreenBarStyle {
    fn tengen() -> Self {
        Self {
            glass: FULLSCREEN_GLASS,
            tint: StateValue::new(ShapeStyle::Color(crate::glass::glass_tint())),
            animation: Animation::Custom {
                duration_ms: DURATION_BASE,
                curve: crate::animation::ease_out,
            },
        }
    }
}

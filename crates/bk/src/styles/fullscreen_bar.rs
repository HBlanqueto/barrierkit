use ctappearance::Resolved;
use ctdesign::{DURATION_BASE, FULLSCREEN_GLASS, FullscreenGlassMetrics};

use super::{
    Animation, ColorEffect, ControlStyle, ShapeStyle, StateValue, reduce_motion, system_effect,
};

#[derive(Clone, Debug)]
pub struct FullscreenBarStyle {
    /// Tengen: `FULLSCREEN_GLASS`. Its `height` is how deep `FullscreenBar::ProgressiveGlass` is;
    /// the compositor takes the rest of the metrics.
    pub glass: FullscreenGlassMetrics,
    /// Tengen: `Color` of the solid a window without compositor glass draws, as `TopGlassStyle`.
    pub tint: StateValue<ShapeStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `Custom { duration_ms: DURATION_BASE, curve }` with a cubic ease-out, how the
    /// bar comes down and goes up.
    pub animation: Animation,
}

impl ControlStyle for FullscreenBarStyle {
    fn tengen() -> Self {
        Self {
            glass: FULLSCREEN_GLASS,
            tint: StateValue::new(ShapeStyle::Color(crate::glass::glass_tint())),
            effect: system_effect,
            animation: Animation::Custom {
                duration_ms: DURATION_BASE,
                curve: crate::animation::ease_out,
            },
        }
    }

    fn accessible(self, appearance: &Resolved) -> Self {
        reduce_motion(self, appearance, |style| Self {
            animation: Animation::None,
            ..style
        })
    }
}

use ctappearance::Resolved;
use ctdesign::{DURATION_BASE, FULLSCREEN_BAND, FULLSCREEN_GLASS, FullscreenGlassMetrics};

use super::{
    Animation, ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, reduce_motion,
    system_effect,
};

#[derive(Clone, Debug)]
pub struct FullscreenBarStyle {
    /// Tengen, with `glass`: `FULLSCREEN_BAND` for `FullscreenBar::Glass`, `FULLSCREEN_GLASS`

    /// for `ProgressiveGlass`.
    pub band: FullscreenGlassMetrics,
    /// Tengen, with `band`: `FULLSCREEN_BAND` for `FullscreenBar::Glass`, `FULLSCREEN_GLASS`
    /// for `ProgressiveGlass`.
    pub glass: FullscreenGlassMetrics,
    /// Tengen: `Color` of the glass of the window, as `TopGlassStyle`.
    pub tint: StateValue<ShapeStyle>,
    /// Tengen: `Role(Separator)`, the line under the `Glass` band.
    pub line: StateValue<ShapeStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `Custom { duration_ms: DURATION_BASE, curve }` with a cubic ease-out, how the
    /// bar comes down and goes up.
    pub animation: Animation,
}

impl ControlStyle for FullscreenBarStyle {
    fn tengen() -> Self {
        Self {
            band: FULLSCREEN_BAND,
            glass: FULLSCREEN_GLASS,
            tint: StateValue::new(ShapeStyle::Color(crate::glass::glass_tint())),
            line: StateValue::new(ShapeStyle::Role(Role::Separator)),
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

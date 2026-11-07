use ctappearance::Resolved;
use ctdesign::{DURATION_BASE, FULLSCREEN_BAND, FULLSCREEN_GLASS, FullscreenGlassMetrics};

use super::{
    Animation, ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, reduce_motion,
    system_effect,
};

#[derive(Clone, Debug)]
pub struct FullscreenBarStyle {
    pub band: FullscreenGlassMetrics,
    pub glass: FullscreenGlassMetrics,
    pub tint: StateValue<ShapeStyle>,
    pub line: StateValue<ShapeStyle>,
    pub effect: ColorEffect,
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

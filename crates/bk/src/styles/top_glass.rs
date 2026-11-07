use ctappearance::Resolved;
use ctdesign::{FULLSCREEN_GLASS, FullscreenGlassMetrics};

use super::{ColorEffect, ControlStyle, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct TopGlassStyle {
    pub glass: FullscreenGlassMetrics,
    pub tint: StateValue<ShapeStyle>,
    pub effect: ColorEffect,
}

impl ControlStyle for TopGlassStyle {
    fn tengen() -> Self {
        Self {
            glass: FULLSCREEN_GLASS,
            tint: StateValue::new(ShapeStyle::Color(crate::glass::glass_tint())),
            effect: system_effect,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

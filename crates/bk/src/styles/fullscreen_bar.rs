use ctappearance::Resolved;
use ctdesign::{FULLSCREEN_BAND, FULLSCREEN_GLASS, FullscreenGlassMetrics};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct FullscreenBarStyle {
    pub band: FullscreenGlassMetrics,
    pub glass: FullscreenGlassMetrics,
    pub tint: StateValue<ShapeStyle>,
    pub line: StateValue<ShapeStyle>,
    pub effect: ColorEffect,
}

impl ControlStyle for FullscreenBarStyle {
    fn tengen() -> Self {
        Self {
            band: FULLSCREEN_BAND,
            glass: FULLSCREEN_GLASS,
            tint: StateValue::new(ShapeStyle::Color(crate::glass::glass_tint())),
            line: StateValue::new(ShapeStyle::Role(Role::Separator)),
            effect: system_effect,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

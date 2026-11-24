use ctdesign::{FULLSCREEN_GLASS, FullscreenGlassMetrics};

use crate::styles::{ControlStyle, ShapeStyle, StateValue};

#[derive(Clone, Debug)]
pub struct TopGlassStyle {
    /// Tengen: `FULLSCREEN_GLASS`: the height, blur, layers and the stops of the tint and the
    /// mask.
    pub glass: FullscreenGlassMetrics,
    /// Tengen: `Color` of the glass of the window: `raised_background` over a `Backdrop`, the
    /// window surface without one.
    pub tint: StateValue<ShapeStyle>,
}

impl ControlStyle for TopGlassStyle {
    fn tengen() -> Self {
        Self {
            glass: FULLSCREEN_GLASS,
            tint: StateValue::new(ShapeStyle::Color(crate::glass::glass_tint())),
        }
    }
}

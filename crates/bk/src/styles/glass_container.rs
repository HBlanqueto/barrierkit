use ctdesign::MEDIA;

use crate::styles::{ControlStyle, ShapeStyle, StateValue};

#[derive(Clone, Debug)]
pub struct GlassContainerStyle {
    /// Tengen: `Color(MEDIA.round_glass)`.
    pub tint: StateValue<ShapeStyle>,
    /// Tengen: `Color` of the window's glass, as `TopGlassStyle`, opaque in place of `tint` when
    /// the user reduces transparency.
    pub solid: StateValue<ShapeStyle>,
    /// Tengen: `false`. `true` asks the platform that draws the glass itself for `Glass.clear`.
    pub clear: bool,
}

impl ControlStyle for GlassContainerStyle {
    fn tengen() -> Self {
        Self {
            tint: StateValue::new(ShapeStyle::Color(MEDIA.round_glass)),
            solid: StateValue::new(ShapeStyle::Color(crate::glass::glass_tint())),
            clear: false,
        }
    }
}

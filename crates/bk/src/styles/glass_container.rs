use ctappearance::Resolved;
use ctdesign::{DURATION_BASE, MEDIA};

use super::{
    Animation, ColorEffect, ControlStyle, ShapeStyle, StateValue, reduce_motion, system_effect,
};

#[derive(Clone, Debug)]
pub struct GlassContainerStyle {
    /// Tengen: `Color(MEDIA.round_glass)`.
    pub tint: StateValue<ShapeStyle>,
    /// Tengen: `Color` of the window's glass, as `TopGlassStyle`, opaque in place of `tint` when
    /// the user reduces transparency.
    pub solid: StateValue<ShapeStyle>,
    /// Tengen: `Color(MEDIA.round_rim)`, one pixel along the outline.
    pub rim: StateValue<ShapeStyle>,
    /// Tengen: `MEDIA.material_blur`.
    pub blur: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `Ease { duration_ms: DURATION_BASE }`: the shapes morph; `None` makes them jump.
    pub animation: Animation,
    /// Tengen: `false`. `true` asks the platform that draws the glass itself for `Glass.clear`.
    pub clear: bool,
}

impl ControlStyle for GlassContainerStyle {
    fn tengen() -> Self {
        Self {
            tint: StateValue::new(ShapeStyle::Color(MEDIA.round_glass)),
            solid: StateValue::new(ShapeStyle::Color(crate::glass::glass_tint())),
            rim: StateValue::new(ShapeStyle::Color(MEDIA.round_rim)),
            blur: MEDIA.material_blur,
            effect: system_effect,
            animation: Animation::Ease {
                duration_ms: DURATION_BASE,
            },
            clear: false,
        }
    }

    fn accessible(self, appearance: &Resolved) -> Self {
        reduce_motion(self, appearance, |style| Self {
            animation: Animation::None,
            ..style
        })
    }
}

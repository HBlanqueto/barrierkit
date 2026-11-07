use ctappearance::Resolved;
use ctdesign::{DURATION_BASE, MEDIA};

use super::{Animation, ColorEffect, ControlStyle, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct GlassContainerStyle {
    pub tint: StateValue<ShapeStyle>,
    pub solid: StateValue<ShapeStyle>,
    pub rim: StateValue<ShapeStyle>,
    pub blur: f32,
    pub effect: ColorEffect,
    pub animation: Animation,
}

impl ControlStyle for GlassContainerStyle {
    fn tengen() -> Self {
        Self {
            tint: StateValue::new(ShapeStyle::Color(MEDIA.round_glass)),
            solid: StateValue::new(ShapeStyle::Color(MEDIA.material_tint)),
            rim: StateValue::new(ShapeStyle::Color(MEDIA.round_rim)),
            blur: MEDIA.material_blur,
            effect: system_effect,
            animation: Animation::Ease {
                duration_ms: DURATION_BASE,
            },
        }
    }

    fn accessible(self, appearance: &Resolved) -> Self {
        if appearance.reduced_motion {
            Self {
                animation: Animation::None,
                ..self
            }
        } else {
            self
        }
    }
}

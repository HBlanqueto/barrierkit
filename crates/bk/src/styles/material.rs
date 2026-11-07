use ctdesign::{HAIRLINE, MEDIA};

use super::{ColorEffect, ControlStyle, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct MaterialStyle {
    pub tint: StateValue<ShapeStyle>,
    pub blur: f32,
    pub border: Option<(ShapeStyle, f32)>,
    pub effect: ColorEffect,
}

impl ControlStyle for MaterialStyle {
    fn tengen() -> Self {
        Self {
            tint: StateValue::new(ShapeStyle::Color(MEDIA.material_tint)),
            blur: MEDIA.material_blur,
            border: Some((ShapeStyle::Color(MEDIA.material_rim), HAIRLINE)),
            effect: system_effect,
        }
    }
}

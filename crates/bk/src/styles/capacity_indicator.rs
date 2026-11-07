use ctdesign::{HAIRLINE, LEVEL_INDICATOR, LEVEL_INDICATOR_COLORS};

use super::button::rounded;
use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct CapacityIndicatorStyle {
    pub height: f32,
    pub segment_gap: f32,
    pub shape: Shape,
    pub track: StateValue<ShapeStyle>,
    pub fill: StateValue<ShapeStyle>,
    pub warning: StateValue<ShapeStyle>,
    pub critical: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
}

impl ControlStyle for CapacityIndicatorStyle {
    fn tengen() -> Self {
        let [fill, warning, critical] =
            LEVEL_INDICATOR_COLORS.map(|rgba| StateValue::new(ShapeStyle::Color(rgba)));
        Self {
            height: LEVEL_INDICATOR.height,
            segment_gap: LEVEL_INDICATOR.segment_gap,
            shape: rounded(LEVEL_INDICATOR.radius),
            track: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            fill,
            warning,
            critical,
            border: Some((ShapeStyle::Role(Role::CardBorder), HAIRLINE)),
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
        }
    }
}

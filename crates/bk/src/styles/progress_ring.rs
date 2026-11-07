use ctdesign::PROGRESS;

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct ProgressRingStyle {
    pub size: f32,
    pub stroke: f32,
    pub track: StateValue<ShapeStyle>,
    pub fill: StateValue<ShapeStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
}

impl ControlStyle for ProgressRingStyle {
    fn tengen() -> Self {
        Self {
            size: PROGRESS.ring_size,
            stroke: PROGRESS.ring_stroke,
            track: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            fill: StateValue::new(ShapeStyle::Accent),
            opacity: 1.0,
            effect: system_effect,
        }
    }
}

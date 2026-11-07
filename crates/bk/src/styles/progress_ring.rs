use ctdesign::PROGRESS;

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct ProgressRingStyle {
    /// Tengen: `PROGRESS.ring_size`.
    pub size: f32,
    /// Tengen: `PROGRESS.ring_stroke`.
    pub stroke: f32,
    /// Tengen: `Role(ControlBackground)`.
    pub track: StateValue<ShapeStyle>,
    /// Tengen: `Accent`.
    pub fill: StateValue<ShapeStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
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

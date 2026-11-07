use ctappearance::Resolved;
use ctdesign::{PROGRESS, SYMBOL};

use super::{
    Animation, ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, reduce_motion,
    system_effect,
};

#[derive(Clone, Debug)]
pub struct SpinnerStyle {
    pub size: f32,
    pub spokes: u32,
    pub stroke: f32,
    pub spoke: StateValue<ShapeStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
    pub animation: Animation,
}

impl ControlStyle for SpinnerStyle {
    fn tengen() -> Self {
        Self {
            size: PROGRESS.spinner_size,
            spokes: PROGRESS.spinner_spokes,
            stroke: SYMBOL.stroke,
            spoke: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            opacity: 1.0,
            effect: system_effect,
            animation: Animation::Custom {
                duration_ms: PROGRESS.spinner_ms,
                curve: std::convert::identity,
            },
        }
    }

    fn accessible(self, appearance: &Resolved) -> Self {
        reduce_motion(self, appearance, |style| Self {
            animation: Animation::None,
            ..style
        })
    }
}

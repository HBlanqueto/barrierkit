use ctappearance::Resolved;
use ctdesign::{PROGRESS, SYMBOL};

use super::{
    Animation, ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, reduce_motion,
    system_effect,
};

#[derive(Clone, Debug)]
pub struct SpinnerStyle {
    /// Tengen: `PROGRESS.spinner_size`.
    pub size: f32,
    /// Tengen: `PROGRESS.spinner_spokes`.
    pub spokes: u32,
    /// Tengen: `SYMBOL.stroke`.
    pub stroke: f32,
    /// Tengen: `spoke` `Role(SecondaryLabel)`, fading around the turn.
    pub spoke: StateValue<ShapeStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `Custom { duration_ms: PROGRESS.spinner_ms, curve: identity }`: one turn, spoke
    /// by spoke, again and again.
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

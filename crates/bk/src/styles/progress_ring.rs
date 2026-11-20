use ctdesign::{
    GAUGE_CONTRAST_DARK, GAUGE_CONTRAST_DARK_TINT, GAUGE_CONTRAST_LIGHT, PROGRESS, RING_TRACK_DARK,
    RING_TRACK_DARK_TINT, RING_TRACK_LIGHT,
};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, per_look, system_effect};

#[derive(Clone, Debug)]
pub struct ProgressRingStyle {
    /// Tengen: `PROGRESS.ring_size`.
    pub size: f32,
    /// Tengen: `PROGRESS.ring_stroke`.
    pub stroke: f32,
    /// Tengen: `PROGRESS.ring_trim`, in degrees: how much shorter the arc is than the value
    /// times 360, measured at its center line.
    pub trim: f32,
    /// Tengen: `Color` of `RING_TRACK`.
    pub track: StateValue<ShapeStyle>,
    /// Tengen: `Role(RingAccent)`.
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
            trim: PROGRESS.ring_trim,
            track: StateValue::new(ShapeStyle::Color(per_look(
                [RING_TRACK_LIGHT, RING_TRACK_DARK, RING_TRACK_DARK_TINT],
                [
                    GAUGE_CONTRAST_LIGHT.ring_track,
                    GAUGE_CONTRAST_DARK.ring_track,
                    GAUGE_CONTRAST_DARK_TINT.ring_track,
                ],
            ))),
            fill: StateValue::new(ShapeStyle::Role(Role::RingAccent)),
            opacity: 1.0,
            effect: system_effect,
        }
    }
}

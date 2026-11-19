use ctappearance::Resolved;
use ctdesign::PROGRESS;

use crate::animation::ease_in_out;

use super::{
    Animation, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    reduce_motion, system_effect,
};

/// An SVG part (the ring and the spinner) takes one color: a `Gradient` gives its
/// first stop and a `Material` its tint.
///
/// States: only `normal`. `ProgressView::tint(Tint)` is an instance override of
/// `fill`. With `Animation::None` (also with reduce motion) the segment rests at
/// the start and the spinner stands still.
#[derive(Clone, Debug)]
pub struct ProgressViewStyle {
    /// Tengen: `PROGRESS.height`; the bar fills its width.
    pub height: f32,
    /// Tengen: `PROGRESS.indeterminate_segment`, the share of the track the sweeping segment
    /// takes.
    pub indeterminate_length: f32,
    /// Tengen: `Capsule`, the track and the fill.
    pub shape: Shape,
    /// Tengen: `Role(ControlBackground)`.
    pub track: StateValue<ShapeStyle>,
    /// Tengen: `Accent`.
    pub fill: StateValue<ShapeStyle>,
    /// Tengen: `None`, around the track.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `Custom { duration_ms: PROGRESS.indeterminate_ms, curve }` with an ease-in-out
    /// cubic curve: the segment sweeps there and back.
    pub animation: Animation,
}

impl ControlStyle for ProgressViewStyle {
    fn tengen() -> Self {
        Self {
            height: PROGRESS.height,
            indeterminate_length: PROGRESS.indeterminate_segment,
            shape: Shape::Capsule,
            track: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            fill: StateValue::new(ShapeStyle::Accent),
            border: None,
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
            animation: Animation::Custom {
                duration_ms: PROGRESS.indeterminate_ms,
                curve: ease_in_out,
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

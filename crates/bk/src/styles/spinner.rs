use ctappearance::Resolved;
use ctdesign::{ControlSize, PROGRESS, SYMBOL};

use crate::styles::{Animation, ControlStyle, Role, ShapeStyle, StateValue, reduce_motion};

#[derive(Clone, Debug)]
pub struct SpinnerStyle {
    /// Tengen: `PROGRESS.spinner_sizes` of the control size (32 at regular).
    pub size: f32,
    /// Tengen: `PROGRESS.spinner_spokes`.
    pub spokes: u32,
    /// Tengen: `PROGRESS.spinner_frames`: the steps of one turn; the fade moves a spoke every `frames / spokes` of them.
    pub frames: u32,
    /// Tengen: `SYMBOL.stroke`.
    pub stroke: f32,
    /// Tengen: `spoke` `Role(SecondaryLabel)`, fading around the turn.
    pub spoke: StateValue<ShapeStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `Custom { duration_ms: PROGRESS.spinner_ms, curve: identity }`: one turn in
    /// `frames` steps, again and again.
    pub animation: Animation,
}

impl SpinnerStyle {
    pub(crate) fn sized(self, size: ControlSize) -> Self {
        Self {
            size: PROGRESS.spinner_sizes[size as usize],
            ..self
        }
    }
}

impl ControlStyle for SpinnerStyle {
    fn tengen() -> Self {
        Self {
            size: PROGRESS.spinner_sizes[ControlSize::Regular as usize],
            spokes: PROGRESS.spinner_spokes,
            frames: PROGRESS.spinner_frames,
            stroke: SYMBOL.stroke,
            spoke: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            opacity: 1.0,
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

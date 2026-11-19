use ctappearance::Resolved;
use ctdesign::{BACKDROP, DURATION_SLOW_MS};

use super::{Animation, ControlStyle, reduce_motion};
use crate::animation::ease_in_out;

/// The colors of a `Backdrop` come from its source: they are content. The
/// palette it gives its content has no style.
#[derive(Clone, Debug)]
pub struct BackdropStyle {
    /// Tengen: `BACKDROP.angles`, the direction of each gradient layer in degrees.
    pub angles: [f32; 3],
    /// Tengen: `BACKDROP.opacity`, of each layer where it starts.
    pub opacity: f32,
    /// Tengen: `Custom { duration_ms: DURATION_SLOW_MS, curve }` with an ease-in-out cubic
    /// curve: a new source fades in; `None` makes it jump.
    pub animation: Animation,
}

impl ControlStyle for BackdropStyle {
    fn tengen() -> Self {
        Self {
            angles: BACKDROP.angles,
            opacity: BACKDROP.opacity,
            animation: Animation::Custom {
                duration_ms: DURATION_SLOW_MS,
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

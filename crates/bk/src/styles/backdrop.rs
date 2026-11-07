use ctappearance::Resolved;
use ctdesign::{BACKDROP, DURATION_SLOW_MS};

use super::{Animation, ControlStyle};
use crate::animation::ease_in_out;

#[derive(Clone, Debug)]
pub struct BackdropStyle {
    pub angles: [f32; 3],
    pub opacity: f32,
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
        if appearance.reduced_motion {
            Self {
                animation: Animation::None,
                ..self
            }
        } else {
            self
        }
    }
}

use ctappearance::Resolved;
use ctdesign::{BAR_TRACK_DARK, BAR_TRACK_DARK_TINT, BAR_TRACK_LIGHT, ControlSize, PROGRESS};

use crate::animation::ease_in_out;

use crate::styles::{
    Animation, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, per_scheme,
    reduce_motion,
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
    /// Tengen: `PROGRESS.frames` of the control size (20 at regular), the box the bar is centered in.
    pub frame_height: f32,
    /// Tengen: `PROGRESS.indeterminate_segment`, the share of the track the sweeping segment
    /// takes.
    pub indeterminate_length: f32,
    /// Tengen: `Capsule`, the track and the fill.
    pub shape: Shape,
    /// Tengen: `Color` of `BAR_TRACK`; `Role(ControlBackground)` over a tinted `Backdrop`.
    pub track: StateValue<ShapeStyle>,
    /// Tengen: `Accent`.
    pub fill: StateValue<ShapeStyle>,
    /// Tengen: `None`, around the track.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `Custom { duration_ms: PROGRESS.indeterminate_ms, curve }` with an ease-in-out
    /// cubic curve: the segment sweeps there and back.
    pub animation: Animation,
}

impl ProgressViewStyle {
    pub(crate) fn sized(self, size: ControlSize) -> Self {
        Self {
            frame_height: PROGRESS.frames[size as usize],
            ..self
        }
    }
}

impl ControlStyle for ProgressViewStyle {
    fn tengen() -> Self {
        Self {
            height: PROGRESS.height,
            frame_height: PROGRESS.frames[ControlSize::Regular as usize],
            indeterminate_length: PROGRESS.indeterminate_segment,
            shape: Shape::Capsule,
            track: StateValue::new(if crate::backdrop::tinted().is_some() {
                ShapeStyle::Role(Role::ControlBackground)
            } else {
                ShapeStyle::Color(per_scheme([
                    BAR_TRACK_LIGHT,
                    BAR_TRACK_DARK,
                    BAR_TRACK_DARK_TINT,
                ]))
            }),
            fill: StateValue::new(ShapeStyle::Accent),
            border: None,
            shadows: Vec::new(),
            opacity: 1.0,
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

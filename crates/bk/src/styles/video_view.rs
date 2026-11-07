use super::{ControlStyle, Shape};

/// With the `video` feature.
///
/// `.corner_radius(r)` is an instance override of `shape` with
/// `RoundedRectangle { radius: r, style: Circular }`, applied before `.style`.
#[derive(Clone, Debug)]
pub struct VideoViewStyle {
    /// Tengen: `Rectangle`, the clip of the frame; a `Capsule` or `Circle` rounds by the
    /// view's size.
    pub shape: Shape,
}

impl ControlStyle for VideoViewStyle {
    fn tengen() -> Self {
        Self {
            shape: Shape::Rectangle,
        }
    }
}

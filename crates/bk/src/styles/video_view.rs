use ctappearance::Resolved;

use super::{ControlStyle, Shape};

#[derive(Clone, Debug)]
pub struct VideoViewStyle {
    pub shape: Shape,
}

impl ControlStyle for VideoViewStyle {
    fn tengen() -> Self {
        Self {
            shape: Shape::Rectangle,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

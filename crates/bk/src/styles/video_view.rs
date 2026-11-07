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
}

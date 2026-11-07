use ctappearance::Resolved;
use ctdesign::PROGRESS;

use crate::animation::ease_in_out;

use super::{
    Animation, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    reduce_motion, system_effect,
};

#[derive(Clone, Debug)]
pub struct ProgressViewStyle {
    pub height: f32,
    pub indeterminate_length: f32,
    pub shape: Shape,
    pub track: StateValue<ShapeStyle>,
    pub fill: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
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

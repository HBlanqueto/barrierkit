use ctappearance::Resolved;
use ctdesign::{BADGE, CAPTION, Font};

use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct BadgeStyle {
    pub height: f32,
    pub padding_x: f32,
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub shadows: Vec<ShadowStyle>,
    pub effect: ColorEffect,
    pub font: Font,
}

impl ControlStyle for BadgeStyle {
    fn tengen() -> Self {
        Self {
            height: BADGE.height,
            padding_x: BADGE.padding_x,
            shape: Shape::Capsule,
            background: StateValue::new(ShapeStyle::Role(Role::Destructive)),
            label: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            border: None,
            shadows: Vec::new(),
            effect: system_effect,
            font: CAPTION,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

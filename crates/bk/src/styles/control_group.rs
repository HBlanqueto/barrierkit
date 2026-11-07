use ctappearance::Resolved;
use ctdesign::{BUTTON, HAIRLINE};

use super::button::rounded;
use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct ControlGroupStyle {
    pub height: f32,
    pub separator_size: (f32, f32),
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub separator: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
}

impl ControlStyle for ControlGroupStyle {
    fn tengen() -> Self {
        Self {
            height: BUTTON.height,
            separator_size: (HAIRLINE, BUTTON.height / 2.0),
            shape: rounded(BUTTON.radius),
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            separator: StateValue::new(ShapeStyle::Role(Role::Separator)),
            border: None,
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

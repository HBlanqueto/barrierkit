use ctappearance::Resolved;
use ctappearance::backdrop::ink_on;
use ctdesign::{Font, HEADLINE, SETTINGS};

use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};
use crate::style::Palette;

#[derive(Clone, Debug)]
pub struct AvatarStyle {
    pub size: f32,
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub shadows: Vec<ShadowStyle>,
    pub effect: ColorEffect,
    pub font: Font,
}

impl ControlStyle for AvatarStyle {
    fn tengen() -> Self {
        Self {
            size: SETTINGS.account_avatar as f32,
            shape: Shape::Circle,
            background: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            label: StateValue::new(ShapeStyle::Color(ink_on(
                Palette::current().secondary_label,
            ))),
            border: None,
            shadows: Vec::new(),
            effect: system_effect,
            font: HEADLINE,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

use ctdesign::{BADGE, CAPTION, Font};

use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct BadgeStyle {
    /// Tengen, with `padding_x`: `height`, `padding_x`: `BADGE.height`, `BADGE.padding_x`; the

    /// badge is at least as wide as it is tall.
    pub height: f32,
    /// Tengen, with `height`: `height`, `padding_x`: `BADGE.height`, `BADGE.padding_x`; the
    /// badge is at least as wide as it is tall.
    pub padding_x: f32,
    /// Tengen: `Capsule`.
    pub shape: Shape,
    /// Tengen, with `label`: `background` `Role(Destructive)`, `label` `Role(OnAccent)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen, with `background`: `background` `Role(Destructive)`, `label` `Role(OnAccent)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `None`.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `font` `CAPTION`.
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
}

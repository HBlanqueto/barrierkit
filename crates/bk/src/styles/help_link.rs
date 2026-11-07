use ctappearance::Resolved;
use ctdesign::{BODY, BUTTON, Font};

use super::{
    Animation, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    reduce_motion, system_effect,
};

#[derive(Clone, Debug)]
pub struct HelpLinkStyle {
    /// Tengen: `size` `BUTTON.height`, a square.
    pub size: f32,
    /// Tengen: `Circle`.
    pub shape: Shape,
    /// Tengen: `Role(ControlBackground)`, hovered `Role(ControlHover)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`, the `?`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `None`.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `BUTTON.pressed_scale`.
    pub pressed_scale: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `BODY`.
    pub font: Font,
    /// Tengen: `None`: the pressed scale jumps.
    pub animation: Animation,
}

impl ControlStyle for HelpLinkStyle {
    fn tengen() -> Self {
        Self {
            size: BUTTON.height,
            shape: Shape::Circle,
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground))
                .hovered(ShapeStyle::Role(Role::ControlHover)),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            border: None,
            shadows: Vec::new(),
            opacity: 1.0,
            pressed_scale: BUTTON.pressed_scale,
            effect: system_effect,
            font: BODY,
            animation: Animation::None,
        }
    }

    fn accessible(self, appearance: &Resolved) -> Self {
        reduce_motion(self, appearance, |style| Self {
            animation: Animation::None,
            ..style
        })
    }
}

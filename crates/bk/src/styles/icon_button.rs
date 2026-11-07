use ctappearance::Resolved;
use ctdesign::{BUTTON, ICON_BUTTON, TOOLGROUP};

use super::button::rounded;
use super::{
    Animation, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    system_effect,
};

#[derive(Clone, Debug)]
pub struct IconButtonStyle {
    pub size: f32,
    pub icon_size: f32,
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub icon: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub disabled_opacity: f32,
    pub pressed_scale: f32,
    pub effect: ColorEffect,
    pub animation: Animation,
}

impl ControlStyle for IconButtonStyle {
    fn tengen() -> Self {
        let (size, shape) = if crate::compact::in_toolbar() {
            (TOOLGROUP.height as f32, Shape::Capsule)
        } else {
            (ICON_BUTTON.size, rounded(ICON_BUTTON.radius))
        };
        Self {
            size,
            icon_size: ICON_BUTTON.icon,
            shape,
            background: StateValue::new(ShapeStyle::Clear)
                .hovered(ShapeStyle::Role(Role::ControlHover))
                .disabled(ShapeStyle::Clear),
            icon: StateValue::new(ShapeStyle::Role(Role::Label))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            border: None,
            shadows: Vec::new(),
            opacity: 1.0,
            disabled_opacity: 1.0,
            pressed_scale: BUTTON.pressed_scale,
            effect: system_effect,
            animation: Animation::None,
        }
    }

    fn accessible(self, appearance: &Resolved) -> Self {
        if appearance.reduced_motion {
            Self {
                animation: Animation::None,
                ..self
            }
        } else {
            self
        }
    }
}

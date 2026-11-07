use ctappearance::Resolved;
use ctdesign::{BUTTON, HAIRLINE, MEDIA};

use super::{
    Animation, ColorEffect, ControlStyle, ShadowStyle, Shape, ShapeStyle, StateValue,
    reduce_motion, system_effect,
};

#[derive(Clone, Debug)]
pub struct RoundButtonStyle {
    /// Tengen: `size` `MEDIA.round`, a square.
    pub size: f32,
    /// Tengen: `MEDIA.round_icon`.
    pub icon_size: f32,
    /// Tengen: `Circle`.
    pub shape: Shape,
    /// Tengen: `Color(MEDIA.round_glass)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Color(MEDIA.button_fill)`, while `.selected(true)`.
    pub selected: StateValue<ShapeStyle>,
    /// Tengen: `Color(MEDIA.button_fill)`.
    pub icon: StateValue<ShapeStyle>,
    /// Tengen: `Color(MEDIA.button_ink)`.
    pub selected_icon: StateValue<ShapeStyle>,
    /// Tengen: `Some((Color(MEDIA.round_rim), HAIRLINE))`.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `BUTTON.pressed_scale`.
    pub pressed_scale: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `None`.
    pub animation: Animation,
}

impl ControlStyle for RoundButtonStyle {
    fn tengen() -> Self {
        Self {
            size: MEDIA.round,
            icon_size: MEDIA.round_icon,
            shape: Shape::Circle,
            background: StateValue::new(ShapeStyle::Color(MEDIA.round_glass)),
            selected: StateValue::new(ShapeStyle::Color(MEDIA.button_fill)),
            icon: StateValue::new(ShapeStyle::Color(MEDIA.button_fill)),
            selected_icon: StateValue::new(ShapeStyle::Color(MEDIA.button_ink)),
            border: Some((ShapeStyle::Color(MEDIA.round_rim), HAIRLINE)),
            shadows: Vec::new(),
            opacity: 1.0,
            pressed_scale: BUTTON.pressed_scale,
            effect: system_effect,
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

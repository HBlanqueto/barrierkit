use ctappearance::Resolved;
use ctdesign::{BUTTON, HAIRLINE, MEDIA};

use super::{
    Animation, ColorEffect, ControlStyle, ShadowStyle, Shape, ShapeStyle, StateValue,
    reduce_motion, system_effect,
};

#[derive(Clone, Debug)]
pub struct RoundButtonStyle {
    pub size: f32,
    pub icon_size: f32,
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub selected: StateValue<ShapeStyle>,
    pub icon: StateValue<ShapeStyle>,
    pub selected_icon: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub pressed_scale: f32,
    pub effect: ColorEffect,
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

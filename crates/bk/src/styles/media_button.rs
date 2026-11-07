use ctappearance::Resolved;
use ctdesign::{BODY, BUTTON, Font, MEDIA};

use super::button::rounded;
use super::{
    Animation, ColorEffect, ControlStyle, ShadowStyle, Shape, ShapeStyle, StateValue,
    reduce_motion, system_effect,
};

#[derive(Clone, Debug)]
pub struct MediaButtonStyle {
    pub height: f32,
    pub padding_x: f32,
    pub icon_size: f32,
    pub icon_gap: f32,
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub pressed_scale: f32,
    pub effect: ColorEffect,
    pub font: Font,
    pub animation: Animation,
}

impl ControlStyle for MediaButtonStyle {
    fn tengen() -> Self {
        Self {
            height: MEDIA.button_height,
            padding_x: MEDIA.button_padding_x,
            icon_size: MEDIA.button_font_size,
            icon_gap: MEDIA.button_font_size / 2.0,
            shape: rounded(MEDIA.button_radius),
            background: StateValue::new(ShapeStyle::Color(MEDIA.button_fill)),
            label: StateValue::new(ShapeStyle::Color(MEDIA.button_ink)),
            border: None,
            shadows: Vec::new(),
            opacity: 1.0,
            pressed_scale: BUTTON.pressed_scale,
            effect: system_effect,
            font: Font {
                size: MEDIA.button_font_size as u32,
                weight: MEDIA.button_font_weight as u16,
                ..BODY
            },
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

use ctappearance::Resolved;
use ctdesign::{BODY, BUTTON, Font, MEDIA};

use super::button::rounded;
use super::{
    Animation, ColorEffect, ControlStyle, ShadowStyle, Shape, ShapeStyle, StateValue,
    reduce_motion, system_effect,
};

/// The media buttons sit on artwork, so their Tengen paints are fixed colors.
///
/// States: hovered, pressed, focused.
#[derive(Clone, Debug)]
pub struct MediaButtonStyle {
    /// Tengen: `MEDIA.button_height`.
    pub height: f32,
    /// Tengen: `MEDIA.button_padding_x`.
    pub padding_x: f32,
    /// Tengen: `MEDIA.button_font_size`.
    pub icon_size: f32,
    /// Tengen: `MEDIA.button_font_size` / 2.
    pub icon_gap: f32,
    /// Tengen: `RoundedRectangle { radius: MEDIA.button_radius, style: Circular }`.
    pub shape: Shape,
    /// Tengen: `Color(MEDIA.button_fill)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Color(MEDIA.button_ink)`, the text and the symbol.
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
    /// Tengen: `BODY` with `MEDIA.button_font_size` and `MEDIA.button_font_weight`;
    /// `line_height` has no effect.
    pub font: Font,
    /// Tengen: `None`: the pressed scale jumps.
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

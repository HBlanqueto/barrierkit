use ctdesign::{Font, HAIRLINE, SETTINGS, SWITCH};

use super::button::rounded;
use super::{
    Axis, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    system_effect,
};

#[derive(Clone, Debug)]
pub struct ChoiceCardsStyle {
    pub card_size: (f32, f32),
    pub caption_gap: f32,
    pub spacing: f32,
    pub card_shape: Shape,
    pub selection: StateValue<ShapeStyle>,
    pub caption: StateValue<ShapeStyle>,
    pub selected_caption: StateValue<ShapeStyle>,
    pub card_border: Option<(ShapeStyle, f32)>,
    pub card_shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub disabled_opacity: f32,
    pub effect: ColorEffect,
    pub axis: Axis,
    pub wraps: bool,
    pub caption_font: Font,
    pub selected_caption_font: Font,
}

impl ControlStyle for ChoiceCardsStyle {
    fn tengen() -> Self {
        Self {
            card_size: (SETTINGS.card_width as f32, SETTINGS.card_height as f32),
            caption_gap: SETTINGS.card_caption_gap as f32,
            spacing: SETTINGS.card_gap as f32,
            card_shape: rounded(SETTINGS.card_radius as f32),
            selection: StateValue::new(ShapeStyle::Accent),
            caption: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            selected_caption: StateValue::new(ShapeStyle::Role(Role::Label)),
            card_border: Some((ShapeStyle::Color(SETTINGS.card_outline), HAIRLINE)),
            card_shadows: Vec::new(),
            opacity: 1.0,
            disabled_opacity: SWITCH.disabled_opacity,
            effect: system_effect,
            axis: Axis::Horizontal,
            wraps: false,
            caption_font: SETTINGS.card_caption_font,
            selected_caption_font: Font {
                weight: SETTINGS.card_caption_selected_weight as u16,
                ..SETTINGS.card_caption_font
            },
        }
    }
}

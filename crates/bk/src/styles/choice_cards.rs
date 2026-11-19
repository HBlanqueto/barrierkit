use ctdesign::{Font, HAIRLINE, SETTINGS, SWITCH};

use super::button::rounded;
use super::{
    Axis, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    system_effect,
};

#[derive(Clone, Debug)]
pub struct ChoiceCardsStyle {
    /// Tengen: `(SETTINGS.card_width, SETTINGS.card_height)`.
    pub card_size: (f32, f32),
    /// Tengen: `SETTINGS.card_caption_gap`.
    pub caption_gap: f32,
    /// Tengen: `SETTINGS.card_gap`; also between wrapped lines.
    pub spacing: f32,
    /// Tengen: `RoundedRectangle { radius: SETTINGS.card_radius, style: Circular }`.
    pub card_shape: Shape,
    /// Tengen: `Accent`, the ring of the chosen card.
    pub selection: StateValue<ShapeStyle>,
    /// Tengen: `0.0`, between the card and its ring.
    pub selection_gap: f32,
    /// Tengen: `Role(SecondaryLabel)`.
    pub caption: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub selected_caption: StateValue<ShapeStyle>,
    /// Tengen: `Some((Color(SETTINGS.card_outline), HAIRLINE))`.
    pub card_border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub card_shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `SWITCH.disabled_opacity`.
    pub disabled_opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `Horizontal`; `Vertical` stacks them.
    pub axis: Axis,
    /// Tengen: `false`; `true` starts a new line when the next one does not fit.
    pub wraps: bool,
    /// Tengen, with `selected_caption_font`: `SETTINGS.card_caption_font`, the same with
    /// `SETTINGS.card_caption_selected_weight`.
    pub caption_font: Font,
    /// Tengen, with `caption_font`: `SETTINGS.card_caption_font`, the same with
    /// `SETTINGS.card_caption_selected_weight`.
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
            selection_gap: 0.0,
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

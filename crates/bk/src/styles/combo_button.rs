use ctdesign::{BODY, BUTTON, Font, HAIRLINE, SETTINGS};

use super::button::rounded;
use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct ComboButtonStyle {
    /// Tengen: `BUTTON.height`.
    pub height: f32,
    /// Tengen: `BUTTON.padding_x`, the label half.
    pub padding_x: f32,
    /// Tengen: `BUTTON.menu_padding_x`, the chevron half.
    pub menu_padding_x: f32,
    /// Tengen: `SETTINGS.dropdown_chevron_glyph`.
    pub chevron_size: f32,
    /// Tengen: `(HAIRLINE, BUTTON.height / 2)`.
    pub separator_size: (f32, f32),
    /// Tengen: `RoundedRectangle { radius: BUTTON.radius, style: Circular }`.
    pub shape: Shape,
    /// Tengen: `Role(ControlBackground)`, behind both halves.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Clear`, hovered `Role(ControlHover)`, for each half.
    pub segment: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`.
    pub chevron: StateValue<ShapeStyle>,
    /// Tengen: `Role(Separator)`.
    pub separator: StateValue<ShapeStyle>,
    /// Tengen: `None`.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `BODY`.
    pub font: Font,
}

impl ControlStyle for ComboButtonStyle {
    fn tengen() -> Self {
        Self {
            height: BUTTON.height,
            padding_x: BUTTON.padding_x,
            menu_padding_x: BUTTON.menu_padding_x,
            chevron_size: SETTINGS.dropdown_chevron_glyph as f32,
            separator_size: (HAIRLINE, BUTTON.height / 2.0),
            shape: rounded(BUTTON.radius),
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            segment: StateValue::new(ShapeStyle::Clear)
                .hovered(ShapeStyle::Role(Role::ControlHover)),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            chevron: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            separator: StateValue::new(ShapeStyle::Role(Role::Separator)),
            border: None,
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
        }
    }
}

use ctdesign::{BODY, Font, SETTINGS, TEXT_FIELD};

use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct SearchFieldStyle {
    /// Tengen: `TEXT_FIELD.height`.
    pub height: f32,
    /// Tengen: `SETTINGS.search_padding_x`.
    pub padding_x: f32,
    /// Tengen: `SETTINGS.search_icon`.
    pub icon_size: f32,
    /// Tengen: `SETTINGS.search_gap`.
    pub icon_gap: f32,
    /// Tengen: `Capsule`.
    pub shape: Shape,
    /// Tengen: `Role(ControlBackground)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Clear`, focused `Accent`.
    pub border: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub text: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`.
    pub placeholder: StateValue<ShapeStyle>,
    /// Tengen, with `clear_icon`: `Role(TertiaryLabel)` magnifying glass,
    /// `Role(SecondaryLabel)` clear button.
    pub icon: StateValue<ShapeStyle>,
    /// Tengen, with `icon`: `Role(TertiaryLabel)` magnifying glass, `Role(SecondaryLabel)`
    /// clear button.
    pub clear_icon: StateValue<ShapeStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `BODY`; the text sits (`height` − `line_height`) / 2 from the top.
    pub font: Font,
}

impl ControlStyle for SearchFieldStyle {
    fn tengen() -> Self {
        Self {
            height: TEXT_FIELD.height,
            padding_x: SETTINGS.search_padding_x as f32,
            icon_size: SETTINGS.search_icon as f32,
            icon_gap: SETTINGS.search_gap as f32,
            shape: Shape::Capsule,
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            border: StateValue::new(ShapeStyle::Clear).focused(ShapeStyle::Accent),
            text: StateValue::new(ShapeStyle::Role(Role::Label)),
            placeholder: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            icon: StateValue::new(ShapeStyle::Role(Role::TertiaryLabel)),
            clear_icon: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
        }
    }
}

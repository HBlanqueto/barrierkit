use ctdesign::{
    BODY, Font, SEARCH_CLEAR_DARK, SEARCH_CLEAR_DARK_TINT, SEARCH_CLEAR_LIGHT, SETTINGS, TEXT_FIELD,
};

use super::{
    ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, per_scheme, system_effect,
};

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
    /// Tengen: `SETTINGS.search_glyph` and `SETTINGS.search_glyph_stroke`: the size and line
    /// width of the magnifying glass, which is wider than its `icon_size` slot.
    pub glyph: (f32, f32),
    /// Tengen: `(SETTINGS.search_glyph_dx, SETTINGS.search_glyph_dy)`: how far the glass sits
    /// right of and below the center of its slot.
    pub glyph_offset: (f32, f32),
    /// Tengen: `Capsule`.
    pub shape: Shape,
    /// Tengen: `Role(TextBackground)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Role(ControlBorder)`, focused `Accent`.
    pub border: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub text: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`.
    pub placeholder: StateValue<ShapeStyle>,
    /// Tengen, with `clear_icon`: `Role(SecondaryLabel)` magnifying glass and clear button.
    pub icon: StateValue<ShapeStyle>,
    /// Tengen, with `icon`: `Role(TextBackground)`, the cross cut out of the clear button's disc.
    pub clear_icon: StateValue<ShapeStyle>,
    /// Tengen: `SETTINGS.search_clear`, the diameter of the clear button's disc, and
    /// `SETTINGS.search_clear_glyph` and `SETTINGS.search_clear_glyph_stroke`, the size and line
    /// width of its cross.
    pub clear_size: (f32, f32, f32),
    /// Tengen: `(SETTINGS.search_clear_dx, SETTINGS.search_clear_dy)`: how far the disc sits right
    /// of and below where the field puts it.
    pub clear_offset: (f32, f32),
    /// Tengen: `Color` of `SEARCH_CLEAR`, the disc of the clear button.
    pub clear_background: StateValue<ShapeStyle>,
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
            glyph: (SETTINGS.search_glyph, SETTINGS.search_glyph_stroke),
            glyph_offset: (SETTINGS.search_glyph_dx, SETTINGS.search_glyph_dy),
            shape: Shape::Capsule,
            background: StateValue::new(ShapeStyle::Role(Role::TextBackground)),
            border: StateValue::new(ShapeStyle::Role(Role::ControlBorder))
                .focused(ShapeStyle::Accent),
            text: StateValue::new(ShapeStyle::Role(Role::Label)),
            placeholder: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            icon: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            clear_icon: StateValue::new(ShapeStyle::Role(Role::TextBackground)),
            clear_size: (
                SETTINGS.search_clear,
                SETTINGS.search_clear_glyph,
                SETTINGS.search_clear_glyph_stroke,
            ),
            clear_offset: (SETTINGS.search_clear_dx, SETTINGS.search_clear_dy),
            clear_background: StateValue::new(ShapeStyle::Color(per_scheme([
                SEARCH_CLEAR_LIGHT,
                SEARCH_CLEAR_DARK,
                SEARCH_CLEAR_DARK_TINT,
            ]))),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
        }
    }
}

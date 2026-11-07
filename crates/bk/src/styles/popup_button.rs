use ctdesign::{BODY, Font, SETTINGS, SYMBOL};

use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

/// States: hovered, pressed, focused. The menu it opens is a menu view
/// (`docs/barrierkit/07-menus-and-popovers.md`).
#[derive(Clone, Debug)]
pub struct PopUpButtonStyle {
    /// Tengen, with `gap`: `SETTINGS.popup_height`, `SETTINGS.popup_text_gap` between the
    /// icon, the label and the indicator.
    pub height: f32,
    /// Tengen, with `height`: `SETTINGS.popup_height`, `SETTINGS.popup_text_gap` between the
    /// icon, the label and the indicator.
    pub gap: f32,
    /// Tengen: `SYMBOL.grid`, the SVG icon of the chosen item.
    pub icon_size: f32,
    /// Tengen: `SETTINGS.popup_chevron`.
    pub indicator_size: f32,
    /// Tengen: `SETTINGS.dropdown_chevron_glyph`.
    pub chevron_size: f32,
    /// Tengen: `Circle`; the focus ring takes its corners.
    pub indicator_shape: Shape,
    /// Tengen: `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Role(ControlBackground)`.
    pub indicator: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub chevron: StateValue<ShapeStyle>,
    /// Tengen: `None`.
    pub indicator_border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub indicator_shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `BODY`.
    pub font: Font,
}

impl ControlStyle for PopUpButtonStyle {
    fn tengen() -> Self {
        Self {
            height: SETTINGS.popup_height as f32,
            gap: SETTINGS.popup_text_gap as f32,
            icon_size: SYMBOL.grid,
            indicator_size: SETTINGS.popup_chevron as f32,
            chevron_size: SETTINGS.dropdown_chevron_glyph as f32,
            indicator_shape: Shape::Circle,
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            indicator: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            chevron: StateValue::new(ShapeStyle::Role(Role::Label)),
            indicator_border: None,
            indicator_shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
        }
    }
}

use ctappearance::Resolved;
use ctdesign::{BODY, Font, SETTINGS, SYMBOL};

use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct PopUpButtonStyle {
    pub height: f32,
    pub gap: f32,
    pub icon_size: f32,
    pub indicator_size: f32,
    pub chevron_size: f32,
    pub indicator_shape: Shape,
    pub label: StateValue<ShapeStyle>,
    pub indicator: StateValue<ShapeStyle>,
    pub chevron: StateValue<ShapeStyle>,
    pub indicator_border: Option<(ShapeStyle, f32)>,
    pub indicator_shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
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

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

use ctappearance::Resolved;
use ctdesign::COMBO_BOX;

use super::button::rounded;
use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct ComboBoxStyle {
    pub button_size: (f32, f32),
    pub button_inset: f32,
    pub text_gap: f32,
    pub chevron_size: f32,
    pub button_shape: Shape,
    pub button: StateValue<ShapeStyle>,
    pub chevron: StateValue<ShapeStyle>,
    pub button_border: Option<(ShapeStyle, f32)>,
    pub button_shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
}

impl ControlStyle for ComboBoxStyle {
    fn tengen() -> Self {
        Self {
            button_size: (COMBO_BOX.button_width, COMBO_BOX.button_height),
            button_inset: COMBO_BOX.button_inset_x,
            text_gap: COMBO_BOX.text_gap,
            chevron_size: COMBO_BOX.chevron,
            button_shape: rounded(COMBO_BOX.button_radius),
            button: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            chevron: StateValue::new(ShapeStyle::Role(Role::Label)),
            button_border: None,
            button_shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

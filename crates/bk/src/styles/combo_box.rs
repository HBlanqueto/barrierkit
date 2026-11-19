use ctdesign::COMBO_BOX;

use super::button::rounded;
use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

/// The field is a text field: it draws from the `TextFieldStyle` in context, whose
/// `misspelling` has no effect and whose `opacity` dims the whole combo box.
/// `ComboBoxStyle` holds the arrow button.
///
/// States: the field is hovered and focused; the button has only `normal`.
#[derive(Clone, Debug)]
pub struct ComboBoxStyle {
    /// Tengen: `(COMBO_BOX.button_width, COMBO_BOX.button_height)`.
    pub button_size: (f32, f32),
    /// Tengen, with `text_gap`: `COMBO_BOX.button_inset_x` from the trailing edge,
    /// `COMBO_BOX.text_gap` between the text and the button.
    pub button_inset: f32,
    /// Tengen, with `button_inset`: `COMBO_BOX.button_inset_x` from the trailing edge,
    /// `COMBO_BOX.text_gap` between the text and the button.
    pub text_gap: f32,
    /// Tengen: `COMBO_BOX.chevron`.
    pub chevron_size: f32,
    /// Tengen: `RoundedRectangle { radius: COMBO_BOX.button_radius, style: Circular }`.
    pub button_shape: Shape,
    /// Tengen: `Role(ControlBackground)`.
    pub button: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub chevron: StateValue<ShapeStyle>,
    /// Tengen: `None`.
    pub button_border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub button_shadows: Vec<ShadowStyle>,
    /// Tengen, with `effect`: `1.0`, `system_effect`, for the button.
    pub opacity: f32,
    /// Tengen, with `opacity`: `1.0`, `system_effect`, for the button.
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
}

use ctdesign::{BODY, COMBO_BOX, ControlSize, Font, TEXT_FIELD};

use super::button::continuous_rounded;
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
    /// Tengen: `TEXT_FIELD.height`, of the field.
    pub height: f32,
    /// Tengen: `BODY`, of the field.
    pub font: Font,
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
    /// Tengen: `COMBO_BOX.chevron_stroke`, in points.
    pub chevron_stroke: f32,
    /// Tengen: `RoundedRectangle { radius: COMBO_BOX.button_radius, style: Continuous }`.
    pub button_shape: Shape,
    /// Tengen: `Role(ComboButton)`.
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

impl ComboBoxStyle {
    pub(crate) fn sized(self, size: ControlSize) -> Self {
        let row = match size {
            ControlSize::Mini => COMBO_BOX.mini,
            ControlSize::Small => COMBO_BOX.small,
            ControlSize::Regular => return self,
            ControlSize::Large => COMBO_BOX.large,
            ControlSize::ExtraLarge => COMBO_BOX.xl,
        };
        Self {
            height: row.height,
            font: Font {
                size: row.font_size,
                line_height: row.line_height,
                ..self.font
            },
            button_size: (row.button_width, row.button_height),
            button_inset: row.button_inset_x,
            chevron_size: row.chevron,
            chevron_stroke: row.chevron_stroke,
            button_shape: continuous_rounded(row.button_radius),
            ..self
        }
    }
}

impl ControlStyle for ComboBoxStyle {
    fn tengen() -> Self {
        Self {
            height: TEXT_FIELD.height,
            font: BODY,
            button_size: (COMBO_BOX.button_width, COMBO_BOX.button_height),
            button_inset: COMBO_BOX.button_inset_x,
            text_gap: COMBO_BOX.text_gap,
            chevron_size: COMBO_BOX.chevron,
            chevron_stroke: COMBO_BOX.chevron_stroke,
            button_shape: continuous_rounded(COMBO_BOX.button_radius),
            button: StateValue::new(ShapeStyle::Role(Role::ComboButton))
                .disabled(ShapeStyle::Role(Role::ControlBackgroundDisabled)),
            chevron: StateValue::new(ShapeStyle::Role(Role::Label))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            button_border: None,
            button_shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
        }
    }
}

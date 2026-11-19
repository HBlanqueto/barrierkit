use ctdesign::{BODY, BUTTON, Font, HAIRLINE, SPACE_2, SPLIT_VIEW};

use super::button::rounded;
use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

/// The color in the well and in the panel is content: no paint or `effect`
/// changes it. The hex text keeps its contrast with the color.
///
/// States: the well is hovered, pressed and focused. The sliders of the panel
/// are `LevelSlider`s with Tengen's `LevelSliderStyle`.
///
/// `FontPicker` is a `Button`: its `.style` takes a `ButtonStyle`. The font
/// panel it opens has no style.
#[derive(Clone, Debug)]
pub struct ColorPickerStyle {
    /// Tengen: `(BUTTON.height × 2, BUTTON.height)`; the panel's preview takes its height.
    pub size: (f32, f32),
    /// Tengen: `(SPLIT_VIEW.inspector, BODY.line_height × 7)`.
    pub panel_size: (f32, f32),
    /// Tengen: `SPACE_2`, between the rows of the panel.
    pub spacing: f32,
    /// Tengen: `RoundedRectangle { radius: BUTTON.radius, style: Circular }`, the well and the
    /// preview.
    pub shape: Shape,
    /// Tengen: `Role(SecondaryLabel)`, the names of the sliders.
    pub caption: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(CardBorder), HAIRLINE))`.
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

impl ControlStyle for ColorPickerStyle {
    fn tengen() -> Self {
        Self {
            size: (BUTTON.height * 2.0, BUTTON.height),
            panel_size: (SPLIT_VIEW.inspector, BODY.line_height as f32 * 7.0),
            spacing: SPACE_2,
            shape: rounded(BUTTON.radius),
            caption: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            border: Some((ShapeStyle::Role(Role::CardBorder), HAIRLINE)),
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
        }
    }
}

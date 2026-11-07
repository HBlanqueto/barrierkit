use ctdesign::{CAPTION, Font, HAIRLINE, SETTINGS, SWITCH};

use super::{
    Axis, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    system_effect,
};

/// The colors of the swatches and the previews of the cards are content: no
/// paint or `effect` changes them.
///
/// States: each swatch and card is hovered, pressed, focused and disabled on its
/// own.
#[derive(Clone, Debug)]
pub struct ColorWellStyle {
    /// Tengen: `SETTINGS.swatch_size`.
    pub swatch_size: f32,
    /// Tengen: `SETTINGS.swatch_column`, the width of a swatch with its name.
    pub column_width: f32,
    /// Tengen: `SETTINGS.swatch_name_gap`.
    pub name_gap: f32,
    /// Tengen: `SETTINGS.swatch_gap`.
    pub spacing: f32,
    /// Tengen: `Circle`.
    pub swatch_shape: Shape,
    /// Tengen: `Role(Label)`, the ring of the chosen swatch.
    pub selection: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`, shown only under the chosen swatch.
    pub name: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(Separator), HAIRLINE))`.
    pub swatch_border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub swatch_shadows: Vec<ShadowStyle>,
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
    /// Tengen: `CAPTION`.
    pub name_font: Font,
}

impl ControlStyle for ColorWellStyle {
    fn tengen() -> Self {
        Self {
            swatch_size: SETTINGS.swatch_size as f32,
            column_width: SETTINGS.swatch_column as f32,
            name_gap: SETTINGS.swatch_name_gap as f32,
            spacing: SETTINGS.swatch_gap as f32,
            swatch_shape: Shape::Circle,
            selection: StateValue::new(ShapeStyle::Role(Role::Label)),
            name: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            swatch_border: Some((ShapeStyle::Role(Role::Separator), HAIRLINE)),
            swatch_shadows: Vec::new(),
            opacity: 1.0,
            disabled_opacity: SWITCH.disabled_opacity,
            effect: system_effect,
            axis: Axis::Horizontal,
            wraps: false,
            name_font: CAPTION,
        }
    }
}

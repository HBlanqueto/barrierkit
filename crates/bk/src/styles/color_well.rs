use ctdesign::{CAPTION, Font, HAIRLINE, SETTINGS, SWITCH};

use crate::styles::{Axis, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue};

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
    /// Tengen: `Accent`, the ring of the chosen swatch, like macOS.
    pub selection: StateValue<ShapeStyle>,
    /// Tengen: `SETTINGS.swatch.ring_gap`, between the swatch and its ring.
    pub selection_gap: f32,
    /// Tengen: `Role(SecondaryLabel)`, shown only under the chosen swatch.
    pub name: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(Outline), HAIRLINE))`.
    pub swatch_border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub swatch_shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `SWITCH.disabled_opacity`.
    pub disabled_opacity: f32,
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
            selection: StateValue::new(ShapeStyle::Accent),
            selection_gap: SETTINGS.swatch_ring_gap as f32,
            name: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            swatch_border: Some((ShapeStyle::Role(Role::Outline), HAIRLINE)),
            swatch_shadows: Vec::new(),
            opacity: 1.0,
            disabled_opacity: SWITCH.disabled_opacity,
            axis: Axis::Horizontal,
            wraps: false,
            name_font: CAPTION,
        }
    }
}

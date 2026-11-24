use ctdesign::{BODY, BUTTON, COLOR_PICKER, Font, SPACE_2, SPLIT_VIEW};

use crate::styles::button::{continuous_rounded, rounded};
use crate::styles::{ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue};

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
    /// Tengen: `RoundedRectangle { radius: BUTTON.radius, style: Continuous }`, the preview of the
    /// panel.
    pub shape: Shape,
    /// Tengen: `Capsule`, the well.
    pub well_shape: Shape,
    /// Tengen: `Role(SecondaryLabel)`, the names of the sliders.
    pub caption: StateValue<ShapeStyle>,
    /// Tengen: `Role(SystemFill)`, the bezel around the swatch.
    pub bezel: StateValue<ShapeStyle>,
    /// Tengen: `COLOR_PICKER.inset`, between the bezel and the swatch with its ring.
    pub inset: f32,
    /// Tengen: `COLOR_PICKER.ring_shift`: how much each channel of the color moves for the ring
    /// around the swatch, darker in light and lighter in dark.
    pub ring_shift: f32,
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `false`; the color wheel button beside the swatch of the expanded style.
    pub wheel: bool,
    /// Tengen: `Role(SystemFill)`, behind the color wheel.
    pub wheel_background: StateValue<ShapeStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `BODY`.
    pub font: Font,
}

impl ControlStyle for ColorPickerStyle {
    fn tengen() -> Self {
        Self {
            size: (BUTTON.height * 2.0, BUTTON.height),
            panel_size: (SPLIT_VIEW.inspector, BODY.line_height as f32 * 7.0),
            spacing: SPACE_2,
            shape: continuous_rounded(BUTTON.radius),
            well_shape: Shape::Capsule,
            caption: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            bezel: StateValue::new(ShapeStyle::Role(Role::SystemFill)),
            inset: COLOR_PICKER.inset,
            ring_shift: COLOR_PICKER.ring_shift,
            shadows: Vec::new(),
            wheel: false,
            wheel_background: StateValue::new(ShapeStyle::Role(Role::SystemFill)),
            opacity: 1.0,
            font: BODY,
        }
    }
}

impl ColorPickerStyle {
    pub fn minimal(self) -> Self {
        Self {
            bezel: StateValue::new(ShapeStyle::Clear),
            inset: 0.0,
            ..self
        }
    }

    pub fn expanded(self) -> Self {
        Self {
            size: (COLOR_PICKER.expanded_width, self.size.1),
            well_shape: rounded(COLOR_PICKER.expanded_radius),
            wheel: true,
            ..self.minimal()
        }
    }
}

use ctdesign::{
    BODY, ControlSize, Font, HAIRLINE, STEPPER, STEPPER_DIVIDER_CONTRAST_DARK,
    STEPPER_DIVIDER_CONTRAST_DARK_TINT, STEPPER_DIVIDER_CONTRAST_LIGHT, STEPPER_DIVIDER_DARK,
    STEPPER_DIVIDER_DARK_TINT, STEPPER_DIVIDER_LIGHT,
};

use super::button::rounded;
use super::{
    ControlStyle, Edge, Role, ShadowStyle, Shape, ShapeStyle, StateValue, disabled_palette,
    per_look,
};

/// States: the stepper is hovered, pressed and focused; each arrow is disabled at
/// its limit. The arrows of a `DatePicker` draw from the `StepperStyle` in
/// context, with only `normal` and that disabled state.
#[derive(Clone, Debug)]
pub struct StepperStyle {
    /// Tengen: `(STEPPER.width, STEPPER.height)`.
    pub arrows_size: (f32, f32),
    /// Tengen: `STEPPER.chevron`.
    pub chevron_size: f32,
    /// Tengen: `STEPPER.chevron_stroke`, in points.
    pub chevron_stroke: f32,
    /// Tengen: `(STEPPER.chevron_offset_up, STEPPER.chevron_offset_down)`: how far each chevron sits below the center of its half.
    pub chevron_offsets: (f32, f32),
    /// Tengen: `STEPPER.separator_inset`, on each side of the line between the chevrons.
    pub separator_inset: f32,
    /// Tengen: `STEPPER.separator_drop`, how far the line sits below the center of the arrows.
    pub separator_drop: f32,
    /// Tengen: `HAIRLINE`.
    pub separator_height: f32,
    /// Tengen: `STEPPER.gap`.
    pub value_gap: f32,
    /// Tengen: `RoundedRectangle { radius: STEPPER.radius, style: Circular }`, the arrows.
    pub shape: Shape,
    /// Tengen: `Role(ControlBackground)` (disabled `Role(ControlBackgroundDisabled)`).
    pub background: StateValue<ShapeStyle>,
    /// Tengen, with `value`: `Role(Label)` (disabled `Color(DisabledPalette.stepper_chevron)`), `Role(Label)`.
    pub chevron: StateValue<ShapeStyle>,
    /// Tengen: `Color(STEPPER_DIVIDER)`, `STEPPER_DIVIDER_CONTRAST` under Increase Contrast.
    pub separator: StateValue<ShapeStyle>,
    /// Tengen, with `chevron`: `Role(Label)` (disabled `Color(DisabledPalette.stepper_chevron)`), `Role(Label)`.
    pub value: StateValue<ShapeStyle>,
    /// Tengen: `None`.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `Some(Edge::Leading)`; `None` hides the value, which stays accessible.
    pub value_position: Option<Edge>,
    /// Tengen: `BODY`.
    pub value_font: Font,
}

impl StepperStyle {
    pub(crate) fn sized(self, size: ControlSize) -> Self {
        let row = STEPPER.size(size);
        Self {
            arrows_size: (row.width, row.height),
            chevron_size: row.chevron,
            chevron_stroke: row.chevron_stroke,
            shape: rounded(row.radius),
            ..self
        }
    }
}

impl ControlStyle for StepperStyle {
    fn tengen() -> Self {
        Self {
            arrows_size: (STEPPER.width, STEPPER.height),
            chevron_size: STEPPER.chevron,
            chevron_stroke: STEPPER.chevron_stroke,
            chevron_offsets: (STEPPER.chevron_offset_up, STEPPER.chevron_offset_down),
            separator_inset: STEPPER.separator_inset,
            separator_drop: STEPPER.separator_drop,
            separator_height: HAIRLINE,
            value_gap: STEPPER.gap,
            shape: rounded(STEPPER.radius),
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground))
                .disabled(ShapeStyle::Role(Role::ControlBackgroundDisabled)),
            chevron: StateValue::new(ShapeStyle::Role(Role::Label))
                .disabled(ShapeStyle::Color(disabled_palette().stepper_chevron)),
            separator: StateValue::new(ShapeStyle::Color(per_look(
                [
                    STEPPER_DIVIDER_LIGHT,
                    STEPPER_DIVIDER_DARK,
                    STEPPER_DIVIDER_DARK_TINT,
                ],
                [
                    STEPPER_DIVIDER_CONTRAST_LIGHT,
                    STEPPER_DIVIDER_CONTRAST_DARK,
                    STEPPER_DIVIDER_CONTRAST_DARK_TINT,
                ],
            ))),
            value: StateValue::new(ShapeStyle::Role(Role::Label)),
            border: None,
            shadows: Vec::new(),
            opacity: 1.0,
            value_position: Some(Edge::Leading),
            value_font: BODY,
        }
    }
}

use ctdesign::{BODY, Font, HAIRLINE, STEPPER};

use super::button::rounded;
use super::{
    ColorEffect, ControlStyle, Edge, LayoutDirection, Role, ShadowStyle, Shape, ShapeStyle,
    StateValue, system_effect,
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
    /// Tengen: `HAIRLINE`.
    pub separator_height: f32,
    /// Tengen: `STEPPER.gap`.
    pub value_gap: f32,
    /// Tengen: `RoundedRectangle { radius: STEPPER.radius, style: Circular }`, the arrows.
    pub shape: Shape,
    /// Tengen: `Role(ControlBackground)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen, with `value`: `Role(Label)` (disabled `Role(TertiaryLabel)`), `Role(Label)`.
    pub chevron: StateValue<ShapeStyle>,
    /// Tengen: `Role(Separator)`.
    pub separator: StateValue<ShapeStyle>,
    /// Tengen, with `chevron`: `Role(Label)` (disabled `Role(TertiaryLabel)`), `Role(Label)`.
    pub value: StateValue<ShapeStyle>,
    /// Tengen: `None`.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `Some(Edge::Leading)`; `None` hides the value, which stays accessible.
    pub value_position: Option<Edge>,
    /// Tengen: `LeftToRight`; `RightToLeft` swaps `Leading` and `Trailing`.
    pub layout_direction: LayoutDirection,
    /// Tengen: `BODY`.
    pub value_font: Font,
}

impl ControlStyle for StepperStyle {
    fn tengen() -> Self {
        Self {
            arrows_size: (STEPPER.width, STEPPER.height),
            chevron_size: STEPPER.chevron,
            separator_height: HAIRLINE,
            value_gap: STEPPER.gap,
            shape: rounded(STEPPER.radius),
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            chevron: StateValue::new(ShapeStyle::Role(Role::Label))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            separator: StateValue::new(ShapeStyle::Role(Role::Separator)),
            value: StateValue::new(ShapeStyle::Role(Role::Label)),
            border: None,
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
            value_position: Some(Edge::Leading),
            layout_direction: LayoutDirection::LeftToRight,
            value_font: BODY,
        }
    }
}

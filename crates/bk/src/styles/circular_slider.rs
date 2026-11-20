use ctdesign::CIRCULAR_SLIDER;

use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};

/// States: hovered, pressed, focused, disabled.
#[derive(Clone, Debug)]
pub struct CircularSliderStyle {
    /// Tengen: `CIRCULAR_SLIDER.size`.
    pub size: f32,
    /// Tengen: `(CIRCULAR_SLIDER.dash_from, CIRCULAR_SLIDER.dash_to, CIRCULAR_SLIDER.dash_width)`:
    /// where the dash that shows the value starts and ends from the center, and how thick it is.
    pub dash: (f32, f32, f32),
    /// Tengen: `Circle`.
    pub shape: Shape,
    /// Tengen: `Role(ControlBackground)` (disabled `Role(ControlBackgroundDisabled)`).
    pub disc: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)` (disabled `Role(TertiaryLabel)`).
    pub dash_ink: StateValue<ShapeStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
}

impl ControlStyle for CircularSliderStyle {
    fn tengen() -> Self {
        Self {
            size: CIRCULAR_SLIDER.size,
            dash: (
                CIRCULAR_SLIDER.dash_from,
                CIRCULAR_SLIDER.dash_to,
                CIRCULAR_SLIDER.dash_width,
            ),
            shape: Shape::Circle,
            disc: StateValue::new(ShapeStyle::Role(Role::ControlBackground))
                .disabled(ShapeStyle::Role(Role::ControlBackgroundDisabled)),
            dash_ink: StateValue::new(ShapeStyle::Role(Role::Label))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            opacity: 1.0,
            effect: system_effect,
        }
    }
}

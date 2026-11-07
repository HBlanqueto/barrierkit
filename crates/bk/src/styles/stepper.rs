use ctdesign::{BODY, Font, HAIRLINE, STEPPER};

use super::button::rounded;
use super::{
    ColorEffect, ControlStyle, Edge, LayoutDirection, Role, ShadowStyle, Shape, ShapeStyle,
    StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct StepperStyle {
    pub arrows_size: (f32, f32),
    pub chevron_size: f32,
    pub separator_height: f32,
    pub value_gap: f32,
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub chevron: StateValue<ShapeStyle>,
    pub separator: StateValue<ShapeStyle>,
    pub value: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
    pub value_position: Option<Edge>,
    pub layout_direction: LayoutDirection,
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

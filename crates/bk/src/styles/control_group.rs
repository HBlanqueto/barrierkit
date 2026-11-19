use ctdesign::{BUTTON, HAIRLINE};

use super::button::rounded;
use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

/// States: only `normal`. The children keep their own styles: a `Button` inside
/// draws from its `ButtonStyle`.
#[derive(Clone, Debug)]
pub struct ControlGroupStyle {
    /// Tengen: `BUTTON.height`.
    pub height: f32,
    /// Tengen: `(HAIRLINE, BUTTON.height / 2)`, between children.
    pub separator_size: (f32, f32),
    /// Tengen: `RoundedRectangle { radius: BUTTON.radius, style: Circular }`.
    pub shape: Shape,
    /// Tengen: `Role(ControlBackground)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Separator)`.
    pub separator: StateValue<ShapeStyle>,
    /// Tengen: `None`.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
}

impl ControlStyle for ControlGroupStyle {
    fn tengen() -> Self {
        Self {
            height: BUTTON.height,
            separator_size: (HAIRLINE, BUTTON.height / 2.0),
            shape: rounded(BUTTON.radius),
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            separator: StateValue::new(ShapeStyle::Role(Role::Separator)),
            border: None,
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
        }
    }
}

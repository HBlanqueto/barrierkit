use ctdesign::{BADGE, CAPTION, Font};

use crate::styles::{ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue};

#[derive(Clone, Debug)]
pub struct BadgeStyle {
    /// Tengen: `BADGE.height`.
    pub height: f32,
    /// Tengen: `BADGE.padding_x`; the badge is at least as wide as it is tall.
    pub padding_x: f32,
    /// Tengen: `Capsule`.
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `None`.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `font` `CAPTION`.
    pub font: Font,
}

impl ControlStyle for BadgeStyle {
    fn tengen() -> Self {
        Self {
            height: BADGE.height,
            padding_x: BADGE.padding_x,
            shape: Shape::Capsule,
            background: StateValue::new(ShapeStyle::Role(Role::Destructive)),
            label: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            border: None,
            shadows: Vec::new(),
            font: CAPTION,
        }
    }
}

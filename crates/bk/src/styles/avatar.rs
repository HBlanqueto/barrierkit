use ctappearance::backdrop::ink_on;
use ctdesign::{Font, HEADLINE, SETTINGS};

use crate::style::Palette;
use crate::styles::{ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue};

#[derive(Clone, Debug)]
pub struct AvatarStyle {
    /// Tengen: `size`: `SETTINGS.account_avatar`, a square.
    pub size: f32,
    /// Tengen: `Circle`.
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `None`.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `font` `HEADLINE`.
    pub font: Font,
}

impl ControlStyle for AvatarStyle {
    fn tengen() -> Self {
        Self {
            size: SETTINGS.account_avatar as f32,
            shape: Shape::Circle,
            background: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            label: StateValue::new(ShapeStyle::Color(ink_on(
                Palette::current().secondary_label,
            ))),
            border: None,
            shadows: Vec::new(),
            font: HEADLINE,
        }
    }
}

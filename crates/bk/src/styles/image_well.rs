use ctdesign::{HAIRLINE, IMAGE_WELL};

use super::button::rounded;
use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

/// States: hovered, pressed, focused. The image is content.
#[derive(Clone, Debug)]
pub struct ImageWellStyle {
    /// Tengen: `IMAGE_WELL.size`, a square.
    pub size: f32,
    /// Tengen: `IMAGE_WELL.inset` around the image, whose corners are the well's minus `inset`.
    pub inset: f32,
    /// Tengen: `RoundedRectangle { radius: IMAGE_WELL.radius, style: Circular }`.
    pub shape: Shape,
    /// Tengen: `Role(ContentBackground)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(Separator), HAIRLINE))`.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: `(Accent, IMAGE_WELL.drop_border)`, in place of `border` while an image is
    /// dragged over it.
    pub drop_border: (ShapeStyle, f32),
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
}

impl ControlStyle for ImageWellStyle {
    fn tengen() -> Self {
        Self {
            size: IMAGE_WELL.size,
            inset: IMAGE_WELL.inset,
            shape: rounded(IMAGE_WELL.radius),
            background: StateValue::new(ShapeStyle::Role(Role::ContentBackground)),
            border: Some((ShapeStyle::Role(Role::Separator), HAIRLINE)),
            drop_border: (ShapeStyle::Accent, IMAGE_WELL.drop_border),
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
        }
    }
}

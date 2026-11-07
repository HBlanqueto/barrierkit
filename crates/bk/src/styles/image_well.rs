use ctappearance::Resolved;
use ctdesign::{HAIRLINE, IMAGE_WELL};

use super::button::rounded;
use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct ImageWellStyle {
    pub size: f32,
    pub inset: f32,
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub drop_border: (ShapeStyle, f32),
    pub shadows: Vec<ShadowStyle>,
    pub opacity: f32,
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

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

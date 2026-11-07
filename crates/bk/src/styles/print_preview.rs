use ctappearance::Resolved;
use ctdesign::{CAPTION, Font, SPACE_2, SPACE_3};

use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};
use crate::style::Palette;

#[derive(Clone, Debug)]
pub struct PrintPreviewStyle {
    pub padding: f32,
    pub spacing: f32,
    pub caption_gap: f32,
    pub page_shape: Shape,
    pub page: StateValue<ShapeStyle>,
    pub caption: StateValue<ShapeStyle>,
    pub page_shadows: Vec<ShadowStyle>,
    pub effect: ColorEffect,
    pub caption_font: Font,
}

impl ControlStyle for PrintPreviewStyle {
    fn tengen() -> Self {
        Self {
            padding: SPACE_3,
            spacing: SPACE_3,
            caption_gap: SPACE_2,
            page_shape: Shape::Rectangle,
            page: StateValue::new(ShapeStyle::Role(Role::ContentBackground)),
            caption: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            page_shadows: vec![ShadowStyle {
                color: Palette::current().separator,
                x: 0.0,
                y: 0.0,
                blur: SPACE_2,
                spread: 0.0,
                inner: false,
            }],
            effect: system_effect,
            caption_font: CAPTION,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

use ctdesign::{CAPTION, Font, SPACE_2, SPACE_3};

use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};
use crate::style::Palette;

/// The pages are content: no paint or `effect` changes them.
///
/// States: only `normal`.
#[derive(Clone, Debug)]
pub struct PrintPreviewStyle {
    /// Tengen, with `spacing`, `caption_gap`: `SPACE_3` around the pages, `SPACE_3` between

    /// them, `SPACE_2` between a page and its number.
    pub padding: f32,
    /// Tengen, with `padding`, `caption_gap`: `SPACE_3` around the pages, `SPACE_3` between
    /// them, `SPACE_2` between a page and its number.
    pub spacing: f32,
    /// Tengen, with `padding`, `spacing`: `SPACE_3` around the pages, `SPACE_3` between them,
    /// `SPACE_2` between a page and its number.
    pub caption_gap: f32,
    /// Tengen: `Rectangle`.
    pub page_shape: Shape,
    /// Tengen, with `caption`: `Role(ContentBackground)` behind each page,
    /// `Role(SecondaryLabel)` for the page number.
    pub page: StateValue<ShapeStyle>,
    /// Tengen, with `page`: `Role(ContentBackground)` behind each page, `Role(SecondaryLabel)`
    /// for the page number.
    pub caption: StateValue<ShapeStyle>,
    /// Tengen: one shadow in the separator color with `SPACE_2` of blur.
    pub page_shadows: Vec<ShadowStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `CAPTION`.
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
}

use ctdesign::{BODY, CAPTION, Font, HAIRLINE, SPACE_1, SPACE_2, TEXT_FIELD};

use super::text_field::field_shape;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct TextEditorStyle {
    pub padding_x: f32,
    pub padding_y: f32,
    pub spacing: f32,
    pub find_bar_spacing: f32,
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub text: StateValue<ShapeStyle>,
    pub placeholder: StateValue<ShapeStyle>,
    pub link: StateValue<ShapeStyle>,
    pub misspelling: StateValue<ShapeStyle>,
    pub caption: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub opacity: f32,
    pub effect: ColorEffect,
    pub font: Font,
    pub caption_font: Font,
}

impl ControlStyle for TextEditorStyle {
    fn tengen() -> Self {
        Self {
            padding_x: TEXT_FIELD.padding_x,
            padding_y: (TEXT_FIELD.height - BODY.line_height as f32) / 2.0,
            spacing: SPACE_1,
            find_bar_spacing: SPACE_2,
            shape: field_shape(),
            background: StateValue::new(ShapeStyle::Role(Role::ContentBackground)),
            text: StateValue::new(ShapeStyle::Role(Role::Label)),
            placeholder: StateValue::new(ShapeStyle::Role(Role::TertiaryLabel)),
            link: StateValue::new(ShapeStyle::Accent),
            misspelling: StateValue::new(ShapeStyle::Role(Role::Destructive)),
            caption: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            border: Some((ShapeStyle::Role(Role::Separator), HAIRLINE)),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
            caption_font: CAPTION,
        }
    }
}

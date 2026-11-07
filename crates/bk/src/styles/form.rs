use ctappearance::Resolved;
use ctdesign::{
    BODY, CAPTION, Font, HAIRLINE, HEADLINE, LIST_GROUP, Rgba, SPACE_2, SPACE_3, SPACE_4,
};

use super::button::rounded;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};
use crate::style::Palette;

#[derive(Clone, Debug)]
pub struct FormStyle {
    pub padding: [f32; 4],
    pub section_spacing: f32,
    pub header_padding: [f32; 4],
    pub row_min_height: f32,
    pub row_padding: (f32, f32),
    pub row_spacing: f32,
    pub separator_inset: f32,
    pub chevron_size: f32,
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub row_background: StateValue<ShapeStyle>,
    pub highlight: StateValue<ShapeStyle>,
    pub header: StateValue<ShapeStyle>,
    pub footer: StateValue<ShapeStyle>,
    pub title: StateValue<ShapeStyle>,
    pub destructive_title: StateValue<ShapeStyle>,
    pub detail: StateValue<ShapeStyle>,
    pub value: StateValue<ShapeStyle>,
    pub chevron: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub effect: ColorEffect,
    pub header_font: Font,
    pub footer_font: Font,
    pub title_font: Font,
    pub detail_font: Font,
    pub value_font: Font,
}

impl ControlStyle for FormStyle {
    fn tengen() -> Self {
        Self {
            padding: LIST_GROUP.padding,
            section_spacing: SPACE_4,
            header_padding: LIST_GROUP.header_padding,
            row_min_height: LIST_GROUP.row_min_height,
            row_padding: (LIST_GROUP.row_padding_x, SPACE_2),
            row_spacing: SPACE_3,
            separator_inset: LIST_GROUP.separator_inset,
            chevron_size: CAPTION.size as f32,
            shape: rounded(LIST_GROUP.radius),
            background: StateValue::new(ShapeStyle::Role(Role::ContentBackground)),
            row_background: StateValue::new(ShapeStyle::Clear)
                .hovered(ShapeStyle::Role(Role::ControlHover)),
            highlight: StateValue::new(ShapeStyle::Color(Rgba {
                a: LIST_GROUP.highlight,
                ..Palette::current().accent
            })),
            header: StateValue::new(ShapeStyle::Role(Role::Label)),
            footer: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            title: StateValue::new(ShapeStyle::Role(Role::Label)),
            destructive_title: StateValue::new(ShapeStyle::Role(Role::Destructive)),
            detail: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            value: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            chevron: StateValue::new(ShapeStyle::Role(Role::TertiaryLabel)),
            border: Some((ShapeStyle::Role(Role::CardBorder), HAIRLINE)),
            effect: system_effect,
            header_font: HEADLINE,
            footer_font: CAPTION,
            title_font: BODY,
            detail_font: CAPTION,
            value_font: BODY,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

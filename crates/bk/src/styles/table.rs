use ctdesign::{BODY, CAPTION, Font, HAIRLINE, Rgba, SPACE_1, TABLE};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};
use crate::style::Palette;

#[derive(Clone, Debug)]
pub struct TableStyle {
    pub header_height: f32,
    pub row_height: f32,
    pub padding_x: f32,
    pub header_spacing: f32,
    pub header_label: StateValue<ShapeStyle>,
    pub sorted_header_label: StateValue<ShapeStyle>,
    pub row_background: StateValue<ShapeStyle>,
    pub alternate_row_background: StateValue<ShapeStyle>,
    pub selected_row_background: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub selected_label: StateValue<ShapeStyle>,
    pub header_border: Option<(ShapeStyle, f32)>,
    pub effect: ColorEffect,
    pub header_font: Font,
    pub font: Font,
}

impl ControlStyle for TableStyle {
    fn tengen() -> Self {
        Self {
            header_height: TABLE.header_height,
            row_height: TABLE.row_height,
            padding_x: TABLE.padding_x,
            header_spacing: SPACE_1,
            header_label: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            sorted_header_label: StateValue::new(ShapeStyle::Role(Role::Label)),
            row_background: StateValue::new(ShapeStyle::Clear),
            alternate_row_background: StateValue::new(ShapeStyle::Color(Rgba {
                a: TABLE.zebra,
                ..Palette::current().label
            })),
            selected_row_background: StateValue::new(ShapeStyle::Accent),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            selected_label: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            header_border: Some((ShapeStyle::Role(Role::Separator), HAIRLINE)),
            effect: system_effect,
            header_font: CAPTION,
            font: BODY,
        }
    }
}

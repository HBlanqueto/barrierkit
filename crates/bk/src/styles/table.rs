use ctdesign::{BODY, CAPTION, Font, HAIRLINE, Rgba, SPACE_1, TABLE};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};
use crate::style::Palette;

/// `OutlineViewStyle` is one style for `OutlineView` and `Browser`, which draw
/// the same rows; the rows of the font panel of `FontPicker` take it too.
///
/// States: a header cell takes hovered, pressed and focused; rows only `normal`.
/// The resize handle and the minimum column width stay `TABLE.resize_handle` and
/// `TABLE.min_column_width`: they are behavior.
///
/// States: tiles take hovered, pressed and focused.
///
/// States: only `normal`; selection has its own fields.
#[derive(Clone, Debug)]
pub struct TableStyle {
    /// Tengen, with `row_height`, `padding_x`, `header_spacing`: `TABLE.header_height`,

    /// `TABLE.row_height`, `TABLE.padding_x` on each side of a cell, `SPACE_1` between a

    /// header title and its arrow.
    pub header_height: f32,
    /// Tengen, with `header_height`, `padding_x`, `header_spacing`: `TABLE.header_height`,
    /// `TABLE.row_height`, `TABLE.padding_x` on each side of a cell, `SPACE_1` between a header
    /// title and its arrow.
    pub row_height: f32,
    /// Tengen, with `header_height`, `row_height`, `header_spacing`: `TABLE.header_height`,
    /// `TABLE.row_height`, `TABLE.padding_x` on each side of a cell, `SPACE_1` between a header
    /// title and its arrow.
    pub padding_x: f32,
    /// Tengen, with `header_height`, `row_height`, `padding_x`: `TABLE.header_height`,
    /// `TABLE.row_height`, `TABLE.padding_x` on each side of a cell, `SPACE_1` between a header
    /// title and its arrow.
    pub header_spacing: f32,
    /// Tengen, with `sorted_header_label`: `Role(SecondaryLabel)`; `Role(Label)` for the sorted
    /// column, which also paints its arrow.
    pub header_label: StateValue<ShapeStyle>,
    /// Tengen, with `header_label`: `Role(SecondaryLabel)`; `Role(Label)` for the sorted
    /// column, which also paints its arrow.
    pub sorted_header_label: StateValue<ShapeStyle>,
    /// Tengen, with `alternate_row_background`, `selected_row_background`: `Clear` for even
    /// rows; `Color` of the label color at `TABLE.zebra` for odd rows; `Accent`.
    pub row_background: StateValue<ShapeStyle>,
    /// Tengen, with `row_background`, `selected_row_background`: `Clear` for even rows; `Color`
    /// of the label color at `TABLE.zebra` for odd rows; `Accent`.
    pub alternate_row_background: StateValue<ShapeStyle>,
    /// Tengen, with `row_background`, `alternate_row_background`: `Clear` for even rows;
    /// `Color` of the label color at `TABLE.zebra` for odd rows; `Accent`.
    pub selected_row_background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Role(OnAccent)`.
    pub selected_label: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(Separator), HAIRLINE))`, drawn inside the header.
    pub header_border: Option<(ShapeStyle, f32)>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen, with `font`: `CAPTION`, its size also the arrow's; `BODY`.
    pub header_font: Font,
    /// Tengen, with `header_font`: `CAPTION`, its size also the arrow's; `BODY`.
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

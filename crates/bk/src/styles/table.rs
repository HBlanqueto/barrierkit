use ctdesign::{BODY, CAPTION, Font, Rgba, SPACE_1, TABLE};

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
    /// Tengen, with `column_shift`: `TABLE.header_padding_x` on each side of a header cell,
    /// `TABLE.column_shift` more at the leading edge of every column after the first.
    pub header_padding_x: f32,
    /// Tengen, with `header_padding_x`: `TABLE.column_shift`, the extra leading padding of every
    /// column after the first, in the header and in the rows.
    pub column_shift: f32,
    /// Tengen: `Role(Label)`.
    pub header_label: StateValue<ShapeStyle>,
    /// Tengen, with `row_radius`, `body_inset`: `TABLE.row_inset` between the table and the header cells and rows.
    pub row_inset: f32,
    /// Tengen, with `row_inset`, `body_inset`: `TABLE.row_radius`, the corners of a row.
    pub row_radius: f32,
    /// Tengen, with `row_inset`, `row_radius`: `TABLE.body_inset` above the first row.
    pub body_inset: f32,
    /// Tengen: `Some((Role(SeparatorStrong), TABLE.header_separator))`, the 1 wide separator
    /// after each header cell, with its height.
    pub header_separator: Option<(ShapeStyle, f32)>,
    /// Tengen: `Role(Label)` for the sorted column, which also paints its arrow.
    pub sorted_header_label: StateValue<ShapeStyle>,
    /// Tengen: `Clear` for even rows.
    pub row_background: StateValue<ShapeStyle>,
    /// Tengen: `Color` of the label color at `TABLE.zebra` for odd rows.
    pub alternate_row_background: StateValue<ShapeStyle>,
    /// Tengen: `Role(SelectedContent)`.
    pub selected_row_background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Role(OnSelectedContent)`.
    pub selected_label: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(Separator), TABLE.header_line))`, the line under the header, which
    /// reaches 1 into the first row.
    pub header_border: Option<(ShapeStyle, f32)>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `CAPTION` with the weight of `BODY`.
    pub header_font: Font,
    /// Tengen: `header_font` at `TABLE.sorted_weight`, for the sorted column.
    pub sorted_header_font: Font,
    /// Tengen: `(TABLE.sort_arrow, TABLE.sort_arrow_stroke)`.
    pub sort_arrow_size: (f32, f32),
    /// Tengen: `(TABLE.sort_arrow_inset, TABLE.sort_arrow_drop)`: from the right edge of the
    /// header cell, and below its center.
    pub sort_arrow_offset: (f32, f32),
    /// Tengen: `Role(SecondaryLabel)`.
    pub sort_arrow: StateValue<ShapeStyle>,
    /// Tengen: `BODY`.
    pub font: Font,
}

impl ControlStyle for TableStyle {
    fn tengen() -> Self {
        Self {
            header_height: TABLE.header_height,
            row_height: TABLE.row_height,
            padding_x: TABLE.padding_x,
            header_spacing: SPACE_1,
            header_padding_x: TABLE.header_padding_x,
            column_shift: TABLE.column_shift,
            row_inset: TABLE.row_inset,
            row_radius: TABLE.row_radius,
            body_inset: TABLE.body_inset,
            header_label: StateValue::new(ShapeStyle::Role(Role::Label)),
            sorted_header_label: StateValue::new(ShapeStyle::Role(Role::Label)),
            row_background: StateValue::new(ShapeStyle::Clear),
            alternate_row_background: StateValue::new(ShapeStyle::Color(Rgba {
                a: TABLE.zebra,
                ..Palette::current().label
            })),
            selected_row_background: StateValue::new(ShapeStyle::Role(Role::SelectedContent)),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            selected_label: StateValue::new(ShapeStyle::Role(Role::OnSelectedContent)),
            header_border: Some((ShapeStyle::Role(Role::Separator), TABLE.header_line)),
            header_separator: Some((
                ShapeStyle::Role(Role::SeparatorStrong),
                TABLE.header_separator,
            )),
            effect: system_effect,
            header_font: Font {
                weight: BODY.weight,
                ..CAPTION
            },
            sorted_header_font: Font {
                weight: TABLE.sorted_weight as u16,
                ..CAPTION
            },
            sort_arrow_size: (TABLE.sort_arrow, TABLE.sort_arrow_stroke),
            sort_arrow_offset: (TABLE.sort_arrow_inset, TABLE.sort_arrow_drop),
            sort_arrow: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            font: BODY,
        }
    }
}

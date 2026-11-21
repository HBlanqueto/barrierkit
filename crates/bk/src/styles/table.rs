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
    /// Tengen: `Role(SelectedContentInactive)`, `Role(SelectedContent)` while the table has focus.
    pub selected_row_background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`, `Role(OnSelectedContent)` while the table has focus.
    pub selected_label: StateValue<ShapeStyle>,
    /// Tengen: `None`; the list styles draw `Role(Separator)` under every row that is not next to a selected one.
    pub row_separator: Option<ShapeStyle>,
    /// Tengen: `(0, 0)`, the gap before and after a separator inside the row; the list styles take `padding_x` and, in `plain_list`, `(plain_padding_x, 0)`.
    pub separator_margin: (f32, f32),
    /// Tengen: `None`; `bordered_list` draws `(Role(Separator), HAIRLINE)` inside the edge.
    pub border: Option<(ShapeStyle, f32)>,
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

fn stripe() -> StateValue<ShapeStyle> {
    let [light, dark] = TABLE.zebra_contrast;
    let resolved = crate::theme::resolved();
    let alpha = match (resolved.contrast, resolved.scheme) {
        (false, _) => TABLE.zebra,
        (true, ctappearance::ColorScheme::Light) => light,
        (true, _) => dark,
    };
    StateValue::new(ShapeStyle::Color(Rgba {
        a: alpha,
        ..Palette::current().label
    }))
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
            alternate_row_background: stripe(),
            selected_row_background: StateValue::new(ShapeStyle::Role(
                Role::SelectedContentInactive,
            ))
            .focused(ShapeStyle::Role(Role::SelectedContent)),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            selected_label: StateValue::new(ShapeStyle::Role(Role::Label))
                .focused(ShapeStyle::Role(Role::OnSelectedContent)),
            row_separator: None,
            separator_margin: (0.0, 0.0),
            border: None,
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

impl TableStyle {
    pub fn striped(self) -> Self {
        Self {
            alternate_row_background: stripe(),
            row_separator: None,
            ..self
        }
    }

    pub fn inset_list() -> Self {
        Self {
            header_height: 0.0,
            header_border: None,
            header_separator: None,
            body_inset: TABLE.row_inset,
            alternate_row_background: StateValue::new(ShapeStyle::Clear),
            row_separator: Some(ShapeStyle::Role(Role::Separator)),
            separator_margin: (TABLE.padding_x, TABLE.padding_x),
            ..Self::tengen()
        }
    }

    pub fn plain_list() -> Self {
        Self {
            row_inset: 0.0,
            body_inset: 0.0,
            padding_x: TABLE.plain_padding_x,
            row_radius: 0.0,
            separator_margin: (TABLE.plain_padding_x, 0.0),
            ..Self::inset_list()
        }
    }

    pub fn bordered_list() -> Self {
        Self {
            row_inset: HAIRLINE,
            body_inset: HAIRLINE,
            row_radius: 0.0,
            border: Some((ShapeStyle::Role(Role::Separator), HAIRLINE)),
            ..Self::inset_list()
        }
    }
}

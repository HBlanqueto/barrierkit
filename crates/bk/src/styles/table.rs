use ctdesign::{BODY, CAPTION, ControlSize, Font, HAIRLINE, Rgba, SPACE_1, TABLE};

use crate::style::Palette;
use crate::styles::{ControlStyle, Role, ShapeStyle, StateValue};

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
    pub header_height: f32,
    pub row_height: f32,
    pub padding_x: f32,
    pub header_spacing: f32,
    pub header_padding_x: f32,
    pub column_shift: f32,
    /// Tengen: `Role(Label)`.
    pub header_label: StateValue<ShapeStyle>,
    pub row_inset: f32,
    pub row_radius: f32,
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
    pub icon_size: f32,
    pub icon_spacing: f32,
    /// Tengen: `None`; `source_list` takes `TABLE.source_list_row_height`, the height of a row by
    /// small, medium and large row size.
    pub row_heights: Option<[f32; 3]>,
    /// Tengen: `Role(SecondaryLabel)`, the icon of a row; `Accent` in `source_list`.
    pub icon: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`, `Role(OnSelectedContent)` while the table has focus;
    /// `Accent` in `source_list`.
    pub selected_icon: StateValue<ShapeStyle>,
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
            icon_size: TABLE.row_size_icon[1],
            icon_spacing: TABLE.icon_gap,
            row_heights: None,
            icon: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            selected_icon: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel))
                .focused(ShapeStyle::Role(Role::OnSelectedContent)),
        }
    }
}

impl TableStyle {
    /// `NSTableView.Style.fullWidth`: rows and stripes from edge to edge, with square selection.
    pub fn full_width() -> Self {
        Self {
            row_inset: 0.0,
            body_inset: 0.0,
            row_radius: 0.0,
            ..Self::tengen()
        }
    }

    /// `NSTableView.Style.plain`: `full_width` with 17 high rows, the cell 8 in and no extra inset
    /// in later columns.
    pub fn plain() -> Self {
        Self {
            row_height: TABLE.plain_row_height,
            padding_x: TABLE.plain_padding_x,
            column_shift: 0.0,
            ..Self::full_width()
        }
    }

    /// `NSTableView.Style.sourceList`: no header, the rows 10 in from each side and from the top, and
    /// the icons in the accent. The sidebar material behind it is the compositor's.
    pub fn source_list() -> Self {
        Self {
            header_height: 0.0,
            header_border: None,
            header_separator: None,
            body_inset: TABLE.source_list_inset,
            row_height: TABLE.source_list_row_height[1],
            row_heights: Some(TABLE.source_list_row_height),
            alternate_row_background: StateValue::new(ShapeStyle::Clear),
            icon: StateValue::new(ShapeStyle::Accent),
            selected_icon: StateValue::new(ShapeStyle::Accent)
                .focused(ShapeStyle::Role(Role::OnSelectedContent)),
            ..Self::tengen()
        }
    }

    /// `NSTableView.rowSizeStyle`: the font and the icon grow; only a source list changes its rows'
    /// height. `Small`, `Regular` and `Large` are small, medium and large; the other sizes take the
    /// nearest.
    pub fn row_size(self, size: ControlSize) -> Self {
        let index = match size {
            ControlSize::Mini | ControlSize::Small => 0,
            ControlSize::Regular => 1,
            ControlSize::Large | ControlSize::ExtraLarge => 2,
        };
        Self {
            font: Font {
                size: TABLE.row_size_font[index] as u32,
                ..self.font
            },
            icon_size: TABLE.row_size_icon[index],
            row_height: self
                .row_heights
                .map_or(self.row_height, |heights| heights[index]),
            ..self
        }
    }

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

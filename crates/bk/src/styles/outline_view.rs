use ctdesign::{BODY, DISCLOSURE, Font, HAIRLINE, SPACE_1};

use super::{
    ColorEffect, ControlStyle, CornerStyle, Role, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct OutlineViewStyle {
    /// Tengen: `DISCLOSURE.row_height`.
    pub row_height: f32,
    /// Tengen: `DISCLOSURE.indent` per level of an `OutlineView`.
    pub indent: f32,
    /// Tengen: `DISCLOSURE.inset` between the view and its rows, on each side and above the first.
    pub inset: f32,
    /// Tengen: `DISCLOSURE.lead`, the column of the disclosure chevron.
    pub lead_width: f32,
    /// Tengen: `DISCLOSURE.label_gap` between an icon and its title.
    pub spacing: f32,
    /// Tengen: `Some(Role(Separator))`, the line under a row from its title to the end of the row.
    pub row_separator: Option<ShapeStyle>,
    /// Tengen: `DISCLOSURE.separator_margin` between the end of the line and the end of the row.
    pub separator_margin: f32,
    /// Tengen: `DISCLOSURE.icon`.
    pub icon_size: f32,
    /// Tengen: `DISCLOSURE.chevron`, the chevron of a `Browser` row.
    pub chevron_size: f32,
    /// Tengen: `(DISCLOSURE.list_chevron, DISCLOSURE.list_chevron_stroke)`, the disclosure chevron.
    pub triangle_size: (f32, f32),
    /// Tengen: `(DISCLOSURE.list_chevron_x, 0)`, where the closed chevron sits from the center of its column.
    pub chevron_offset: (f32, f32),
    /// Tengen: `(DISCLOSURE.list_chevron_open_x, DISCLOSURE.list_chevron_open_y)`, the same for the open chevron.
    pub open_chevron_offset: (f32, f32),
    /// Tengen: `DISCLOSURE.column_width`.
    pub column_width: f32,
    /// Tengen: `SPACE_1` on each side of a row. `Browser` only.
    pub column_padding_x: f32,
    /// Tengen: `RoundedRectangle { radius: DISCLOSURE.row_radius, style: Continuous }`.
    pub row_shape: Shape,
    /// Tengen: `Clear`.
    pub row_background: StateValue<ShapeStyle>,
    /// Tengen: `Role(SelectedContentInactive)`, `Role(SelectedContent)` while the view has focus.
    pub selected_row_background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`, `Role(OnSelectedContent)` while the view has focus.
    pub selected_label: StateValue<ShapeStyle>,
    /// Tengen: `Accent`.
    pub icon: StateValue<ShapeStyle>,
    /// Tengen: `Accent`, `Role(OnSelectedContent)` while the view has focus.
    pub selected_icon: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`, the disclosure chevron of `OutlineView`.
    pub disclosure: StateValue<ShapeStyle>,
    /// Tengen: `Role(TertiaryLabel)`, the chevron of a `Browser` row with children.
    pub column_chevron: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`, `Role(OnSelectedContent)` on a selected row while the view has focus.
    pub selected_chevron: StateValue<ShapeStyle>,
    /// Tengen: `(Role(Separator), HAIRLINE)`, between `Browser` columns.
    pub column_separator: (ShapeStyle, f32),
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `BODY`.
    pub font: Font,
}

impl OutlineViewStyle {
    /// `NSOutlineView` with the default cell views: gray icons, no row lines and the title 19 after the
    /// disclosure column.
    pub fn classic() -> Self {
        Self {
            row_separator: None,
            lead_width: 15.0,
            spacing: 3.0,
            icon: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            selected_icon: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel))
                .focused(ShapeStyle::Role(Role::OnSelectedContent)),
            ..Self::tengen()
        }
    }

    /// `NSTableView.Style.fullWidth` for an outline: `classic` with square rows from edge to edge.
    pub fn full_width() -> Self {
        Self {
            inset: 0.0,
            row_shape: Shape::Rectangle,
            ..Self::classic()
        }
    }
}

impl ControlStyle for OutlineViewStyle {
    fn tengen() -> Self {
        Self {
            row_height: DISCLOSURE.row_height,
            indent: DISCLOSURE.indent,
            inset: DISCLOSURE.inset,
            lead_width: DISCLOSURE.lead,
            spacing: DISCLOSURE.label_gap,
            row_separator: Some(ShapeStyle::Role(Role::Separator)),
            separator_margin: DISCLOSURE.separator_margin,
            icon_size: DISCLOSURE.icon,
            chevron_size: DISCLOSURE.chevron,
            triangle_size: (DISCLOSURE.list_chevron, DISCLOSURE.list_chevron_stroke),
            chevron_offset: (DISCLOSURE.list_chevron_x, 0.0),
            open_chevron_offset: (
                DISCLOSURE.list_chevron_open_x,
                DISCLOSURE.list_chevron_open_y,
            ),
            column_width: DISCLOSURE.column_width,
            column_padding_x: SPACE_1,
            row_shape: Shape::RoundedRectangle {
                radius: DISCLOSURE.row_radius,
                style: CornerStyle::Continuous,
            },
            row_background: StateValue::new(ShapeStyle::Clear),
            selected_row_background: StateValue::new(ShapeStyle::Role(
                Role::SelectedContentInactive,
            ))
            .focused(ShapeStyle::Role(Role::SelectedContent)),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            selected_label: StateValue::new(ShapeStyle::Role(Role::Label))
                .focused(ShapeStyle::Role(Role::OnSelectedContent)),
            icon: StateValue::new(ShapeStyle::Accent),
            selected_icon: StateValue::new(ShapeStyle::Accent)
                .focused(ShapeStyle::Role(Role::OnSelectedContent)),
            disclosure: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            column_chevron: StateValue::new(ShapeStyle::Role(Role::TertiaryLabel)),
            selected_chevron: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel))
                .focused(ShapeStyle::Role(Role::OnSelectedContent)),
            column_separator: (ShapeStyle::Role(Role::Separator), HAIRLINE),
            effect: system_effect,
            font: BODY,
        }
    }
}

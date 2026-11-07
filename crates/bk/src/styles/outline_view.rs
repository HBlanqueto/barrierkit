use ctdesign::{BODY, DISCLOSURE, Font, HAIRLINE, SPACE_1};

use super::button::rounded;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct OutlineViewStyle {
    /// Tengen, with `indent`, `spacing`, `icon_size`, `chevron_size`: `DISCLOSURE.row_height`;

    /// `DISCLOSURE.indent` per level and of the chevron column; `SPACE_1` between the parts of

    /// a row; `DISCLOSURE.icon`; `DISCLOSURE.chevron`.
    pub row_height: f32,
    /// Tengen, with `row_height`, `spacing`, `icon_size`, `chevron_size`:
    /// `DISCLOSURE.row_height`; `DISCLOSURE.indent` per level and of the chevron column;
    /// `SPACE_1` between the parts of a row; `DISCLOSURE.icon`; `DISCLOSURE.chevron`.
    pub indent: f32,
    /// Tengen, with `row_height`, `indent`, `icon_size`, `chevron_size`:
    /// `DISCLOSURE.row_height`; `DISCLOSURE.indent` per level and of the chevron column;
    /// `SPACE_1` between the parts of a row; `DISCLOSURE.icon`; `DISCLOSURE.chevron`.
    pub spacing: f32,
    /// Tengen, with `row_height`, `indent`, `spacing`, `chevron_size`: `DISCLOSURE.row_height`;
    /// `DISCLOSURE.indent` per level and of the chevron column; `SPACE_1` between the parts of
    /// a row; `DISCLOSURE.icon`; `DISCLOSURE.chevron`.
    pub icon_size: f32,
    /// Tengen, with `row_height`, `indent`, `spacing`, `icon_size`: `DISCLOSURE.row_height`;
    /// `DISCLOSURE.indent` per level and of the chevron column; `SPACE_1` between the parts of
    /// a row; `DISCLOSURE.icon`; `DISCLOSURE.chevron`.
    pub chevron_size: f32,
    /// Tengen, with `column_padding_x`: `DISCLOSURE.column_width`; `SPACE_1` on each side of a
    /// row. `Browser` only.
    pub column_width: f32,
    /// Tengen, with `column_width`: `DISCLOSURE.column_width`; `SPACE_1` on each side of a row.
    /// `Browser` only.
    pub column_padding_x: f32,
    /// Tengen: `RoundedRectangle { radius: DISCLOSURE.row_radius, style: Circular }`.
    pub row_shape: Shape,
    /// Tengen: `Clear`.
    pub row_background: StateValue<ShapeStyle>,
    /// Tengen: `Accent`.
    pub selected_row_background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Role(OnAccent)`.
    pub selected_label: StateValue<ShapeStyle>,
    /// Tengen: `Accent`.
    pub icon: StateValue<ShapeStyle>,
    /// Tengen: `Role(OnAccent)`.
    pub selected_icon: StateValue<ShapeStyle>,
    /// Tengen, with `column_chevron`, `selected_chevron`: `Role(SecondaryLabel)`, the
    /// disclosure chevron of `OutlineView`; `Role(TertiaryLabel)`, the chevron of a `Browser`
    /// row with children; `Role(OnAccent)` on a selected row.
    pub disclosure: StateValue<ShapeStyle>,
    /// Tengen, with `disclosure`, `selected_chevron`: `Role(SecondaryLabel)`, the disclosure
    /// chevron of `OutlineView`; `Role(TertiaryLabel)`, the chevron of a `Browser` row with
    /// children; `Role(OnAccent)` on a selected row.
    pub column_chevron: StateValue<ShapeStyle>,
    /// Tengen, with `disclosure`, `column_chevron`: `Role(SecondaryLabel)`, the disclosure
    /// chevron of `OutlineView`; `Role(TertiaryLabel)`, the chevron of a `Browser` row with
    /// children; `Role(OnAccent)` on a selected row.
    pub selected_chevron: StateValue<ShapeStyle>,
    /// Tengen: `(Role(Separator), HAIRLINE)`, between `Browser` columns.
    pub column_separator: (ShapeStyle, f32),
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `BODY`.
    pub font: Font,
}

impl ControlStyle for OutlineViewStyle {
    fn tengen() -> Self {
        Self {
            row_height: DISCLOSURE.row_height,
            indent: DISCLOSURE.indent,
            spacing: SPACE_1,
            icon_size: DISCLOSURE.icon,
            chevron_size: DISCLOSURE.chevron,
            column_width: DISCLOSURE.column_width,
            column_padding_x: SPACE_1,
            row_shape: rounded(DISCLOSURE.row_radius),
            row_background: StateValue::new(ShapeStyle::Clear),
            selected_row_background: StateValue::new(ShapeStyle::Accent),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            selected_label: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            icon: StateValue::new(ShapeStyle::Accent),
            selected_icon: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            disclosure: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            column_chevron: StateValue::new(ShapeStyle::Role(Role::TertiaryLabel)),
            selected_chevron: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            column_separator: (ShapeStyle::Role(Role::Separator), HAIRLINE),
            effect: system_effect,
            font: BODY,
        }
    }
}

use ctdesign::{
    BODY, DISCLOSURE, DISCLOSURE_TRIANGLE_CONTRAST_DARK, DISCLOSURE_TRIANGLE_CONTRAST_DARK_TINT,
    DISCLOSURE_TRIANGLE_CONTRAST_LIGHT, DISCLOSURE_TRIANGLE_DARK, DISCLOSURE_TRIANGLE_DARK_TINT,
    DISCLOSURE_TRIANGLE_LIGHT, Font,
};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, per_look, system_effect};

pub(super) fn disclosure_triangle() -> StateValue<ShapeStyle> {
    StateValue::new(ShapeStyle::Color(per_look(
        [
            DISCLOSURE_TRIANGLE_LIGHT,
            DISCLOSURE_TRIANGLE_DARK,
            DISCLOSURE_TRIANGLE_DARK_TINT,
        ],
        [
            DISCLOSURE_TRIANGLE_CONTRAST_LIGHT,
            DISCLOSURE_TRIANGLE_CONTRAST_DARK,
            DISCLOSURE_TRIANGLE_CONTRAST_DARK_TINT,
        ],
    )))
}

#[derive(Clone, Debug)]
pub struct DisclosureGroupStyle {
    /// Tengen: `DISCLOSURE.row_height`.
    pub row_height: f32,
    /// Tengen: `0`, the indent of the children.
    pub indent: f32,
    /// Tengen: `DISCLOSURE.group_lead`, the column of the chevron.
    pub lead_width: f32,
    /// Tengen: `(DISCLOSURE.group_chevron, DISCLOSURE.group_chevron_stroke)`.
    pub chevron_size: (f32, f32),
    /// Tengen: `(DISCLOSURE.group_chevron_x, 0)`, where the closed chevron sits from the center of its column.
    pub chevron_offset: (f32, f32),
    /// Tengen: `(DISCLOSURE.group_chevron_open_x, DISCLOSURE.group_chevron_open_y)`, the same for the open chevron.
    pub open_chevron_offset: (f32, f32),
    /// Tengen: `Color(DISCLOSURE_TRIANGLE)` (`DISCLOSURE_TRIANGLE_CONTRAST` under Increase
    /// Contrast).
    pub chevron: StateValue<ShapeStyle>,
    /// Tengen: `label` `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `BODY`.
    pub font: Font,
}

impl ControlStyle for DisclosureGroupStyle {
    fn tengen() -> Self {
        Self {
            row_height: DISCLOSURE.row_height,
            indent: 0.0,
            lead_width: DISCLOSURE.group_lead,
            chevron_size: (DISCLOSURE.group_chevron, DISCLOSURE.group_chevron_stroke),
            chevron_offset: (DISCLOSURE.group_chevron_x, 0.0),
            open_chevron_offset: (
                DISCLOSURE.group_chevron_open_x,
                DISCLOSURE.group_chevron_open_y,
            ),
            chevron: disclosure_triangle(),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            effect: system_effect,
            font: BODY,
        }
    }
}

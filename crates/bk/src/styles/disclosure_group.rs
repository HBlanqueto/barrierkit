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
    /// Tengen: `DISCLOSURE.indent` of the chevron column and the children.
    pub indent: f32,
    /// Tengen: `(DISCLOSURE.triangle, DISCLOSURE.triangle_stroke)`.
    pub chevron_size: (f32, f32),
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
            indent: DISCLOSURE.indent,
            chevron_size: (DISCLOSURE.triangle, DISCLOSURE.triangle_stroke),
            chevron: disclosure_triangle(),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            effect: system_effect,
            font: BODY,
        }
    }
}

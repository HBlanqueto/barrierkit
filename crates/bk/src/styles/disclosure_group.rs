use ctdesign::{BODY, DISCLOSURE, Font};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct DisclosureGroupStyle {
    /// Tengen: `DISCLOSURE.row_height`.
    pub row_height: f32,
    /// Tengen: `DISCLOSURE.indent` of the chevron column and the children.
    pub indent: f32,
    /// Tengen: `DISCLOSURE.chevron`.
    pub chevron_size: f32,
    /// Tengen: `chevron` `Role(SecondaryLabel)`.
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
            chevron_size: DISCLOSURE.chevron,
            chevron: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            effect: system_effect,
            font: BODY,
        }
    }
}

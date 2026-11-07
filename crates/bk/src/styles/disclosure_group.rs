use ctappearance::Resolved;
use ctdesign::{BODY, DISCLOSURE, Font};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct DisclosureGroupStyle {
    pub row_height: f32,
    pub indent: f32,
    pub chevron_size: f32,
    pub chevron: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub effect: ColorEffect,
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

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

use ctdesign::{BODY, DISCLOSURE, Font, SPACE_1, SPACE_2};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct RuleEditorStyle {
    pub spacing: f32,
    pub row_spacing: f32,
    pub indent: f32,
    pub label: StateValue<ShapeStyle>,
    pub effect: ColorEffect,
    pub font: Font,
}

impl ControlStyle for RuleEditorStyle {
    fn tengen() -> Self {
        Self {
            spacing: SPACE_2,
            row_spacing: SPACE_1,
            indent: DISCLOSURE.indent,
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            effect: system_effect,
            font: BODY,
        }
    }
}

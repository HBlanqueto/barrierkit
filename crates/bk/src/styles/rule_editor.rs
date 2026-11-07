use ctdesign::{BODY, DISCLOSURE, Font, SPACE_1, SPACE_2};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

/// The rows hold `PopUpButton`s, `TextField`s and `IconButton`s, which take
/// their own styles.
///
/// States: only `normal`.
#[derive(Clone, Debug)]
pub struct RuleEditorStyle {
    /// Tengen, with `row_spacing`: `SPACE_2` between the rows, `SPACE_1` inside a row.
    pub spacing: f32,
    /// Tengen, with `spacing`: `SPACE_2` between the rows, `SPACE_1` inside a row.
    pub row_spacing: f32,
    /// Tengen: `DISCLOSURE.indent` per level of a nested group.
    pub indent: f32,
    /// Tengen: `Role(Label)`, the text "of the following".
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `BODY`.
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

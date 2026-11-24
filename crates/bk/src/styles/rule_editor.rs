use ctdesign::{BODY, DISCLOSURE, Font, SPACE_1, SPACE_2};

use crate::styles::{ControlStyle, Role, ShapeStyle, StateValue};

/// The rows hold `PopUpButton`s, `TextField`s and `IconButton`s, which take
/// their own styles.
///
/// States: only `normal`.
#[derive(Clone, Debug)]
pub struct RuleEditorStyle {
    pub spacing: f32,
    pub row_spacing: f32,
    /// Tengen: `DISCLOSURE.indent` per level of a nested group.
    pub indent: f32,
    /// Tengen: `Role(Label)`, the text "of the following".
    pub label: StateValue<ShapeStyle>,
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
            font: BODY,
        }
    }
}

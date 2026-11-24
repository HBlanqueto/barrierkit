use crate::styles::{ControlStyle, Role, ShapeStyle, StateValue};

/// States: focused while the view edits. The `ink` of `TextView::new` and
/// `.cursor(color)` are instance overrides of `text` and `cursor`, applied
/// before `.style`; the font stays an argument of `new`, because it can be any
/// family. The selection keeps Freya's highlight.
#[derive(Clone, Debug)]
pub struct TextViewStyle {
    pub text: StateValue<ShapeStyle>,
    pub cursor: StateValue<ShapeStyle>,
}

impl ControlStyle for TextViewStyle {
    fn tengen() -> Self {
        Self {
            text: StateValue::new(ShapeStyle::Role(Role::Label)),
            cursor: StateValue::new(ShapeStyle::Role(Role::Label)),
        }
    }
}

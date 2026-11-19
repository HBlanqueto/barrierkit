use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

/// States: focused while the view edits. The `ink` of `TextView::new` and
/// `.cursor(color)` are instance overrides of `text` and `cursor`, applied
/// before `.style`; the font stays an argument of `new`, because it can be any
/// family. The selection keeps Freya's highlight.
#[derive(Clone, Debug)]
pub struct TextViewStyle {
    /// Tengen, with `cursor`: `Role(Label)`.
    pub text: StateValue<ShapeStyle>,
    /// Tengen, with `text`: `Role(Label)`.
    pub cursor: StateValue<ShapeStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
}

impl ControlStyle for TextViewStyle {
    fn tengen() -> Self {
        Self {
            text: StateValue::new(ShapeStyle::Role(Role::Label)),
            cursor: StateValue::new(ShapeStyle::Role(Role::Label)),
            effect: system_effect,
        }
    }
}

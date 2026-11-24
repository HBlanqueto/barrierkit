use ctdesign::{
    CONTENT_UNAVAILABLE_ACTIONS_GAP, CONTENT_UNAVAILABLE_DESCRIPTION,
    CONTENT_UNAVAILABLE_DESCRIPTION_GAP, CONTENT_UNAVAILABLE_ICON, CONTENT_UNAVAILABLE_ICON_GAP,
    CONTENT_UNAVAILABLE_ICON_LINE, CONTENT_UNAVAILABLE_TITLE, Font,
};

use crate::styles::{ControlStyle, Role, ShapeStyle, StateValue};

#[derive(Clone, Debug)]
pub struct ContentUnavailableViewStyle {
    /// Tengen: `CONTENT_UNAVAILABLE_ICON`.
    pub symbol_size: f32,
    /// Tengen: `CONTENT_UNAVAILABLE_ICON_LINE`.
    pub symbol_line: f32,
    /// Tengen: `CONTENT_UNAVAILABLE_ICON_GAP`.
    pub symbol_gap: f32,
    /// Tengen: `Role(TertiaryLabel)`.
    pub symbol: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`.
    pub title: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`.
    pub description: StateValue<ShapeStyle>,
    /// Tengen: `CONTENT_UNAVAILABLE_DESCRIPTION_GAP`.
    pub description_gap: f32,
    /// Tengen: `CONTENT_UNAVAILABLE_ACTIONS_GAP`.
    pub actions_gap: f32,
    /// Tengen: `CONTENT_UNAVAILABLE_TITLE`.
    pub title_font: Font,
    /// Tengen: `CONTENT_UNAVAILABLE_DESCRIPTION`.
    pub description_font: Font,
}

impl ControlStyle for ContentUnavailableViewStyle {
    fn tengen() -> Self {
        Self {
            symbol_size: CONTENT_UNAVAILABLE_ICON,
            symbol_line: CONTENT_UNAVAILABLE_ICON_LINE,
            symbol_gap: CONTENT_UNAVAILABLE_ICON_GAP,
            symbol: StateValue::new(ShapeStyle::Role(Role::TertiaryLabel)),
            title: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            description: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            description_gap: CONTENT_UNAVAILABLE_DESCRIPTION_GAP,
            actions_gap: CONTENT_UNAVAILABLE_ACTIONS_GAP,
            title_font: CONTENT_UNAVAILABLE_TITLE,
            description_font: CONTENT_UNAVAILABLE_DESCRIPTION,
        }
    }
}

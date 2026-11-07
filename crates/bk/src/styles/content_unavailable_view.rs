use ctdesign::{BODY, Font, SETTINGS};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct ContentUnavailableViewStyle {
    /// Tengen: `SETTINGS.empty_padding_y`.
    pub padding_y: f32,
    /// Tengen: `empty_gap`.
    pub spacing: f32,
    /// Tengen: `empty_icon`.
    pub symbol_size: f32,
    /// Tengen: `symbol` `Role(TertiaryLabel)`.
    pub symbol: StateValue<ShapeStyle>,
    /// Tengen: `title` `Role(SecondaryLabel)`.
    pub title: StateValue<ShapeStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `font` `BODY`.
    pub font: Font,
}

impl ControlStyle for ContentUnavailableViewStyle {
    fn tengen() -> Self {
        Self {
            padding_y: SETTINGS.empty_padding_y as f32,
            spacing: SETTINGS.empty_gap as f32,
            symbol_size: SETTINGS.empty_icon as f32,
            symbol: StateValue::new(ShapeStyle::Role(Role::TertiaryLabel)),
            title: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            effect: system_effect,
            font: BODY,
        }
    }
}

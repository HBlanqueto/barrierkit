use ctdesign::{BODY, Font, SETTINGS};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct ContentUnavailableViewStyle {
    /// Tengen, with `spacing`, `symbol_size`: `padding_y`, `spacing`, `symbol_size`:

    /// `SETTINGS.empty_padding_y`, `empty_gap`, `empty_icon`.
    pub padding_y: f32,
    /// Tengen, with `padding_y`, `symbol_size`: `padding_y`, `spacing`, `symbol_size`:
    /// `SETTINGS.empty_padding_y`, `empty_gap`, `empty_icon`.
    pub spacing: f32,
    /// Tengen, with `padding_y`, `spacing`: `padding_y`, `spacing`, `symbol_size`:
    /// `SETTINGS.empty_padding_y`, `empty_gap`, `empty_icon`.
    pub symbol_size: f32,
    /// Tengen, with `title`: `symbol` `Role(TertiaryLabel)`; `title` `Role(SecondaryLabel)`.
    pub symbol: StateValue<ShapeStyle>,
    /// Tengen, with `symbol`: `symbol` `Role(TertiaryLabel)`; `title` `Role(SecondaryLabel)`.
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

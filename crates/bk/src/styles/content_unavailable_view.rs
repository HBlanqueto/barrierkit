use ctappearance::Resolved;
use ctdesign::{BODY, Font, SETTINGS};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct ContentUnavailableViewStyle {
    pub padding_y: f32,
    pub spacing: f32,
    pub symbol_size: f32,
    pub symbol: StateValue<ShapeStyle>,
    pub title: StateValue<ShapeStyle>,
    pub effect: ColorEffect,
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

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

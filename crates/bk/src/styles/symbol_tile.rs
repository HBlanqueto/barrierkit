use ctappearance::Resolved;
use ctdesign::SIDEBAR;

use super::button::rounded;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct SymbolTileStyle {
    pub size: f32,
    pub symbol_size: f32,
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub symbol: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub effect: ColorEffect,
}

impl ControlStyle for SymbolTileStyle {
    fn tengen() -> Self {
        Self {
            size: SIDEBAR.app_icon as f32,
            symbol_size: SIDEBAR.item_icon as f32,
            shape: rounded(SIDEBAR.item_radius as f32),
            background: StateValue::new(ShapeStyle::Accent),
            symbol: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            border: None,
            effect: system_effect,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

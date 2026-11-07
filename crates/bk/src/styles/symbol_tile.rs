use ctdesign::SIDEBAR;

use super::button::rounded;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct SymbolTileStyle {
    /// Tengen, with `symbol_size`: `size`, `symbol_size`: `SIDEBAR.app_icon`,

    /// `SIDEBAR.item_icon`.
    pub size: f32,
    /// Tengen, with `size`: `size`, `symbol_size`: `SIDEBAR.app_icon`, `SIDEBAR.item_icon`.
    pub symbol_size: f32,
    /// Tengen: `RoundedRectangle { radius: SIDEBAR.item_radius, style: Circular }`.
    pub shape: Shape,
    /// Tengen, with `symbol`: `background` `Accent`, `symbol` `Role(OnAccent)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen, with `background`: `background` `Accent`, `symbol` `Role(OnAccent)`.
    pub symbol: StateValue<ShapeStyle>,
    /// Tengen: `border` `None`, drawn inside.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: `system_effect`.
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
}

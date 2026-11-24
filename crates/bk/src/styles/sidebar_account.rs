use ctdesign::{Font, SETTINGS};

use crate::styles::{ControlStyle, ShapeStyle, StateValue};

#[derive(Clone, Debug)]
pub struct SidebarAccountStyle {
    /// Tengen: `SETTINGS.account_height`.
    pub height: f32,
    /// Tengen: `account_margin_bottom`.
    pub margin_bottom: f32,
    /// Tengen: `account_padding_x`.
    pub padding_x: f32,
    /// Tengen: `account_gap`.
    pub spacing: f32,
    pub name: StateValue<ShapeStyle>,
    pub detail: StateValue<ShapeStyle>,
    /// Tengen: `SETTINGS.account_name_font`.
    pub name_font: Font,
    /// Tengen: `account_detail_font`.
    pub detail_font: Font,
}

impl ControlStyle for SidebarAccountStyle {
    fn tengen() -> Self {
        let palette = crate::sidebar::item_palette(true);
        Self {
            height: SETTINGS.account_height as f32,
            margin_bottom: SETTINGS.account_margin_bottom as f32,
            padding_x: SETTINGS.account_padding_x as f32,
            spacing: SETTINGS.account_gap as f32,
            name: StateValue::new(ShapeStyle::Color(palette.item_ink)),
            detail: StateValue::new(ShapeStyle::Color(palette.count_ink)),
            name_font: SETTINGS.account_name_font,
            detail_font: SETTINGS.account_detail_font,
        }
    }
}

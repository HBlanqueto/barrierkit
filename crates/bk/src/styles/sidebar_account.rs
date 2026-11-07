use ctappearance::Resolved;
use ctdesign::{Font, SETTINGS};

use super::{ColorEffect, ControlStyle, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct SidebarAccountStyle {
    pub height: f32,
    pub margin_bottom: f32,
    pub padding_x: f32,
    pub spacing: f32,
    pub name: StateValue<ShapeStyle>,
    pub detail: StateValue<ShapeStyle>,
    pub effect: ColorEffect,
    pub name_font: Font,
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
            effect: system_effect,
            name_font: SETTINGS.account_name_font,
            detail_font: SETTINGS.account_detail_font,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

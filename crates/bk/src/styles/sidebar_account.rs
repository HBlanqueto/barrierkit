use ctdesign::{Font, SETTINGS};

use super::{ColorEffect, ControlStyle, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct SidebarAccountStyle {
    /// Tengen, with `margin_bottom`, `padding_x`, `spacing`: `height`, `margin_bottom`,

    /// `padding_x`, `spacing`: `SETTINGS.account_height`, `account_margin_bottom`,

    /// `account_padding_x`, `account_gap`.
    pub height: f32,
    /// Tengen, with `height`, `padding_x`, `spacing`: `height`, `margin_bottom`, `padding_x`,
    /// `spacing`: `SETTINGS.account_height`, `account_margin_bottom`, `account_padding_x`,
    /// `account_gap`.
    pub margin_bottom: f32,
    /// Tengen, with `height`, `margin_bottom`, `spacing`: `height`, `margin_bottom`,
    /// `padding_x`, `spacing`: `SETTINGS.account_height`, `account_margin_bottom`,
    /// `account_padding_x`, `account_gap`.
    pub padding_x: f32,
    /// Tengen, with `height`, `margin_bottom`, `padding_x`: `height`, `margin_bottom`,
    /// `padding_x`, `spacing`: `SETTINGS.account_height`, `account_margin_bottom`,
    /// `account_padding_x`, `account_gap`.
    pub spacing: f32,
    /// Tengen, with `detail`: `name`, `detail`: `Color` of the item ink and of the count ink.
    pub name: StateValue<ShapeStyle>,
    /// Tengen, with `name`: `name`, `detail`: `Color` of the item ink and of the count ink.
    pub detail: StateValue<ShapeStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen, with `detail_font`: `name_font`, `detail_font`: `SETTINGS.account_name_font`,
    /// `account_detail_font`.
    pub name_font: Font,
    /// Tengen, with `name_font`: `name_font`, `detail_font`: `SETTINGS.account_name_font`,
    /// `account_detail_font`.
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
}

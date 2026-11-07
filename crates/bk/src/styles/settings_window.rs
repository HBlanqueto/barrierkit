use ctdesign::{BODY, CAPTION, Font, HAIRLINE, LIST_GROUP, SETTINGS, SIDEBAR, SPACE_2};

use super::button::rounded;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};
use crate::theme::{SurfaceKind, popover_palette};

#[derive(Clone, Debug)]
pub struct SettingsWindowStyle {
    pub search_gap: f32,
    pub toolbar_spacing: f32,
    pub navigation_spacing: f32,
    pub popover_gap: f32,
    pub popover_max_height: f32,
    pub popover_padding: f32,
    pub header_height: f32,
    pub result_height: f32,
    pub result_subtitle_height: f32,
    pub result_padding_x: f32,
    pub result_spacing: f32,
    pub popover_shape: Shape,
    pub result_shape: Shape,
    pub popover_background: StateValue<ShapeStyle>,
    pub header: StateValue<ShapeStyle>,
    pub result_background: StateValue<ShapeStyle>,
    pub result_title: StateValue<ShapeStyle>,
    pub result_subtitle: StateValue<ShapeStyle>,
    pub suggestion_background: StateValue<ShapeStyle>,
    pub suggestion_title: StateValue<ShapeStyle>,
    pub suggestion_subtitle: StateValue<ShapeStyle>,
    pub popover_border: Option<(ShapeStyle, f32)>,
    pub effect: ColorEffect,
    pub header_font: Font,
    pub title_font: Font,
    pub subtitle_font: Font,
}

impl ControlStyle for SettingsWindowStyle {
    fn tengen() -> Self {
        let resolved = crate::theme::resolved();
        let menu = popover_palette(&resolved, SurfaceKind::Toplevel);
        let on_accent = ShapeStyle::Role(Role::OnAccent);
        Self {
            search_gap: SETTINGS.sidebar_search_gap as f32,
            toolbar_spacing: SETTINGS.toolbar_gap as f32,
            navigation_spacing: SPACE_2,
            popover_gap: SETTINGS.search_popover_gap as f32,
            popover_max_height: SETTINGS.search_popover_max_height as f32,
            popover_padding: menu.padding,
            header_height: SETTINGS.search_header_height as f32,
            result_height: SIDEBAR.item_height as f32,
            result_subtitle_height: SETTINGS.search_result_height as f32,
            result_padding_x: menu.item_padding_x,
            result_spacing: SIDEBAR.item_gap as f32,
            popover_shape: rounded(LIST_GROUP.radius),
            result_shape: rounded(menu.item_radius),
            popover_background: StateValue::new(ShapeStyle::Color(menu.bg)),
            header: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            result_background: StateValue::new(ShapeStyle::Clear).hovered(ShapeStyle::Color(
                resolved.sidebar.palette.inactive_selected_bg,
            )),
            result_title: StateValue::new(ShapeStyle::Role(Role::Label)),
            result_subtitle: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            suggestion_background: StateValue::new(ShapeStyle::Clear).hovered(ShapeStyle::Accent),
            suggestion_title: StateValue::new(ShapeStyle::Role(Role::Label))
                .hovered(on_accent.clone()),
            suggestion_subtitle: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel))
                .hovered(on_accent),
            popover_border: Some((ShapeStyle::Color(menu.separator_color), HAIRLINE)),
            effect: system_effect,
            header_font: CAPTION,
            title_font: BODY,
            subtitle_font: CAPTION,
        }
    }
}

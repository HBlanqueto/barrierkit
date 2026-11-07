use ctdesign::{BODY, Font, SIDEBAR};

use super::button::rounded;
use super::{ColorEffect, ControlStyle, Shape, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct SidebarItemStyle {
    pub height: f32,
    pub padding_x: f32,
    pub spacing: f32,
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub selected_background: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub selected_label: StateValue<ShapeStyle>,
    pub selected_tile_border: Option<(ShapeStyle, f32)>,
    pub effect: ColorEffect,
    pub font: Font,
    pub selected_font: Font,
}

impl ControlStyle for SidebarItemStyle {
    fn tengen() -> Self {
        let palette = crate::sidebar::item_palette(true);
        let font = |weight: u16| Font {
            size: SIDEBAR.item_text_size,
            weight,
            ..BODY
        };
        Self {
            height: SIDEBAR.item_height as f32,
            padding_x: SIDEBAR.item_padding_x as f32,
            spacing: SIDEBAR.item_gap as f32,
            shape: rounded(SIDEBAR.item_radius as f32),
            background: StateValue::new(ShapeStyle::Clear)
                .hovered(ShapeStyle::Color(palette.hover_bg)),
            selected_background: StateValue::new(ShapeStyle::Color(palette.selected_bg)),
            label: StateValue::new(ShapeStyle::Color(palette.item_ink)),
            selected_label: StateValue::new(ShapeStyle::Color(palette.selected_ink)),
            selected_tile_border: Some((
                ShapeStyle::Color(palette.selected_icon_rim),
                SIDEBAR.rim_width,
            )),
            effect: system_effect,
            font: font(SIDEBAR.item_weight),
            selected_font: font(SIDEBAR.item_selected_weight),
        }
    }
}

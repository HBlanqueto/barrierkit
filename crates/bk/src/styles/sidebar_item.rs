use ctdesign::{BODY, Font, SIDEBAR};

use super::button::rounded;
use super::{ColorEffect, ControlStyle, Shape, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct SidebarItemStyle {
    /// Tengen, with `padding_x`, `spacing`: `height`, `padding_x`, `spacing`:

    /// `SIDEBAR.item_height`, `item_padding_x`, `item_gap`.
    pub height: f32,
    /// Tengen, with `height`, `spacing`: `height`, `padding_x`, `spacing`:
    /// `SIDEBAR.item_height`, `item_padding_x`, `item_gap`.
    pub padding_x: f32,
    /// Tengen, with `height`, `padding_x`: `height`, `padding_x`, `spacing`:
    /// `SIDEBAR.item_height`, `item_padding_x`, `item_gap`.
    pub spacing: f32,
    /// Tengen: `RoundedRectangle { radius: SIDEBAR.item_radius, style: Circular }`.
    pub shape: Shape,
    /// Tengen, with `selected_background`, `label`, `selected_label`: `background` `Clear`,
    /// hovered `Color` of the hover; `selected_background` `Color` of the selection; `label`,
    /// `selected_label` `Color` of the item and selected ink. In an inactive window the
    /// inactive colors.
    pub background: StateValue<ShapeStyle>,
    /// Tengen, with `background`, `label`, `selected_label`: `background` `Clear`, hovered
    /// `Color` of the hover; `selected_background` `Color` of the selection; `label`,
    /// `selected_label` `Color` of the item and selected ink. In an inactive window the
    /// inactive colors.
    pub selected_background: StateValue<ShapeStyle>,
    /// Tengen, with `background`, `selected_background`, `selected_label`: `background`
    /// `Clear`, hovered `Color` of the hover; `selected_background` `Color` of the selection;
    /// `label`, `selected_label` `Color` of the item and selected ink. In an inactive window
    /// the inactive colors.
    pub label: StateValue<ShapeStyle>,
    /// Tengen, with `background`, `selected_background`, `label`: `background` `Clear`, hovered
    /// `Color` of the hover; `selected_background` `Color` of the selection; `label`,
    /// `selected_label` `Color` of the item and selected ink. In an inactive window the
    /// inactive colors.
    pub selected_label: StateValue<ShapeStyle>,
    /// Tengen: `selected_tile_border` `Some((Color of the selected icon rim,
    /// SIDEBAR.rim_width))`, the tile of a selected item.
    pub selected_tile_border: Option<(ShapeStyle, f32)>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen, with `selected_font`: `font`, `selected_font`: `BODY` at
    /// `SIDEBAR.item_text_size` with `SIDEBAR.item_weight` and `SIDEBAR.item_selected_weight`;
    /// one line.
    pub font: Font,
    /// Tengen, with `font`: `font`, `selected_font`: `BODY` at `SIDEBAR.item_text_size` with
    /// `SIDEBAR.item_weight` and `SIDEBAR.item_selected_weight`; one line.
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

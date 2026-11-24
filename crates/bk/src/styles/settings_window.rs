use ctdesign::{BODY, CAPTION, Font, SETTINGS, SIDEBAR, SPACE_2};

use crate::styles::button::rounded;
use crate::styles::{
    ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect,
};
use crate::theme::{SurfaceKind, over_glass, popover_palette};

/// `SettingsWindow` is built from styled views: its sidebar takes
/// `SidebarStyle`, `SidebarItemStyle`, `SidebarSectionStyle` and
/// `SidebarAccountStyle`, the search field `SearchFieldStyle`, the bar
/// `TitlebarStyle`, the arrows `ToolGroupStyle` and `IconButtonStyle`, the
/// split `SplitViewStyle`, the scroll area `ScrollViewStyle` and each pane
/// `FormStyle`. `SettingsWindowStyle` holds the rest: the gaps between them and
/// the search popover. `Pane` and `PaneGroup` are data and take no style.
///
/// States: a result takes hovered while the pointer or the keyboard cursor is on
/// it, pressed and focused. Its tile takes `SymbolTileStyle`.
///
/// States: only `normal`. `NavigationButtons` draws nothing of its own: it is a
/// `ToolGroup` of two `IconButton`s. `LanguagePicker` is a `PopUpButton`.
#[derive(Clone, Debug)]
pub struct SettingsWindowStyle {
    /// Tengen: `SETTINGS.sidebar_search_gap` under the search field.
    pub search_gap: f32,
    /// Tengen: `SETTINGS.toolbar_gap`, the `spacing` of the bar.
    pub toolbar_spacing: f32,
    /// Tengen: `SPACE_2` between the sidebar toggle and the arrows.
    pub navigation_spacing: f32,
    pub popover_gap: f32,
    pub popover_max_height: f32,
    pub popover_padding: f32,
    /// Tengen: `SETTINGS.search_header_height`.
    pub header_height: f32,
    /// Tengen: `SIDEBAR.item_height`.
    pub result_height: f32,
    /// Tengen: `SETTINGS.search_result_height` for a result with a pane name under it.
    pub result_subtitle_height: f32,
    /// Tengen: `item_padding_x` of the popover menu palette, also for the headers.
    pub result_padding_x: f32,
    /// Tengen: `SIDEBAR.item_gap` between the tile and the text.
    pub result_spacing: f32,
    /// Tengen: `RoundedRectangle` of the palette's `item_radius`.
    pub result_shape: Shape,
    /// Tengen: `Color` of the popover menu background, clear over the compositor's glass.
    pub popover_background: StateValue<ShapeStyle>,
    /// Tengen: `Color` of the glass shortcut ink of the popover menu, "Suggestions", "Recents".
    pub header: StateValue<ShapeStyle>,
    /// Tengen: `Clear`, hovered `Color` of the inactive sidebar selection.
    pub result_background: StateValue<ShapeStyle>,
    /// Tengen: `Color` of the glass ink of the popover menu.
    pub result_title: StateValue<ShapeStyle>,
    /// Tengen: `Color` of the glass shortcut ink of the popover menu. While the user types.
    pub result_subtitle: StateValue<ShapeStyle>,
    /// Tengen: `Clear`, hovered `Accent`.
    pub suggestion_background: StateValue<ShapeStyle>,
    /// Tengen: `Color` of the glass ink of the popover menu, hovered `Role(OnAccent)`.
    pub suggestion_title: StateValue<ShapeStyle>,
    /// Tengen: `Color` of the glass shortcut ink of the popover menu, hovered `Role(OnAccent)`.
    /// Before the user types.
    pub suggestion_subtitle: StateValue<ShapeStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `CAPTION`.
    pub header_font: Font,
    /// Tengen: `BODY`.
    pub title_font: Font,
    /// Tengen: `CAPTION`.
    pub subtitle_font: Font,
}

impl ControlStyle for SettingsWindowStyle {
    fn tengen() -> Self {
        let resolved = crate::theme::resolved();
        let menu = over_glass(popover_palette(&resolved, SurfaceKind::Toplevel));
        let label = ShapeStyle::Color(menu.fg);
        let secondary_label = ShapeStyle::Color(menu.shortcut_color);
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
            result_shape: rounded(menu.item_radius),
            popover_background: StateValue::new(ShapeStyle::Color(menu.bg)),
            header: StateValue::new(secondary_label.clone()),
            result_background: StateValue::new(ShapeStyle::Clear).hovered(ShapeStyle::Color(
                resolved.sidebar.palette.inactive_selected_bg,
            )),
            result_title: StateValue::new(label.clone()),
            result_subtitle: StateValue::new(secondary_label.clone()),
            suggestion_background: StateValue::new(ShapeStyle::Clear).hovered(ShapeStyle::Accent),
            suggestion_title: StateValue::new(label).hovered(on_accent.clone()),
            suggestion_subtitle: StateValue::new(secondary_label).hovered(on_accent),
            effect: system_effect,
            header_font: CAPTION,
            title_font: BODY,
            subtitle_font: CAPTION,
        }
    }
}

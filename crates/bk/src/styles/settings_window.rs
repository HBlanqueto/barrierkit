use ctdesign::{BODY, CAPTION, Font, HAIRLINE, LIST_GROUP, SETTINGS, SIDEBAR, SPACE_2};

use super::button::rounded;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};
use crate::theme::{SurfaceKind, popover_palette};

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
    /// Tengen, with `popover_max_height`, `popover_padding`: `SETTINGS.search_popover_gap`
    /// under the field, `SETTINGS.search_popover_max_height`, the `padding` of the popover menu
    /// palette.
    pub popover_gap: f32,
    /// Tengen, with `popover_gap`, `popover_padding`: `SETTINGS.search_popover_gap` under the
    /// field, `SETTINGS.search_popover_max_height`, the `padding` of the popover menu palette.
    pub popover_max_height: f32,
    /// Tengen, with `popover_gap`, `popover_max_height`: `SETTINGS.search_popover_gap` under
    /// the field, `SETTINGS.search_popover_max_height`, the `padding` of the popover menu
    /// palette.
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
    /// Tengen: `RoundedRectangle { radius: LIST_GROUP.radius, style: Circular }`, a `Capsule`
    /// or `Circle` stays square.
    pub popover_shape: Shape,
    /// Tengen: `RoundedRectangle` of the palette's `item_radius`.
    pub result_shape: Shape,
    /// Tengen: `Color` of the popover menu background.
    pub popover_background: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`, "Suggestions", "Recents".
    pub header: StateValue<ShapeStyle>,
    /// Tengen: `Clear`, hovered `Color` of the inactive sidebar selection.
    pub result_background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub result_title: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`. While the user types.
    pub result_subtitle: StateValue<ShapeStyle>,
    /// Tengen: `Clear`, hovered `Accent`.
    pub suggestion_background: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`, hovered `Role(OnAccent)`.
    pub suggestion_title: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`, hovered `Role(OnAccent)`. Before the user types.
    pub suggestion_subtitle: StateValue<ShapeStyle>,
    /// Tengen: `Some((Color of the popover separator, HAIRLINE))`, drawn outside.
    pub popover_border: Option<(ShapeStyle, f32)>,
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

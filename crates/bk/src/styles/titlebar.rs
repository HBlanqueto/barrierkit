use ctappearance::palette;
use ctdesign::{
    FocusPair, Font, HAIRLINE, SETTINGS, TITLEBAR, TITLEBAR_CLIENT_HEIGHT, TITLEBAR_PADDING_X,
    TITLEBAR_SYSTEM_VEIL, TOOLBAR_TITLE,
};
use torin::prelude::Alignment;

use crate::styles::{ControlStyle, Role, ShapeStyle, StateValue};

/// `Titlebar` has two bars: the centered one (no sidebar) and the one beside a
/// sidebar (`.with_sidebar(true)`). Its paints are `focused` while the window is
/// active.
///
/// States: focused (the window is active). In full screen the bar sits on the
/// full-screen glass and draws no background. `.spacing(px)` is an instance
/// override of `spacing` and `sidebar_spacing`; `.hairline(true)` sets `hairline` to `TitlebarStyle::line()`.
///
/// `FullscreenBarStyle` paints the glass under the overlaid bar and the rail in
/// full screen. The kind of bar stays `WindowOptions::fullscreen_bar`. It has no
/// instance level: set it in the theme or with `.control_style` around the
/// window.
///
/// States: only `normal`. When the user reduces transparency every bar is the
/// solid `tint`.
#[derive(Clone, Debug)]
pub struct TitlebarStyle {
    /// Tengen: `TITLEBAR_CLIENT_HEIGHT`.
    pub height: f32,
    /// Tengen: `TITLEBAR.thickness` for `.close_only(true)`, utility and HUD windows.
    pub close_only_height: f32,
    /// Tengen: `SETTINGS.toolbar_height`, the unified bar of macOS 27. `sidebar_bar_height()`
    /// returns it.
    pub sidebar_height: f32,
    /// Tengen: `TITLEBAR_PADDING_X`.
    pub padding_x: f32,
    /// Tengen: `SETTINGS.toolbar_padding_x`. With the traffic lights in the centered bar the side padding is `ctappearance::traffic::inset`
    /// of `height`.
    pub sidebar_padding_x: f32,
    /// Tengen: `0.0`, between the bar's items.
    pub spacing: f32,
    /// Tengen: `SETTINGS.toolbar_lights_gap`, between the traffic lights and the title when the
    /// bar has no sidebar and no left zone.
    pub lights_spacing: f32,
    /// Tengen: `SETTINGS.toolbar_gap`, between the left zone and the title of the bar beside a
    /// sidebar: the title sits 20 from the sidebar's edge.
    pub sidebar_spacing: f32,
    /// Tengen: `WindowBackground`, focused or not, like a unified AppKit toolbar; `Clear` over
    /// a `Backdrop`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Color` of the toolbar surface while the sidebar shows; over the platform's window
    /// material, `TITLEBAR_SYSTEM_VEIL`, as AppKit's title bar lifts it; `Clear` over a
    /// `Backdrop`.
    pub sidebar_background: StateValue<ShapeStyle>,
    /// Tengen: `Clear`.
    pub close_only_background: StateValue<ShapeStyle>,
    /// Tengen: `Color` of the unfocused title, focused `Color` of the focused one; over a
    /// `Backdrop` `Role(SecondaryLabel)`, focused `Role(Label)`.
    pub title: StateValue<ShapeStyle>,
    /// Tengen: `Color` of the unfocused and focused `row_title`, dimmer than `title`, for the
    /// title row of `WindowToolbarStyle::Expanded`; over a `Backdrop` `Role(TertiaryLabel)`,
    /// focused `Role(SecondaryLabel)`.
    pub row_title: StateValue<ShapeStyle>,
    /// Tengen: `None`, as macOS draws no separator under a unified toolbar at rest over a scroll
    /// view; over content that does not scroll, AppKit's own title bar draws it on macOS.
    /// `NSTitlebarSeparatorStyle.line` is `Some(TitlebarStyle::line())`.
    pub hairline: Option<(ShapeStyle, f32)>,
    /// Tengen: `Center` between the two equal zones.
    pub title_alignment: Alignment,
    /// Tengen: `Start`, next to the left zone.
    pub sidebar_title_alignment: Alignment,
    /// Tengen: `TITLEBAR.font`, `TOOLBAR_TITLE`.
    pub title_font: Font,
    /// Tengen: one line, so `line_height` has no effect.
    pub sidebar_title_font: Font,
}

impl TitlebarStyle {
    pub fn line() -> (ShapeStyle, f32) {
        let color = if crate::backdrop::tinted().is_some() {
            crate::sidebar::sidebar_palette().line
        } else {
            crate::theme::resolved().surfaces.toolbar_line
        };
        (ShapeStyle::Color(color), HAIRLINE)
    }
}

impl ControlStyle for TitlebarStyle {
    fn tengen() -> Self {
        let resolved = crate::theme::resolved();
        let tinted = crate::backdrop::tinted().is_some();
        let surface = |color| {
            if tinted {
                ShapeStyle::Clear
            } else {
                ShapeStyle::Color(color)
            }
        };
        let active = if tinted {
            ShapeStyle::Clear
        } else {
            ShapeStyle::WindowBackground
        };
        let ink = palette(resolved.scheme);
        let (title, row_title) = if tinted {
            (
                StateValue::new(ShapeStyle::Role(Role::SecondaryLabel))
                    .focused(ShapeStyle::Role(Role::Label)),
                StateValue::new(ShapeStyle::Role(Role::TertiaryLabel))
                    .focused(ShapeStyle::Role(Role::SecondaryLabel)),
            )
        } else {
            let pair = |ink: FocusPair| {
                StateValue::new(ShapeStyle::Color(ink.unfocused))
                    .focused(ShapeStyle::Color(ink.focused))
            };
            (pair(ink.title), pair(ink.row_title))
        };
        Self {
            height: TITLEBAR_CLIENT_HEIGHT as f32,
            close_only_height: TITLEBAR.thickness as f32,
            sidebar_height: SETTINGS.toolbar_height as f32,
            padding_x: TITLEBAR_PADDING_X,
            sidebar_padding_x: SETTINGS.toolbar_padding_x as f32,
            spacing: 0.0,
            lights_spacing: SETTINGS.toolbar_lights_gap as f32,
            sidebar_spacing: SETTINGS.toolbar_gap as f32,
            background: StateValue::new(active),
            sidebar_background: StateValue::new(if crate::theme::window_material_here() {
                surface(TITLEBAR_SYSTEM_VEIL)
            } else {
                surface(resolved.surfaces.toolbar)
            }),
            close_only_background: StateValue::new(ShapeStyle::Clear),
            title,
            row_title,
            hairline: None,
            title_alignment: Alignment::Center,
            sidebar_title_alignment: Alignment::Start,
            title_font: TITLEBAR.font,
            sidebar_title_font: TOOLBAR_TITLE,
        }
    }
}

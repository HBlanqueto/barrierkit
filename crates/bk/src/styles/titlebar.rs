use ctappearance::{SidebarStyle as SidebarKind, palette};
use ctdesign::{
    Font, HAIRLINE, SETTINGS, TITLE_2, TITLEBAR, TITLEBAR_CLIENT_HEIGHT, TITLEBAR_PADDING_X,
};
use torin::prelude::Alignment;

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};
use crate::theme::{SurfaceKind, surface_background};

/// `Titlebar` has two bars: the centered one (no sidebar) and the one beside a
/// sidebar (`.with_sidebar(true)`). Its paints are `focused` while the window is
/// active.
///
/// States: focused (the window is active). In full screen the bar sits on the
/// full-screen glass and draws no background. `.spacing(px)` is an instance
/// override of `spacing`; `.hairline(false)` sets `hairline` to `None`.
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
    /// Tengen, with `close_only_height`: `TITLEBAR_CLIENT_HEIGHT`; `TITLEBAR.thickness` for

    /// `.close_only(true)`, utility and HUD windows.
    pub height: f32,
    /// Tengen, with `height`: `TITLEBAR_CLIENT_HEIGHT`; `TITLEBAR.thickness` for
    /// `.close_only(true)`, utility and HUD windows.
    pub close_only_height: f32,
    /// Tengen, with `sidebar_padding_top`: `SETTINGS.toolbar_height`,
    /// `SETTINGS.toolbar_padding_top` with an island sidebar; `TITLEBAR_CLIENT_HEIGHT` and
    /// `0.0` with a flat one. `sidebar_bar_metrics()` returns them.
    pub sidebar_height: f32,
    /// Tengen, with `sidebar_height`: `SETTINGS.toolbar_height`, `SETTINGS.toolbar_padding_top`
    /// with an island sidebar; `TITLEBAR_CLIENT_HEIGHT` and `0.0` with a flat one.
    /// `sidebar_bar_metrics()` returns them.
    pub sidebar_padding_top: f32,
    /// Tengen, with `sidebar_padding_x`: `TITLEBAR_PADDING_X`; `SETTINGS.toolbar_padding_x`
    /// (island) or `TITLEBAR_PADDING_X` (flat). With the traffic lights in the centered bar the
    /// side padding is half of `height` minus the lights' `diameter`.
    pub padding_x: f32,
    /// Tengen, with `padding_x`: `TITLEBAR_PADDING_X`; `SETTINGS.toolbar_padding_x` (island) or
    /// `TITLEBAR_PADDING_X` (flat). With the traffic lights in the centered bar the side
    /// padding is half of `height` minus the lights' `diameter`.
    pub sidebar_padding_x: f32,
    /// Tengen: `0.0`, between the bar's items.
    pub spacing: f32,
    /// Tengen: `Color` of `titlebar_inactive`, focused `Color` of the window surface; `Clear`
    /// over a `Backdrop`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Color` of the window surface while the sidebar shows; `Clear` over a
    /// `Backdrop`.
    pub sidebar_background: StateValue<ShapeStyle>,
    /// Tengen: `Clear`.
    pub close_only_background: StateValue<ShapeStyle>,
    /// Tengen: `Color` of the unfocused title, focused `Color` of the focused one; over a
    /// `Backdrop` `Role(SecondaryLabel)`, focused `Role(Label)`.
    pub title: StateValue<ShapeStyle>,
    /// Tengen: `Some((Color of the sidebar line, HAIRLINE))`, at the bottom of the bar beside a
    /// sidebar.
    pub hairline: Option<(ShapeStyle, f32)>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen, with `sidebar_title_alignment`: `Center` between the two equal zones; `Start`,
    /// next to the left zone.
    pub title_alignment: Alignment,
    /// Tengen, with `title_alignment`: `Center` between the two equal zones; `Start`, next to
    /// the left zone.
    pub sidebar_title_alignment: Alignment,
    /// Tengen, with `sidebar_title_font`: `TITLEBAR.font`, `TITLE_2`; one line, so
    /// `line_height` has no effect.
    pub title_font: Font,
    /// Tengen, with `title_font`: `TITLEBAR.font`, `TITLE_2`; one line, so `line_height` has no
    /// effect.
    pub sidebar_title_font: Font,
}

impl ControlStyle for TitlebarStyle {
    fn tengen() -> Self {
        let resolved = crate::theme::resolved();
        let island = resolved.sidebar.style == SidebarKind::Island;
        let tinted = crate::backdrop::tinted().is_some();
        let surface = |color| {
            if tinted {
                ShapeStyle::Clear
            } else {
                ShapeStyle::Color(color)
            }
        };
        let active = surface(surface_background(&resolved, SurfaceKind::Toplevel));
        let title = if tinted {
            StateValue::new(ShapeStyle::Role(Role::SecondaryLabel))
                .focused(ShapeStyle::Role(Role::Label))
        } else {
            let ink = palette(resolved.scheme).title;
            StateValue::new(ShapeStyle::Color(ink.unfocused))
                .focused(ShapeStyle::Color(ink.focused))
        };
        let (sidebar_height, sidebar_padding_top, sidebar_padding_x) = if island {
            (
                SETTINGS.toolbar_height as f32,
                SETTINGS.toolbar_padding_top as f32,
                SETTINGS.toolbar_padding_x as f32,
            )
        } else {
            (TITLEBAR_CLIENT_HEIGHT as f32, 0.0, TITLEBAR_PADDING_X)
        };
        Self {
            height: TITLEBAR_CLIENT_HEIGHT as f32,
            close_only_height: TITLEBAR.thickness as f32,
            sidebar_height,
            sidebar_padding_top,
            padding_x: TITLEBAR_PADDING_X,
            sidebar_padding_x,
            spacing: 0.0,
            background: StateValue::new(surface(resolved.surfaces.titlebar_inactive))
                .focused(active.clone()),
            sidebar_background: StateValue::new(active),
            close_only_background: StateValue::new(ShapeStyle::Clear),
            title,
            hairline: Some((
                ShapeStyle::Color(crate::sidebar::sidebar_palette().line),
                HAIRLINE,
            )),
            effect: system_effect,
            title_alignment: Alignment::Center,
            sidebar_title_alignment: Alignment::Start,
            title_font: TITLEBAR.font,
            sidebar_title_font: TITLE_2,
        }
    }
}

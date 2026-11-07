use ctappearance::{SidebarStyle as SidebarKind, palette};
use ctdesign::{
    Font, HAIRLINE, SETTINGS, TITLE_2, TITLEBAR, TITLEBAR_CLIENT_HEIGHT, TITLEBAR_PADDING_X,
};
use torin::prelude::Alignment;

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};
use crate::theme::{SurfaceKind, surface_background};

#[derive(Clone, Debug)]
pub struct TitlebarStyle {
    pub height: f32,
    pub close_only_height: f32,
    pub sidebar_height: f32,
    pub sidebar_padding_top: f32,
    pub padding_x: f32,
    pub sidebar_padding_x: f32,
    pub spacing: f32,
    pub background: StateValue<ShapeStyle>,
    pub sidebar_background: StateValue<ShapeStyle>,
    pub close_only_background: StateValue<ShapeStyle>,
    pub title: StateValue<ShapeStyle>,
    pub hairline: Option<(ShapeStyle, f32)>,
    pub effect: ColorEffect,
    pub title_alignment: Alignment,
    pub sidebar_title_alignment: Alignment,
    pub title_font: Font,
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

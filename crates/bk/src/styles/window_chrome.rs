use ctappearance::{ColorScheme, Resolved, SidebarStyle as SidebarKind, palette};
use ctdesign::{DURATION_BASE, Font, HAIRLINE, SHADOW, SHADOW_DARK, TITLEBAR, TITLEBAR_RAIL};

use super::button::rounded;
use super::{
    Animation, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    system_effect,
};
use crate::chrome::{RailStyle, RestingEdge, Reveal, TitlebarLayout};
use crate::theme::{SurfaceKind, surface_background};

#[derive(Clone, Debug)]
pub struct WindowChromeStyle {
    pub layout: TitlebarLayout,
    pub rail_style: RailStyle,
    pub reveal: Reveal,
    pub resting_edge: RestingEdge,
    pub round_lights: bool,
    pub rail_width: f32,
    pub rail_padding_top: f32,
    pub rail_padding_bottom: f32,
    pub rail_spacing: f32,
    pub pill_inset: f32,
    pub pill_padding: (f32, f32),
    pub pill_shape: Shape,
    pub rail_background: StateValue<ShapeStyle>,
    pub drawer_background: StateValue<ShapeStyle>,
    pub title: StateValue<ShapeStyle>,
    pub pill_background: StateValue<ShapeStyle>,
    pub rail_line: Option<(ShapeStyle, f32)>,
    pub line_edge: (ShapeStyle, f32),
    pub accent_edge: (ShapeStyle, f32),
    pub pill_border: Option<(ShapeStyle, f32)>,
    pub shadows: Vec<ShadowStyle>,
    pub effect: ColorEffect,
    pub title_angle: f32,
    pub title_font: Font,
    pub animation: Animation,
}

impl ControlStyle for WindowChromeStyle {
    fn tengen() -> Self {
        let resolved = crate::theme::resolved();
        let island = resolved.sidebar.style == SidebarKind::Island;
        let toplevel = ShapeStyle::Color(surface_background(&resolved, SurfaceKind::Toplevel));
        let ink = palette(resolved.scheme).title;
        let line = ShapeStyle::Color(resolved.sidebar.palette.line);
        let contact = if resolved.scheme == ColorScheme::Light {
            SHADOW.contact
        } else {
            SHADOW_DARK.contact
        };
        Self {
            layout: TitlebarLayout::Top,
            rail_style: RailStyle::Normal,
            reveal: Reveal::Always,
            resting_edge: RestingEdge::None,
            round_lights: false,
            rail_width: TITLEBAR_RAIL.width,
            rail_padding_top: TITLEBAR_RAIL.padding_top,
            rail_padding_bottom: TITLEBAR_RAIL.padding_bottom,
            rail_spacing: TITLEBAR.buttons.spacing as f32,
            pill_inset: TITLEBAR_RAIL.pill_inset,
            pill_padding: (TITLEBAR_RAIL.pill_padding_x, TITLEBAR_RAIL.pill_padding_y),
            pill_shape: rounded(TITLEBAR_RAIL.pill_radius),
            rail_background: if island {
                StateValue::new(ShapeStyle::Clear)
            } else {
                StateValue::new(ShapeStyle::Color(resolved.surfaces.titlebar_inactive))
                    .focused(toplevel.clone())
            },
            drawer_background: StateValue::new(toplevel),
            title: StateValue::new(ShapeStyle::Color(ink.unfocused))
                .focused(ShapeStyle::Color(ink.focused)),
            pill_background: StateValue::new(ShapeStyle::Role(Role::RaisedBackground)),
            rail_line: (!island).then(|| (line.clone(), HAIRLINE)),
            line_edge: (line, TITLEBAR_RAIL.edge_line),
            accent_edge: (ShapeStyle::Accent, TITLEBAR_RAIL.edge_accent),
            pill_border: Some((ShapeStyle::Role(Role::Separator), HAIRLINE)),
            shadows: vec![ShadowStyle {
                color: ctdesign::Rgba {
                    a: contact.opacity,
                    ..contact.color
                },
                ..contact.into()
            }],
            effect: system_effect,
            title_angle: TITLEBAR_RAIL.title_angle,
            title_font: Font {
                size: TITLEBAR_RAIL.title_size as u32,
                ..TITLEBAR.font
            },
            animation: Animation::Ease {
                duration_ms: DURATION_BASE,
            },
        }
    }

    fn accessible(self, appearance: &Resolved) -> Self {
        if appearance.reduced_motion {
            Self {
                animation: Animation::None,
                ..self
            }
        } else {
            self
        }
    }
}

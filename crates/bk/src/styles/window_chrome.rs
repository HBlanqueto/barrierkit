use ctappearance::{ColorScheme, Resolved, SidebarStyle as SidebarKind, palette};
use ctdesign::{DURATION_BASE, Font, HAIRLINE, SHADOW, SHADOW_DARK, TITLEBAR, TITLEBAR_RAIL};

use super::button::rounded;
use super::{
    Animation, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    reduce_motion, system_effect,
};
use crate::chrome::{RailStyle, RestingEdge, Reveal, TitlebarLayout};
use crate::theme::{SurfaceKind, surface_background};

/// `WindowChrome::new(child).style(..)` takes the layout and the look of the
/// chrome. Its paints are `focused` while the window is active.
///
/// States: focused (the window is active). In full screen the rail sits on the
/// full-screen glass and draws no background. A `Theme` that sets `layout` takes
/// effect only inside a `WindowChrome`.
#[derive(Clone, Debug)]
pub struct WindowChromeStyle {
    /// Tengen: `Top`; `Rail` for the vertical bar.
    pub layout: TitlebarLayout,
    /// Tengen: `Normal`; `Unified` puts the column inside the sidebar.
    pub rail_style: RailStyle,
    /// Tengen: `Always` (9 (`docs/barrierkit/09-titlebars.md#reveal`)).
    pub reveal: Reveal,
    /// Tengen: `None`; `Line` draws `line_edge`, `Accent` `accent_edge`.
    pub resting_edge: RestingEdge,
    /// Tengen: `false`; `true` makes the window's traffic lights round.
    pub round_lights: bool,
    /// Tengen: `TITLEBAR_RAIL.width`, at least `TITLEBAR_RAIL.actions_width` with rail items.
    pub rail_width: f32,
    /// Tengen, with `rail_padding_bottom`, `rail_spacing`: `TITLEBAR_RAIL.padding_top`,
    /// `TITLEBAR_RAIL.padding_bottom`, `TITLEBAR.buttons.spacing`; the unified column takes the
    /// last two.
    pub rail_padding_top: f32,
    /// Tengen, with `rail_padding_top`, `rail_spacing`: `TITLEBAR_RAIL.padding_top`,
    /// `TITLEBAR_RAIL.padding_bottom`, `TITLEBAR.buttons.spacing`; the unified column takes the
    /// last two.
    pub rail_padding_bottom: f32,
    /// Tengen, with `rail_padding_top`, `rail_padding_bottom`: `TITLEBAR_RAIL.padding_top`,
    /// `TITLEBAR_RAIL.padding_bottom`, `TITLEBAR.buttons.spacing`; the unified column takes the
    /// last two.
    pub rail_spacing: f32,
    /// Tengen, with `pill_padding`: `TITLEBAR_RAIL.pill_inset`, `(pill_padding_x,
    /// pill_padding_y)` of the `Popup` pill.
    pub pill_inset: f32,
    /// Tengen, with `pill_inset`: `TITLEBAR_RAIL.pill_inset`, `(pill_padding_x,
    /// pill_padding_y)` of the `Popup` pill.
    pub pill_padding: (f32, f32),
    /// Tengen: `RoundedRectangle { radius: TITLEBAR_RAIL.pill_radius, style: Circular }`.
    pub pill_shape: Shape,
    /// Tengen: `Color` of `titlebar_inactive`, focused `Color` of the window surface; `Clear`
    /// with an island sidebar.
    pub rail_background: StateValue<ShapeStyle>,
    /// Tengen: `Color` of the window surface, the rail of `Reveal::Drawer`.
    pub drawer_background: StateValue<ShapeStyle>,
    /// Tengen: `Color` of the unfocused title, focused `Color` of the focused one.
    pub title: StateValue<ShapeStyle>,
    /// Tengen: `Role(RaisedBackground)`.
    pub pill_background: StateValue<ShapeStyle>,
    /// Tengen: `Some((Color of the sidebar line, HAIRLINE))` on the right of the rail; `None`
    /// with an island sidebar.
    pub rail_line: Option<(ShapeStyle, f32)>,
    /// Tengen: `(Color of the sidebar line, TITLEBAR_RAIL.edge_line)`.
    pub line_edge: (ShapeStyle, f32),
    /// Tengen: `(Accent, TITLEBAR_RAIL.edge_accent)`.
    pub accent_edge: (ShapeStyle, f32),
    /// Tengen: `Some((Role(Separator), HAIRLINE))`, drawn inside.
    pub pill_border: Option<(ShapeStyle, f32)>,
    /// Tengen: `[SHADOW.contact]` (`SHADOW_DARK.contact` in dark) with its `opacity`, under a
    /// drawer and the pill.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `TITLEBAR_RAIL.title_angle`, the rotation of the rail title in degrees.
    pub title_angle: f32,
    /// Tengen: `TITLEBAR.font` at `TITLEBAR_RAIL.title_size`; one line.
    pub title_font: Font,
    /// Tengen: `Ease { duration_ms: DURATION_BASE }`: a hidden bar slides and the lights fade
    /// in; `None` makes them jump.
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
        reduce_motion(self, appearance, |style| Self {
            animation: Animation::None,
            ..style
        })
    }
}

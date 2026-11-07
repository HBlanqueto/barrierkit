use ctappearance::{Resolved, SidebarStyle as SidebarKind, SidebarSurface};
use ctdesign::{DURATION_BASE, SETTINGS, SIDEBAR};

use super::{
    Animation, ColorEffect, ControlStyle, CornerStyle, Shape, ShapeStyle, StateValue,
    reduce_motion, system_effect,
};
use crate::sidebar::{SidebarBackdrop, over_backdrop};

/// `SidebarStyle` is not `ctappearance::SidebarStyle`, the user's island or flat
/// choice: the Tengen of each field follows that choice.
///
/// `.backdrop(SidebarBackdrop::Tinted)` is an instance override of `background`
/// and `border` over a `Backdrop`.
///
/// States: `SidebarItem` and the header of `SidebarSection` take hovered,
/// pressed and focused; the rest only `normal`. `SidebarItem::emphasized(false)`
/// is an instance override of `selected_background` (the inactive selection) and
/// `selected_label` (`label`). `SymbolTile::new(symbol, tint)` overrides
/// `background` with `Color(tint)` and `.rim(rgba)` sets `border`; like `Badge`,
/// `.style` builds the tile again, so it comes right after `new`. The avatar of
/// `SidebarAccount` takes `AvatarStyle`.
#[derive(Clone, Debug)]
pub struct SidebarStyle {
    /// Tengen, with `padding_top`, `padding_bottom`: `SIDEBAR.padding_x`;

    /// `SETTINGS.sidebar_padding_top` without traffic lights; `SIDEBAR.padding_x` (island) or

    /// `0.0` (flat).
    pub padding_x: f32,
    /// Tengen, with `padding_x`, `padding_bottom`: `SIDEBAR.padding_x`;
    /// `SETTINGS.sidebar_padding_top` without traffic lights; `SIDEBAR.padding_x` (island) or
    /// `0.0` (flat).
    pub padding_top: f32,
    /// Tengen, with `padding_x`, `padding_top`: `SIDEBAR.padding_x`;
    /// `SETTINGS.sidebar_padding_top` without traffic lights; `SIDEBAR.padding_x` (island) or
    /// `0.0` (flat).
    pub padding_bottom: f32,
    /// Tengen, with `lights_gap`: `(SIDEBAR.traffic_x, SIDEBAR.traffic_y)`;
    /// `SETTINGS.sidebar_lights_gap` between the lights and the first row.
    pub lights_position: (f32, f32),
    /// Tengen, with `lights_position`: `(SIDEBAR.traffic_x, SIDEBAR.traffic_y)`;
    /// `SETTINGS.sidebar_lights_gap` between the lights and the first row.
    pub lights_gap: f32,
    /// Tengen: `RoundedRectangle { radius: SIDEBAR.radius, style: Continuous }` (island),
    /// `Rectangle` (flat); a `Capsule` or `Circle` has no size to round by and stays square.
    pub shape: Shape,
    /// Tengen: `Color` of the island or flat surface; `Clear` over a `Backdrop`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen, with `inner_border`: island: `Color` of the rim with `SIDEBAR.rim_width` and of
    /// the inner line with `SIDEBAR.inner_width`, inside the rim; flat or over a `Backdrop`:
    /// `None`.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen, with `border`: island: `Color` of the rim with `SIDEBAR.rim_width` and of the
    /// inner line with `SIDEBAR.inner_width`, inside the rim; flat or over a `Backdrop`:
    /// `None`.
    pub inner_border: Option<(ShapeStyle, f32)>,
    /// Tengen: flat: `Some((Color of the sidebar line, SIDEBAR.inner_width))` on the right;
    /// island: `None`.
    pub line: Option<(ShapeStyle, f32)>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `Ease { duration_ms: DURATION_BASE }`: the first row slides to the title bar row
    /// when the chrome switches to the `Rail`; `None` makes it jump.
    pub animation: Animation,
}

impl SidebarStyle {
    pub(crate) fn surface(self, surface: SidebarSurface) -> Self {
        let island = surface.style == SidebarKind::Island;
        let visible = |color: ctdesign::Rgba, width| {
            (island && color.a > 0.0).then_some((ShapeStyle::Color(color), width))
        };
        Self {
            background: StateValue::new(ShapeStyle::Color(surface.bg)),
            border: visible(surface.rim, SIDEBAR.rim_width),
            inner_border: visible(surface.inner, SIDEBAR.inner_width),
            line: (!island)
                .then_some((ShapeStyle::Color(surface.palette.line), SIDEBAR.inner_width)),
            ..self
        }
    }
}

impl ControlStyle for SidebarStyle {
    fn tengen() -> Self {
        let surface = over_backdrop(crate::theme::resolved().sidebar, SidebarBackdrop::Clear);
        let island = surface.style == SidebarKind::Island;
        let padding_x = SIDEBAR.padding_x as f32;
        Self {
            padding_x,
            padding_top: SETTINGS.sidebar_padding_top as f32,
            padding_bottom: if island { padding_x } else { 0.0 },
            lights_position: (SIDEBAR.traffic_x as f32, SIDEBAR.traffic_y as f32),
            lights_gap: SETTINGS.sidebar_lights_gap as f32,
            shape: if island {
                Shape::RoundedRectangle {
                    radius: SIDEBAR.radius as f32,
                    style: CornerStyle::Continuous,
                }
            } else {
                Shape::Rectangle
            },
            background: StateValue::new(ShapeStyle::Clear),
            border: None,
            inner_border: None,
            line: None,
            effect: system_effect,
            animation: Animation::Ease {
                duration_ms: DURATION_BASE,
            },
        }
        .surface(surface)
    }

    fn accessible(self, appearance: &Resolved) -> Self {
        reduce_motion(self, appearance, |style| Self {
            animation: Animation::None,
            ..style
        })
    }
}

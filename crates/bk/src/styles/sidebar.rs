use ctappearance::SidebarSurface;
use ctdesign::{DURATION_BASE, SETTINGS, SIDEBAR};

use crate::sidebar::over_backdrop;
use crate::styles::{Animation, ControlStyle, ShapeStyle, StateValue};

/// The sidebar is flat and full height, as in macOS 27: no inset, rim or edge line.
///
/// States: `SidebarItem` and the header of `SidebarSection` take hovered,
/// pressed and focused; the rest only `normal`. `SidebarItem::emphasized(false)`
/// is an instance override of `selected_background` (the inactive selection) and
/// `selected_label` (`label`). `SymbolTile::new(symbol, tint)` overrides
/// `background` with `Color(tint)` and `symbol` with the light ink, also in an
/// inactive window, and `.rim(rgba)` sets `border`; like `Badge`,
/// `.style` builds the tile again, so it comes right after `new`. The avatar of
/// `SidebarAccount` takes `AvatarStyle`.
#[derive(Clone, Debug)]
pub struct SidebarStyle {
    /// Tengen: `SIDEBAR.padding_x`: the gap between the sidebar's edge and its rows.
    pub padding_x: f32,
    /// Tengen: `SETTINGS.sidebar_padding_top` without traffic lights.
    pub padding_top: f32,
    /// Tengen: `(SIDEBAR.traffic_x, SIDEBAR.traffic_y)`.
    pub lights_position: (f32, f32),
    /// Tengen: `SETTINGS.sidebar_lights_gap` between the lights and the first row.
    pub lights_gap: f32,
    /// Tengen: `Color` of the sidebar surface, its `inactive_bg` in an inactive window unless
    /// transparency is reduced; `Clear` over a `Backdrop`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Ease { duration_ms: DURATION_BASE }`: the first row slides to the title bar row
    /// when the chrome switches to the `Rail`; `None` makes it jump.
    pub animation: Animation,
}

impl SidebarStyle {
    pub(crate) fn surface(self, surface: SidebarSurface) -> Self {
        let inactive = !crate::platform::window_activated()
            && surface.bg.a > 0.0
            && !crate::theme::resolved().reduce_transparency;
        Self {
            background: StateValue::new(ShapeStyle::Color(if inactive {
                surface.palette.inactive_bg
            } else {
                surface.bg
            })),
            ..self
        }
    }
}

impl ControlStyle for SidebarStyle {
    fn tengen() -> Self {
        Self {
            padding_x: SIDEBAR.padding_x as f32,
            padding_top: SETTINGS.sidebar_padding_top as f32,
            lights_position: (SIDEBAR.traffic_x as f32, SIDEBAR.traffic_y as f32),
            lights_gap: SETTINGS.sidebar_lights_gap as f32,
            background: StateValue::new(ShapeStyle::Clear),
            animation: Animation::Ease {
                duration_ms: DURATION_BASE,
            },
        }
        .surface(over_backdrop(crate::theme::resolved().sidebar))
    }
}

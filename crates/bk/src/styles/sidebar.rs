use ctappearance::{Resolved, SidebarStyle as SidebarKind, SidebarSurface};
use ctdesign::{DURATION_BASE, SETTINGS, SIDEBAR};

use super::{
    Animation, ColorEffect, ControlStyle, CornerStyle, Shape, ShapeStyle, StateValue, system_effect,
};
use crate::sidebar::{SidebarBackdrop, over_backdrop};

#[derive(Clone, Debug)]
pub struct SidebarStyle {
    pub padding_x: f32,
    pub padding_top: f32,
    pub padding_bottom: f32,
    pub lights_position: (f32, f32),
    pub lights_gap: f32,
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub inner_border: Option<(ShapeStyle, f32)>,
    pub line: Option<(ShapeStyle, f32)>,
    pub effect: ColorEffect,
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

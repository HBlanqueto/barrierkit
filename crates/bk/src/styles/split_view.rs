use ctappearance::{Resolved, SidebarStyle as SidebarKind};
use ctdesign::{DURATION_BASE, DURATION_FAST_MS, HAIRLINE, SIDEBAR, SPLIT_VIEW};

use super::{
    Animation, ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, reduce_motion,
    system_effect,
};
use crate::theme::{SurfaceKind, surface_background};

#[derive(Clone, Debug)]
pub struct SplitViewStyle {
    pub inset: f32,
    pub inspector_width: f32,
    pub grip_width: f32,
    pub grip_line_width: f32,
    pub grip_line_inset: f32,
    pub grip_shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub grip: StateValue<ShapeStyle>,
    pub scrim: StateValue<ShapeStyle>,
    pub inspector_line: (ShapeStyle, f32),
    pub effect: ColorEffect,
    pub animation: Animation,
    pub resize_animation: Animation,
}

impl ControlStyle for SplitViewStyle {
    fn tengen() -> Self {
        let resolved = crate::theme::resolved();
        let island = resolved.sidebar.style == SidebarKind::Island;
        let accent = ShapeStyle::Color(resolved.accent);
        Self {
            inset: if island { SIDEBAR.inset as f32 } else { 0.0 },
            inspector_width: SPLIT_VIEW.inspector,
            grip_width: SPLIT_VIEW.grip,
            grip_line_width: SPLIT_VIEW.grip_line,
            grip_line_inset: SPLIT_VIEW.grip_line_inset,
            grip_shape: Shape::Capsule,
            background: StateValue::new(if crate::backdrop::tinted().is_some() {
                ShapeStyle::Clear
            } else {
                ShapeStyle::Color(surface_background(&resolved, SurfaceKind::Toplevel))
            }),
            grip: StateValue::new(ShapeStyle::Clear)
                .hovered(accent.clone())
                .pressed(accent.clone())
                .focused(accent),
            scrim: StateValue::new(ShapeStyle::Role(Role::Scrim)),
            inspector_line: (ShapeStyle::Role(Role::Separator), HAIRLINE),
            effect: system_effect,
            animation: Animation::Ease {
                duration_ms: DURATION_BASE,
            },
            resize_animation: Animation::Ease {
                duration_ms: DURATION_FAST_MS,
            },
        }
    }

    fn accessible(self, appearance: &Resolved) -> Self {
        reduce_motion(self, appearance, |style| Self {
            animation: Animation::None,
            resize_animation: Animation::None,
            ..style
        })
    }
}

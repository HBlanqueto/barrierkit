use ctappearance::{Resolved, SidebarStyle as SidebarKind};
use ctdesign::{DURATION_BASE, DURATION_FAST_MS, HAIRLINE, SIDEBAR, SPLIT_VIEW};

use super::{
    Animation, ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, reduce_motion,
    system_effect,
};
use crate::theme::{SurfaceKind, surface_background};

/// The widths of a `SplitView` sidebar (`.sidebar_width`, `.sidebar_range`) stay
/// options: they are layout the user changes.
///
/// `SidebarToggle` is an `IconButton` and takes `IconButtonStyle`.
///
/// States: `FloatingPane` is focused while `active`; `SplitPanes` only `normal`.
/// `SplitPanes::divider(rgba)` is an instance override of `divider`. The fields
/// `accent`, `title_bar` and `title_ink` of `FloatingPane` are instance overrides
/// of `outline` (focused), `title_bar` and `title`, applied before `.style`.
#[derive(Clone, Debug)]
pub struct SplitViewStyle {
    /// Tengen: `SIDEBAR.inset` around an island sidebar, `0.0` around a flat one.
    pub inset: f32,
    /// Tengen: `SPLIT_VIEW.inspector`.
    pub inspector_width: f32,
    /// Tengen, with `grip_line_width`, `grip_line_inset`: `SPLIT_VIEW.grip`, the area that
    /// resizes; `SPLIT_VIEW.grip_line`, `SPLIT_VIEW.grip_line_inset` from the top and the
    /// bottom.
    pub grip_width: f32,
    /// Tengen, with `grip_width`, `grip_line_inset`: `SPLIT_VIEW.grip`, the area that resizes;
    /// `SPLIT_VIEW.grip_line`, `SPLIT_VIEW.grip_line_inset` from the top and the bottom.
    pub grip_line_width: f32,
    /// Tengen, with `grip_width`, `grip_line_width`: `SPLIT_VIEW.grip`, the area that resizes;
    /// `SPLIT_VIEW.grip_line`, `SPLIT_VIEW.grip_line_inset` from the top and the bottom.
    pub grip_line_inset: f32,
    /// Tengen: `Capsule`.
    pub grip_shape: Shape,
    /// Tengen: `Color` of the window surface; `Clear` over a `Backdrop`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Clear`, hovered, pressed (dragging) and focused (keyboard) `Color` of the
    /// accent.
    pub grip: StateValue<ShapeStyle>,
    /// Tengen: `Role(Scrim)`, behind the sidebar over the content of a narrow window.
    pub scrim: StateValue<ShapeStyle>,
    /// Tengen: `(Role(Separator), HAIRLINE)`.
    pub inspector_line: (ShapeStyle, f32),
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `Ease { duration_ms: DURATION_BASE }`: the sidebar and the inspector slide in
    /// and out.
    pub animation: Animation,
    /// Tengen: `Ease { duration_ms: DURATION_FAST_MS }`: the sidebar follows a resize from the
    /// keyboard; a drag follows the pointer.
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

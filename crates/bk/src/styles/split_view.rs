use ctappearance::Resolved;
use ctdesign::{DURATION_BASE, HAIRLINE, SPLIT_VIEW, ease_in_ease_out};

use crate::styles::{Animation, ControlStyle, Role, Shape, ShapeStyle, StateValue, reduce_motion};

/// The widths of a `SplitView` sidebar (`.sidebar_width`, `.sidebar_range`) stay
/// options: they are layout the user changes.
///
/// `SidebarToggle` is an `IconButton` and takes `IconButtonStyle`.
///
/// States: `SplitPanes` only `normal`. `SplitPanes::divider(rgba)` is an instance
/// override of `divider`.
#[derive(Clone, Debug)]
pub struct SplitViewStyle {
    /// Tengen: `SPLIT_VIEW.inspector`.
    pub inspector_width: f32,
    pub grip_width: f32,
    pub grip_line_width: f32,
    pub grip_line_inset: f32,
    /// Tengen: `Capsule`.
    pub grip_shape: Shape,
    /// Tengen: `WindowBackground`; `Clear` over a `Backdrop`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `Clear`; focused (keyboard) `Color` of the accent. Hover and drag show only the
    /// cursor, like an AppKit split view divider.
    pub grip: StateValue<ShapeStyle>,
    /// Tengen: `Role(Scrim)`, behind the sidebar over the content of a narrow window.
    pub scrim: StateValue<ShapeStyle>,
    /// Tengen: `(Role(Separator), HAIRLINE)`.
    pub inspector_line: (ShapeStyle, f32),
    /// Tengen: `Custom { DURATION_BASE, ease_in_ease_out }`: the sidebar slides when the user
    /// toggles it or inside `with_animation`, otherwise it appears in one frame; the inspector
    /// always slides. A resize of the sidebar never animates.
    pub animation: Animation,
}

impl ControlStyle for SplitViewStyle {
    fn tengen() -> Self {
        let resolved = crate::theme::resolved();
        let accent = ShapeStyle::Color(resolved.accent);
        Self {
            inspector_width: SPLIT_VIEW.inspector,
            grip_width: SPLIT_VIEW.grip,
            grip_line_width: SPLIT_VIEW.grip_line,
            grip_line_inset: SPLIT_VIEW.grip_line_inset,
            grip_shape: Shape::Capsule,
            background: StateValue::new(if crate::backdrop::tinted().is_some() {
                ShapeStyle::Clear
            } else {
                ShapeStyle::WindowBackground
            }),
            grip: StateValue::new(ShapeStyle::Clear).focused(accent),
            scrim: StateValue::new(ShapeStyle::Role(Role::Scrim)),
            inspector_line: (ShapeStyle::Role(Role::Separator), HAIRLINE),
            animation: Animation::Custom {
                duration_ms: DURATION_BASE,
                curve: ease_in_ease_out,
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

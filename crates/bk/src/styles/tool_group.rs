use ctappearance::Resolved;
use ctdesign::{HAIRLINE, TOOLGROUP};

use crate::titlebar::WindowToolbarStyle;

use super::{
    Axis, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    show_borders, system_effect,
};

/// States: only `normal`. Tengen follows reduce transparency, which picks the opaque tool
/// group colors.
#[derive(Clone, Debug)]
pub struct ToolGroupStyle {
    /// Tengen: `(TOOLGROUP.item_width, TOOLGROUP.height)`: the slot of each item along the axis and
    /// across it, which is the thickness of the group; in a compact toolbar
    /// `(TOOLGROUP.compact_width, TOOLGROUP.compact_height)`, in an expanded one the `expanded_` pair.
    pub item_size: (f32, f32),
    /// Tengen: `TOOLGROUP.padding_x`, at both ends along the axis; in a compact toolbar minus
    /// `TOOLGROUP.compact_inset` (expanded: `expanded_inset`), so the first and last slot reach past
    /// the platter.
    pub padding: f32,
    /// Tengen: `(TOOLGROUP.height - TOOLGROUP.item_width) / 2` for a group of one item, so it is a
    /// circle; in a compact or expanded toolbar `padding`.
    pub padding_alone: f32,
    /// Tengen: `(TOOLGROUP.separator_w, TOOLGROUP.separator_h)`: along the axis, across it; in a
    /// compact toolbar `TOOLGROUP.compact_separator_h` across.
    pub separator_size: (f32, f32),
    /// Tengen: `Capsule`.
    pub shape: Shape,
    /// Tengen: `Color` of the resolved tool group background, or `Role(ControlBackground)`
    /// inside a tinted `Backdrop`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `None` in a unified toolbar, whose group has no divider between its items; in a
    /// compact toolbar the `Color` of the tool group separator, or `Role(Separator)` inside a
    /// tinted `Backdrop`.
    pub separator: Option<StateValue<ShapeStyle>>,
    /// Tengen: the tool group rim, or `Role(CardBorder)`, `HAIRLINE` wide.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `Horizontal`; `Vertical` stacks the items.
    pub axis: Axis,
}

impl ControlStyle for ToolGroupStyle {
    fn tengen() -> Self {
        let resolved = crate::theme::resolved();
        let (background, separator, rim) = if crate::backdrop::tinted().is_some() {
            (
                ShapeStyle::Role(Role::ControlBackground),
                ShapeStyle::Role(Role::Separator),
                ShapeStyle::Role(Role::CardBorder),
            )
        } else {
            let toolgroup = resolved.toolgroup;
            (
                ShapeStyle::Color(toolgroup.bg),
                ShapeStyle::Color(toolgroup.palette.separator),
                ShapeStyle::Color(toolgroup.palette.rim),
            )
        };
        let compact = crate::compact::toolbar_style() == Some(WindowToolbarStyle::UnifiedCompact);
        let slim = crate::compact::slim_toolbar_items();
        let height = TOOLGROUP.height as f32;
        Self {
            item_size: slim.map_or((TOOLGROUP.item_width, height), |(width, height, _)| {
                (width, height)
            }),
            padding: slim.map_or(TOOLGROUP.padding_x, |(_, _, inset)| -inset),
            padding_alone: slim.map_or((height - TOOLGROUP.item_width) / 2.0, |(_, _, inset)| {
                -inset
            }),
            separator_size: (
                TOOLGROUP.separator_w as f32,
                if compact {
                    TOOLGROUP.compact_separator_h
                } else {
                    TOOLGROUP.separator_h
                } as f32,
            ),
            shape: Shape::Capsule,
            background: StateValue::new(background),
            separator: compact.then(|| StateValue::new(separator)),
            border: Some((rim, HAIRLINE)),
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
            axis: Axis::Horizontal,
        }
    }

    fn accessible(self, appearance: &Resolved) -> Self {
        show_borders(self, appearance, |style| Self {
            border: Some((ShapeStyle::Role(Role::ControlBorder), HAIRLINE)),
            ..style
        })
    }
}

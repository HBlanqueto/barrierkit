use ctappearance::SidebarStyle;
use ctdesign::{HAIRLINE, TOOLGROUP};

use super::{
    Axis, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    system_effect,
};

/// States: only `normal`. Tengen follows the user's sidebar style and reduce
/// transparency, which picks the opaque tool group colors.
#[derive(Clone, Debug)]
pub struct ToolGroupStyle {
    /// Tengen: `TOOLGROUP.height`: the square slot of each item and the thickness of the
    /// group.
    pub item_size: f32,
    /// Tengen: `TOOLGROUP.padding_x`, at both ends along the axis.
    pub padding: f32,
    /// Tengen: `(TOOLGROUP.separator_w, TOOLGROUP.separator_h)`: along the axis, across it.
    pub separator_size: (f32, f32),
    /// Tengen: `Capsule`.
    pub shape: Shape,
    /// Tengen: Island sidebar: `Color` of the resolved tool group background, or
    /// `Role(ControlBackground)` inside a tinted `Backdrop`; Flat: `Clear`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: Island: `Color` of the tool group separator, or `Role(Separator)` inside a
    /// tinted `Backdrop`; Flat: `None`, no separators.
    pub separator: Option<StateValue<ShapeStyle>>,
    /// Tengen: Island: the tool group rim, or `Role(CardBorder)`, `HAIRLINE` wide; Flat:
    /// `None`.
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
        let island = resolved.sidebar.style == SidebarStyle::Island;
        Self {
            item_size: TOOLGROUP.height as f32,
            padding: TOOLGROUP.padding_x as f32,
            separator_size: (TOOLGROUP.separator_w as f32, TOOLGROUP.separator_h as f32),
            shape: Shape::Capsule,
            background: StateValue::new(if island {
                background
            } else {
                ShapeStyle::Clear
            }),
            separator: island.then(|| StateValue::new(separator)),
            border: island.then_some((rim, HAIRLINE)),
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
            axis: Axis::Horizontal,
        }
    }
}

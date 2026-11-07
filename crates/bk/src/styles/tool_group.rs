use ctappearance::{Resolved, SidebarStyle};
use ctdesign::{HAIRLINE, TOOLGROUP};

use super::{
    Axis, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    system_effect,
};

#[derive(Clone, Debug)]
pub struct ToolGroupStyle {
    pub item_size: f32,
    pub padding: f32,
    pub separator_size: (f32, f32),
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub separator: Option<StateValue<ShapeStyle>>,
    pub border: Option<(ShapeStyle, f32)>,
    pub shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
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

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

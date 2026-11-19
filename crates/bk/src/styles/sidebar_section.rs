use ctappearance::Resolved;
use ctdesign::{BODY, DURATION_BASE, Font, SIDEBAR};

use super::{
    Animation, ColorEffect, ControlStyle, ShapeStyle, StateValue, reduce_motion, system_effect,
};

#[derive(Clone, Debug)]
pub struct SidebarSectionStyle {
    /// Tengen: `SIDEBAR.section_height`.
    pub height: f32,
    /// Tengen: `section_margin_top`.
    pub margin_top: f32,
    /// Tengen: `section_padding_x`.
    pub padding_x: f32,
    /// Tengen: `chevron`.
    pub chevron_size: f32,
    /// Tengen, with `chevron`: `title`, `chevron`: `Color` of the section ink.
    pub title: StateValue<ShapeStyle>,
    /// Tengen, with `title`: `title`, `chevron`: `Color` of the section ink.
    pub chevron: StateValue<ShapeStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `font`: `BODY` at `SIDEBAR.section_size` with `SIDEBAR.section_weight`.
    pub font: Font,
    /// Tengen: `Ease { duration_ms: DURATION_BASE }`: the rows fold; `None` makes them jump.
    pub animation: Animation,
}

impl ControlStyle for SidebarSectionStyle {
    fn tengen() -> Self {
        let ink = ShapeStyle::Color(crate::sidebar::sidebar_palette().section_ink);
        Self {
            height: SIDEBAR.section_height as f32,
            margin_top: SIDEBAR.section_margin_top as f32,
            padding_x: SIDEBAR.section_padding_x as f32,
            chevron_size: SIDEBAR.chevron as f32,
            title: StateValue::new(ink.clone()),
            chevron: StateValue::new(ink),
            effect: system_effect,
            font: Font {
                size: SIDEBAR.section_size,
                weight: SIDEBAR.section_weight,
                ..BODY
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

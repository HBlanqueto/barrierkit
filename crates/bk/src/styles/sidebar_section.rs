use ctdesign::{BODY, Font, SIDEBAR, SPRING_SHEET_MS};

use crate::styles::{Animation, ColorEffect, ControlStyle, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct SidebarSectionStyle {
    /// Tengen: `SIDEBAR.section_height`.
    pub height: f32,
    /// Tengen: `section_margin_bottom`: the gap to the next section.
    pub margin_bottom: f32,
    /// Tengen: `section_padding_x`.
    pub padding_x: f32,
    /// Tengen: `chevron`.
    pub chevron_size: f32,
    pub title: StateValue<ShapeStyle>,
    pub chevron: StateValue<ShapeStyle>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `font`: `BODY` at `SIDEBAR.section_size` with `SIDEBAR.section_weight`.
    pub font: Font,
    /// Tengen: `Spring { duration_ms: SPRING_SHEET_MS, bounce: 0.0 }`, the measured spring of a card changing height: the rows fold; `None` makes them jump.
    pub animation: Animation,
}

impl ControlStyle for SidebarSectionStyle {
    fn tengen() -> Self {
        let ink = ShapeStyle::Color(crate::sidebar::sidebar_palette().section_ink);
        Self {
            height: SIDEBAR.section_height as f32,
            margin_bottom: SIDEBAR.section_margin_bottom as f32,
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
            animation: Animation::Spring {
                duration_ms: SPRING_SHEET_MS,
                bounce: 0.0,
            },
        }
    }
}

use ctappearance::Resolved;
use ctdesign::{BODY, DURATION_BASE, Font, SIDEBAR};

use super::{Animation, ColorEffect, ControlStyle, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct SidebarSectionStyle {
    pub height: f32,
    pub margin_top: f32,
    pub padding_x: f32,
    pub chevron_size: f32,
    pub title: StateValue<ShapeStyle>,
    pub chevron: StateValue<ShapeStyle>,
    pub effect: ColorEffect,
    pub font: Font,
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

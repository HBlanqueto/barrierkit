use ctdesign::{BODY, Font, PATH_CONTROL};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct PathControlStyle {
    pub height: f32,
    pub gap: f32,
    pub icon_size: f32,
    pub icon_gap: f32,
    pub chevron_size: f32,
    pub icon: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub chevron: StateValue<ShapeStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
    pub font: Font,
}

impl ControlStyle for PathControlStyle {
    fn tengen() -> Self {
        Self {
            height: PATH_CONTROL.height,
            gap: PATH_CONTROL.gap,
            icon_size: PATH_CONTROL.icon,
            icon_gap: PATH_CONTROL.icon_gap,
            chevron_size: PATH_CONTROL.chevron,
            icon: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            chevron: StateValue::new(ShapeStyle::Role(Role::TertiaryLabel)),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
        }
    }
}

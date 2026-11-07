use ctdesign::{BODY, Font, PATH_CONTROL};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

/// States: only `normal`.
#[derive(Clone, Debug)]
pub struct PathControlStyle {
    /// Tengen, with `gap`: `PATH_CONTROL.height`, `PATH_CONTROL.gap` between parts and

    /// chevrons.
    pub height: f32,
    /// Tengen, with `height`: `PATH_CONTROL.height`, `PATH_CONTROL.gap` between parts and
    /// chevrons.
    pub gap: f32,
    /// Tengen: `PATH_CONTROL.icon`.
    pub icon_size: f32,
    /// Tengen: `PATH_CONTROL.icon_gap`.
    pub icon_gap: f32,
    /// Tengen: `PATH_CONTROL.chevron`.
    pub chevron_size: f32,
    /// Tengen: `Role(SecondaryLabel)`.
    pub icon: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Role(TertiaryLabel)`.
    pub chevron: StateValue<ShapeStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `BODY`.
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

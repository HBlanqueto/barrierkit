use ctdesign::{CAPTION, Font, HAIRLINE, SETTINGS, SWITCH};

use super::{
    Axis, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    system_effect,
};

#[derive(Clone, Debug)]
pub struct ColorWellStyle {
    pub swatch_size: f32,
    pub column_width: f32,
    pub name_gap: f32,
    pub spacing: f32,
    pub swatch_shape: Shape,
    pub selection: StateValue<ShapeStyle>,
    pub name: StateValue<ShapeStyle>,
    pub swatch_border: Option<(ShapeStyle, f32)>,
    pub swatch_shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub disabled_opacity: f32,
    pub effect: ColorEffect,
    pub axis: Axis,
    pub wraps: bool,
    pub name_font: Font,
}

impl ControlStyle for ColorWellStyle {
    fn tengen() -> Self {
        Self {
            swatch_size: SETTINGS.swatch_size as f32,
            column_width: SETTINGS.swatch_column as f32,
            name_gap: SETTINGS.swatch_name_gap as f32,
            spacing: SETTINGS.swatch_gap as f32,
            swatch_shape: Shape::Circle,
            selection: StateValue::new(ShapeStyle::Role(Role::Label)),
            name: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            swatch_border: Some((ShapeStyle::Role(Role::Separator), HAIRLINE)),
            swatch_shadows: Vec::new(),
            opacity: 1.0,
            disabled_opacity: SWITCH.disabled_opacity,
            effect: system_effect,
            axis: Axis::Horizontal,
            wraps: false,
            name_font: CAPTION,
        }
    }
}

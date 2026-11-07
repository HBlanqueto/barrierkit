use ctappearance::Resolved;
use ctdesign::{BODY, Font, HAIRLINE, SETTINGS};

use super::{
    Axis, ColorEffect, ControlStyle, Edge, LayoutDirection, Role, ShadowStyle, Shape, ShapeStyle,
    StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct RadioGroupStyle {
    pub button_size: f32,
    pub dot_size: f32,
    pub label_gap: f32,
    pub spacing: f32,
    pub button_shape: Shape,
    pub dot_shape: Shape,
    pub button_background: StateValue<ShapeStyle>,
    pub fill: StateValue<ShapeStyle>,
    pub dot: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub button_border: Option<(ShapeStyle, f32)>,
    pub button_shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub disabled_opacity: f32,
    pub effect: ColorEffect,
    pub axis: Axis,
    pub label_position: Option<Edge>,
    pub layout_direction: LayoutDirection,
    pub label_font: Font,
}

impl ControlStyle for RadioGroupStyle {
    fn tengen() -> Self {
        Self {
            button_size: SETTINGS.radio_size as f32,
            dot_size: SETTINGS.radio_dot as f32,
            label_gap: SETTINGS.radio_gap as f32,
            spacing: SETTINGS.radio_spacing as f32,
            button_shape: Shape::Circle,
            dot_shape: Shape::Circle,
            button_background: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            fill: StateValue::new(ShapeStyle::Accent),
            dot: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            label: StateValue::new(ShapeStyle::Role(Role::Label))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            button_border: Some((ShapeStyle::Role(Role::Separator), HAIRLINE)),
            button_shadows: Vec::new(),
            opacity: 1.0,
            disabled_opacity: 1.0,
            effect: system_effect,
            axis: Axis::Vertical,
            label_position: Some(Edge::Trailing),
            layout_direction: LayoutDirection::LeftToRight,
            label_font: BODY,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

use ctappearance::Resolved;
use ctdesign::{BODY, CHECKBOX, Font, HAIRLINE, SETTINGS};

use super::{
    ColorEffect, ControlStyle, CornerStyle, Edge, LayoutDirection, Role, ShadowStyle, Shape,
    ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct CheckboxStyle {
    pub box_size: f32,
    pub mark_size: f32,
    pub label_gap: f32,
    pub box_shape: Shape,
    pub box_background: StateValue<ShapeStyle>,
    pub fill: StateValue<ShapeStyle>,
    pub mark: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub box_border: Option<(ShapeStyle, f32)>,
    pub box_shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub disabled_opacity: f32,
    pub effect: ColorEffect,
    pub label_position: Option<Edge>,
    pub layout_direction: LayoutDirection,
    pub label_font: Font,
}

impl ControlStyle for CheckboxStyle {
    fn tengen() -> Self {
        Self {
            box_size: CHECKBOX.size,
            mark_size: CHECKBOX.mark,
            label_gap: SETTINGS.check.gap as f32,
            box_shape: Shape::RoundedRectangle {
                radius: CHECKBOX.radius,
                style: CornerStyle::Circular,
            },
            box_background: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            fill: StateValue::new(ShapeStyle::Accent),
            mark: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            label: StateValue::new(ShapeStyle::Role(Role::Label))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            box_border: Some((ShapeStyle::Role(Role::Separator), HAIRLINE)),
            box_shadows: Vec::new(),
            opacity: 1.0,
            disabled_opacity: 1.0,
            effect: system_effect,
            label_position: Some(Edge::Trailing),
            layout_direction: LayoutDirection::LeftToRight,
            label_font: BODY,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

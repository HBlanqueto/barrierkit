use ctdesign::{BODY, Font, GROUP_BOX, HAIRLINE, Rgba, SPACE_1};

use super::button::rounded;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};
use crate::style::Palette;

#[derive(Clone, Debug)]
pub struct GroupBoxStyle {
    pub padding: f32,
    pub spacing: f32,
    pub label_inset: f32,
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub effect: ColorEffect,
    pub font: Font,
}

impl ControlStyle for GroupBoxStyle {
    fn tengen() -> Self {
        Self {
            padding: GROUP_BOX.padding,
            spacing: SPACE_1,
            label_inset: GROUP_BOX.label_inset,
            shape: rounded(GROUP_BOX.radius),
            background: StateValue::new(ShapeStyle::Color(Rgba {
                a: GROUP_BOX.fill_opacity,
                ..Palette::current().label
            })),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            border: Some((ShapeStyle::Role(Role::CardBorder), HAIRLINE)),
            effect: system_effect,
            font: BODY,
        }
    }
}

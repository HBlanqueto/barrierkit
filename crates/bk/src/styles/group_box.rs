use ctdesign::{CAPTION, Font, GROUP_BOX, HAIRLINE, Rgba};

use crate::style::Palette;
use crate::styles::button::continuous_rounded;
use crate::styles::{
    ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct GroupBoxStyle {
    /// Tengen: `GROUP_BOX.padding`.
    pub padding: f32,
    /// Tengen: `0.0` between the label and the box.
    pub spacing: f32,
    /// Tengen: `GROUP_BOX.label_inset`.
    pub label_inset: f32,
    /// Tengen: `GROUP_BOX.label_height`: the label row is shorter than its line, so the label
    /// overlaps the top of the box.
    pub label_height: f32,
    /// Tengen: `RoundedRectangle { radius: GROUP_BOX.radius, style: Continuous }`; a `Capsule` or
    /// `Circle` stays square.
    pub shape: Shape,
    /// Tengen: `background` `Color` of the label color at `GROUP_BOX.fill_opacity`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen: `label` `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(CardBorder), HAIRLINE))`, drawn inside.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `CAPTION`.
    pub font: Font,
}

impl ControlStyle for GroupBoxStyle {
    fn tengen() -> Self {
        Self {
            padding: GROUP_BOX.padding,
            spacing: 0.0,
            label_inset: GROUP_BOX.label_inset,
            label_height: GROUP_BOX.label_height,
            shape: continuous_rounded(GROUP_BOX.radius),
            background: StateValue::new(ShapeStyle::Color(Rgba {
                a: GROUP_BOX.fill_opacity,
                ..Palette::current().label
            })),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            border: Some((ShapeStyle::Role(Role::CardBorder), HAIRLINE)),
            effect: system_effect,
            font: CAPTION,
        }
    }
}

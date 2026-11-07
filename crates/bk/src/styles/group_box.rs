use ctdesign::{BODY, Font, GROUP_BOX, HAIRLINE, Rgba, SPACE_1};

use super::button::rounded;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};
use crate::style::Palette;

#[derive(Clone, Debug)]
pub struct GroupBoxStyle {
    /// Tengen, with `spacing`, `label_inset`: `padding`, `spacing`, `label_inset`:

    /// `GROUP_BOX.padding`, `SPACE_1` between the label and the box, `GROUP_BOX.label_inset`.
    pub padding: f32,
    /// Tengen, with `padding`, `label_inset`: `padding`, `spacing`, `label_inset`:
    /// `GROUP_BOX.padding`, `SPACE_1` between the label and the box, `GROUP_BOX.label_inset`.
    pub spacing: f32,
    /// Tengen, with `padding`, `spacing`: `padding`, `spacing`, `label_inset`:
    /// `GROUP_BOX.padding`, `SPACE_1` between the label and the box, `GROUP_BOX.label_inset`.
    pub label_inset: f32,
    /// Tengen: `RoundedRectangle { radius: GROUP_BOX.radius, style: Circular }`; a `Capsule` or
    /// `Circle` stays square.
    pub shape: Shape,
    /// Tengen, with `label`: `background` `Color` of the label color at
    /// `GROUP_BOX.fill_opacity`; `label` `Role(Label)`.
    pub background: StateValue<ShapeStyle>,
    /// Tengen, with `background`: `background` `Color` of the label color at
    /// `GROUP_BOX.fill_opacity`; `label` `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(CardBorder), HAIRLINE))`, drawn inside.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `BODY`.
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

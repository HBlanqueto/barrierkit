use ctappearance::Resolved;
use ctdesign::{BODY, Font, SETTINGS, TEXT_FIELD};

use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct SearchFieldStyle {
    pub height: f32,
    pub padding_x: f32,
    pub icon_size: f32,
    pub icon_gap: f32,
    pub shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub border: StateValue<ShapeStyle>,
    pub text: StateValue<ShapeStyle>,
    pub placeholder: StateValue<ShapeStyle>,
    pub icon: StateValue<ShapeStyle>,
    pub clear_icon: StateValue<ShapeStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
    pub font: Font,
}

impl ControlStyle for SearchFieldStyle {
    fn tengen() -> Self {
        Self {
            height: TEXT_FIELD.height,
            padding_x: SETTINGS.search_padding_x as f32,
            icon_size: SETTINGS.search_icon as f32,
            icon_gap: SETTINGS.search_gap as f32,
            shape: Shape::Capsule,
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            border: StateValue::new(ShapeStyle::Clear).focused(ShapeStyle::Accent),
            text: StateValue::new(ShapeStyle::Role(Role::Label)),
            placeholder: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            icon: StateValue::new(ShapeStyle::Role(Role::TertiaryLabel)),
            clear_icon: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            opacity: 1.0,
            effect: system_effect,
            font: BODY,
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

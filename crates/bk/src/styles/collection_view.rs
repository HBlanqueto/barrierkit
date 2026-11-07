use ctdesign::{CAPTION, COLLECTION, Font, Rgba};

use super::button::rounded;
use super::{ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect};
use crate::style::Palette;

#[derive(Clone, Debug)]
pub struct CollectionViewStyle {
    pub tile_width: f32,
    pub icon_size: f32,
    pub spacing: f32,
    pub padding: f32,
    pub label_spacing: f32,
    pub label_padding_x: f32,
    pub shape: Shape,
    pub label_shape: Shape,
    pub icon_background: StateValue<ShapeStyle>,
    pub selected_icon_background: StateValue<ShapeStyle>,
    pub label_background: StateValue<ShapeStyle>,
    pub selected_label_background: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub selected_label: StateValue<ShapeStyle>,
    pub effect: ColorEffect,
    pub font: Font,
}

impl ControlStyle for CollectionViewStyle {
    fn tengen() -> Self {
        Self {
            tile_width: COLLECTION.tile,
            icon_size: COLLECTION.icon,
            spacing: COLLECTION.gap,
            padding: COLLECTION.gap,
            label_spacing: COLLECTION.gap / 2.0,
            label_padding_x: COLLECTION.gap,
            shape: rounded(COLLECTION.radius),
            label_shape: rounded(COLLECTION.gap),
            icon_background: StateValue::new(ShapeStyle::Clear),
            selected_icon_background: StateValue::new(ShapeStyle::Color(Rgba {
                a: COLLECTION.selection,
                ..Palette::current().label
            })),
            label_background: StateValue::new(ShapeStyle::Clear),
            selected_label_background: StateValue::new(ShapeStyle::Accent),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            selected_label: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            effect: system_effect,
            font: CAPTION,
        }
    }
}

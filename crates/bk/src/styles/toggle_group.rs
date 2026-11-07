use ctdesign::{BODY, Font, ICON_BUTTON, LARGE_TITLE, SEGMENTED};

use super::button::rounded;
use super::{
    Axis, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    system_effect,
};

#[derive(Clone, Debug)]
pub struct ToggleGroupStyle {
    pub padding: f32,
    pub gap: f32,
    pub item_size: (f32, f32),
    pub shape: Shape,
    pub item_shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub item: StateValue<ShapeStyle>,
    pub selected: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub selected_label: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub selected_shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
    pub axis: Axis,
    pub font: Font,
    pub bold_font: Font,
}

impl ControlStyle for ToggleGroupStyle {
    fn tengen() -> Self {
        Self {
            padding: SEGMENTED.padding,
            gap: SEGMENTED.gap,
            item_size: (ICON_BUTTON.size, SEGMENTED.segment_height),
            shape: rounded(SEGMENTED.radius),
            item_shape: rounded(SEGMENTED.segment_radius),
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            item: StateValue::new(ShapeStyle::Clear),
            selected: StateValue::new(ShapeStyle::Accent),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            selected_label: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            border: None,
            selected_shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
            axis: Axis::Horizontal,
            font: Font {
                weight: SEGMENTED.font_weight as u16,
                ..BODY
            },
            bold_font: Font {
                weight: LARGE_TITLE.weight,
                ..BODY
            },
        }
    }
}

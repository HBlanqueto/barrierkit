use ctappearance::Resolved;
use ctdesign::{BODY, Font, KNOB_SHADOW, SEGMENTED};

use super::{
    Axis, ColorEffect, ControlStyle, CornerStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    system_effect,
};

#[derive(Clone, Debug)]
pub struct SegmentedControlStyle {
    pub padding: f32,
    pub gap: f32,
    pub segment_height: f32,
    pub segment_padding_x: f32,
    pub shape: Shape,
    pub segment_shape: Shape,
    pub background: StateValue<ShapeStyle>,
    pub segment: StateValue<ShapeStyle>,
    pub selected: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub selected_label: StateValue<ShapeStyle>,
    pub border: Option<(ShapeStyle, f32)>,
    pub selected_shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
    pub axis: Axis,
    pub font: Font,
}

impl ControlStyle for SegmentedControlStyle {
    fn tengen() -> Self {
        let rounded = |radius| Shape::RoundedRectangle {
            radius,
            style: CornerStyle::Circular,
        };
        Self {
            padding: SEGMENTED.padding,
            gap: SEGMENTED.gap,
            segment_height: SEGMENTED.segment_height,
            segment_padding_x: SEGMENTED.segment_padding_x,
            shape: rounded(SEGMENTED.radius),
            segment_shape: rounded(SEGMENTED.segment_radius),
            background: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            segment: StateValue::new(ShapeStyle::Clear),
            selected: StateValue::new(ShapeStyle::Role(Role::SelectedControl)),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            selected_label: StateValue::new(ShapeStyle::Role(Role::Label)),
            border: None,
            selected_shadows: vec![KNOB_SHADOW.into()],
            opacity: 1.0,
            effect: system_effect,
            axis: Axis::Horizontal,
            font: Font {
                size: SEGMENTED.font_size as u32,
                weight: SEGMENTED.font_weight as u16,
                ..BODY
            },
        }
    }

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

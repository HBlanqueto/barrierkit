use ctdesign::{BODY, Font, KNOB_SHADOW, SEGMENTED};

use super::button::rounded;
use super::{
    Axis, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    system_effect,
};

/// States: each segment is hovered, pressed and focused on its own; the group
/// has only `normal`. The arrow keys move the choice on both axes.
#[derive(Clone, Debug)]
pub struct SegmentedControlStyle {
    /// Tengen: `SEGMENTED.padding`.
    pub padding: f32,
    /// Tengen: `SEGMENTED.gap`.
    pub gap: f32,
    /// Tengen: `SEGMENTED.segment_height`.
    pub segment_height: f32,
    /// Tengen: `SEGMENTED.segment_padding_x`.
    pub segment_padding_x: f32,
    /// Tengen, with `segment_shape`: `RoundedRectangle` of `SEGMENTED.radius` and of
    /// `SEGMENTED.segment_radius`, both `Circular`.
    pub shape: Shape,
    /// Tengen, with `shape`: `RoundedRectangle` of `SEGMENTED.radius` and of
    /// `SEGMENTED.segment_radius`, both `Circular`.
    pub segment_shape: Shape,
    /// Tengen: `Role(ControlBackground)`, behind the segments.
    pub background: StateValue<ShapeStyle>,
    /// Tengen, with `selected`: `Clear` for a segment that is not chosen,
    /// `Role(SelectedControl)` for the chosen one.
    pub segment: StateValue<ShapeStyle>,
    /// Tengen, with `segment`: `Clear` for a segment that is not chosen,
    /// `Role(SelectedControl)` for the chosen one.
    pub selected: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub selected_label: StateValue<ShapeStyle>,
    /// Tengen: `None`, around the group.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: `[KNOB_SHADOW]`.
    pub selected_shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `Horizontal`: the segments share the width; `Vertical` stacks them at full
    /// width.
    pub axis: Axis,
    /// Tengen: `BODY` with `SEGMENTED.font_size` and `SEGMENTED.font_weight`; `line_height` has
    /// no effect.
    pub font: Font,
}

impl ControlStyle for SegmentedControlStyle {
    fn tengen() -> Self {
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
}

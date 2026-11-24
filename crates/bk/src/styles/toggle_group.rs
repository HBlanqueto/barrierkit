use ctdesign::{BODY, Font, ICON_BUTTON, LARGE_TITLE, SEGMENTED};

use crate::styles::button::continuous_rounded;
use crate::styles::{
    Axis, ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    system_effect,
};

/// States: each toggle is hovered, pressed and focused on its own; the group has
/// only `normal`.
#[derive(Clone, Debug)]
pub struct ToggleGroupStyle {
    /// Tengen: `SEGMENTED.padding`.
    pub padding: f32,
    /// Tengen: `SEGMENTED.gap`.
    pub gap: f32,
    /// Tengen: `(ICON_BUTTON.size, SEGMENTED.segment_height)`.
    pub item_size: (f32, f32),
    pub shape: Shape,
    pub item_shape: Shape,
    /// Tengen: `Role(ControlBackground)`, behind the items.
    pub background: StateValue<ShapeStyle>,
    pub item: StateValue<ShapeStyle>,
    pub selected: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Role(OnAccent)`.
    pub selected_label: StateValue<ShapeStyle>,
    /// Tengen: `None`, around the group.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub selected_shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `Horizontal`; `Vertical` stacks the toggles.
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
            shape: continuous_rounded(SEGMENTED.radius),
            item_shape: continuous_rounded(SEGMENTED.segment_radius),
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

use ctdesign::{BODY, Font, ICON_BUTTON, LARGE_TITLE, SEGMENTED};

use super::button::rounded;
use super::{
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
    /// Tengen, with `item_shape`: `RoundedRectangle` of `SEGMENTED.radius` and of
    /// `SEGMENTED.segment_radius`, both `Circular`.
    pub shape: Shape,
    /// Tengen, with `shape`: `RoundedRectangle` of `SEGMENTED.radius` and of
    /// `SEGMENTED.segment_radius`, both `Circular`.
    pub item_shape: Shape,
    /// Tengen: `Role(ControlBackground)`, behind the items.
    pub background: StateValue<ShapeStyle>,
    /// Tengen, with `selected`: `Clear` for a toggle that is off, `Accent` for one that is on.
    pub item: StateValue<ShapeStyle>,
    /// Tengen, with `item`: `Clear` for a toggle that is off, `Accent` for one that is on.
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
    /// Tengen, with `bold_font`: `BODY` with `SEGMENTED.font_weight`; `BODY` with
    /// `LARGE_TITLE.weight` for `Face::Bold`; `line_height` has no effect.
    pub font: Font,
    /// Tengen, with `font`: `BODY` with `SEGMENTED.font_weight`; `BODY` with
    /// `LARGE_TITLE.weight` for `Face::Bold`; `line_height` has no effect.
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

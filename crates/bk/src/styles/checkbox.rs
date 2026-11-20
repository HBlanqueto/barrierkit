use ctdesign::{BODY, CHECKBOX, Font, HAIRLINE, SETTINGS};

use super::button::rounded;
use super::{
    ColorEffect, ControlStyle, Edge, LayoutDirection, Role, ShadowStyle, Shape, ShapeStyle,
    StateValue, system_effect,
};

/// States: hovered, pressed, focused, disabled. `Checkbox::tint(Tint)` is an
/// instance override of `fill`.
///
/// The label before the control, as in a settings list:
#[derive(Clone, Debug)]
pub struct CheckboxStyle {
    /// Tengen: `CHECKBOX.size`.
    pub box_size: f32,
    /// Tengen: `CHECKBOX.mark`.
    pub mark_size: f32,
    /// Tengen: `SETTINGS.check.gap`.
    pub label_gap: f32,
    /// Tengen: `RoundedRectangle { radius: CHECKBOX.radius, style: Circular }`.
    pub box_shape: Shape,
    /// Tengen, with `fill`: `Role(SystemFill)` while unchecked, `Role(ToggleAccent)` while checked.
    pub box_background: StateValue<ShapeStyle>,
    /// Tengen, with `box_background`: `Role(SystemFill)` while unchecked, `Accent` while
    /// checked.
    pub fill: StateValue<ShapeStyle>,
    /// Tengen, with `label`: `Role(OnAccent)`, `Role(Label)` (disabled `Role(TertiaryLabel)`).
    pub mark: StateValue<ShapeStyle>,
    /// Tengen, with `mark`: `Role(OnAccent)`, `Role(Label)` (disabled `Role(TertiaryLabel)`).
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(Outline), HAIRLINE))`, drawn only while unchecked.
    pub box_border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub box_shadows: Vec<ShadowStyle>,
    /// Tengen, with `disabled_opacity`: `1.0`, `1.0`, for the box and the label.
    pub opacity: f32,
    /// Tengen, with `opacity`: `1.0`, `1.0`, for the box and the label.
    pub disabled_opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `Some(Edge::Trailing)`; `None` hides the label, which stays accessible.
    pub label_position: Option<Edge>,
    /// Tengen: `LeftToRight`; `RightToLeft` swaps `Leading` and `Trailing`.
    pub layout_direction: LayoutDirection,
    /// Tengen: `BODY` with `CHECKBOX.size` as `line_height`, so the row is as high as the box.
    pub label_font: Font,
}

impl ControlStyle for CheckboxStyle {
    fn tengen() -> Self {
        Self {
            box_size: CHECKBOX.size,
            mark_size: CHECKBOX.mark,
            label_gap: SETTINGS.check.gap as f32,
            box_shape: rounded(CHECKBOX.radius),
            box_background: StateValue::new(ShapeStyle::Role(Role::SystemFill)),
            fill: StateValue::new(ShapeStyle::Role(Role::ToggleAccent)),
            mark: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            label: StateValue::new(ShapeStyle::Role(Role::Label))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            box_border: Some((ShapeStyle::Role(Role::Outline), HAIRLINE)),
            box_shadows: Vec::new(),
            opacity: 1.0,
            disabled_opacity: 1.0,
            effect: system_effect,
            label_position: Some(Edge::Trailing),
            layout_direction: LayoutDirection::LeftToRight,
            label_font: Font {
                line_height: CHECKBOX.size as u32,
                ..BODY
            },
        }
    }
}

use ctdesign::{BODY, CHECKBOX, ControlSize, Font};

use super::button::continuous_rounded;
use super::{
    ColorEffect, ControlStyle, Edge, LayoutDirection, Role, ShadowStyle, Shape, ShapeStyle,
    StateValue, disabled_palette, pressed_palette, system_effect,
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
    /// Tengen: `CHECKBOX.mark_stroke`.
    pub mark_stroke: f32,
    /// Tengen: `(CHECKBOX.mixed_mark, CHECKBOX.mixed_stroke)`: the dash of a mixed checkbox.
    pub mixed_mark: (f32, f32),
    /// Tengen: `CHECKBOX.gap`.
    pub label_gap: f32,
    /// Tengen: `RoundedRectangle { radius: CHECKBOX.radius, style: Continuous }`.
    pub box_shape: Shape,
    /// Tengen, with `fill`: `Role(SystemFill)` while unchecked, `Role(ToggleAccent)` while checked;
    /// disabled `Color(DisabledPalette.off)` and `Color(DisabledPalette.on)`.
    pub box_background: StateValue<ShapeStyle>,
    /// Tengen, with `box_background`: `Role(SystemFill)` while unchecked, `Accent` while
    /// checked.
    pub fill: StateValue<ShapeStyle>,
    /// Tengen, with `label`: `Role(OnAccent)` (disabled `Role(TertiaryLabel)`), `Role(Label)` (disabled
    /// `Role(TertiaryLabel)`).
    pub mark: StateValue<ShapeStyle>,
    /// Tengen, with `mark`: `Role(OnAccent)`, `Role(Label)` (disabled `Role(TertiaryLabel)`).
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `None`; `Some` draws it only while unchecked.
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

impl CheckboxStyle {
    pub(crate) fn sized(self, size: ControlSize) -> Self {
        let row = CHECKBOX.size(size);
        Self {
            box_size: row.size,
            mark_size: row.mark,
            mark_stroke: row.mark_stroke,
            mixed_mark: (row.mixed_mark, row.mixed_stroke),
            label_gap: row.gap,
            box_shape: continuous_rounded(row.radius),
            label_font: Font {
                size: row.font_size as u32,
                line_height: row.size as u32,
                ..self.label_font
            },
            ..self
        }
    }
}

impl ControlStyle for CheckboxStyle {
    fn tengen() -> Self {
        Self {
            box_size: CHECKBOX.size,
            mark_size: CHECKBOX.mark,
            mark_stroke: CHECKBOX.mark_stroke,
            mixed_mark: (CHECKBOX.mixed_mark, CHECKBOX.mixed_stroke),
            label_gap: CHECKBOX.gap,
            box_shape: continuous_rounded(CHECKBOX.radius),
            box_background: StateValue::new(ShapeStyle::Role(Role::SystemFill))
                .pressed(ShapeStyle::Color(pressed_palette().off))
                .disabled(ShapeStyle::Color(disabled_palette().off)),
            fill: StateValue::new(ShapeStyle::Role(Role::ToggleAccent))
                .pressed(ShapeStyle::Color(pressed_palette().toggle_accent))
                .disabled(ShapeStyle::Color(disabled_palette().on)),
            mark: StateValue::new(ShapeStyle::Role(Role::OnAccent))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            label: StateValue::new(ShapeStyle::Role(Role::Label))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            box_border: None,
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

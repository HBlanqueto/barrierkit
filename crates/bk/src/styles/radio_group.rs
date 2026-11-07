use ctdesign::{BODY, Font, HAIRLINE, SETTINGS};

use super::{
    Axis, ColorEffect, ControlStyle, Edge, LayoutDirection, Role, ShadowStyle, Shape, ShapeStyle,
    StateValue, system_effect,
};

/// States: each option is hovered, pressed, focused and disabled on its own.
/// `RadioGroup::tint(Tint)` is an instance override of `fill`. The arrow keys
/// move the choice on both axes.
#[derive(Clone, Debug)]
pub struct RadioGroupStyle {
    /// Tengen: `SETTINGS.radio_size`.
    pub button_size: f32,
    /// Tengen: `SETTINGS.radio_dot`.
    pub dot_size: f32,
    /// Tengen, with `spacing`: `SETTINGS.radio_gap` between a button and its label,
    /// `SETTINGS.radio_spacing` between options.
    pub label_gap: f32,
    /// Tengen, with `label_gap`: `SETTINGS.radio_gap` between a button and its label,
    /// `SETTINGS.radio_spacing` between options.
    pub spacing: f32,
    /// Tengen: `Circle`.
    pub button_shape: Shape,
    /// Tengen: `Circle`.
    pub dot_shape: Shape,
    /// Tengen, with `fill`: `Role(ControlBackground)` while not chosen, `Accent` while chosen.
    pub button_background: StateValue<ShapeStyle>,
    /// Tengen, with `button_background`: `Role(ControlBackground)` while not chosen, `Accent`
    /// while chosen.
    pub fill: StateValue<ShapeStyle>,
    /// Tengen, with `label`: `Role(OnAccent)`, `Role(Label)` (disabled `Role(TertiaryLabel)`).
    pub dot: StateValue<ShapeStyle>,
    /// Tengen, with `dot`: `Role(OnAccent)`, `Role(Label)` (disabled `Role(TertiaryLabel)`).
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(Separator), HAIRLINE))`, drawn only while not chosen.
    pub button_border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty.
    pub button_shadows: Vec<ShadowStyle>,
    /// Tengen, with `disabled_opacity`: `1.0`, `1.0`, for the button and the label.
    pub opacity: f32,
    /// Tengen, with `opacity`: `1.0`, `1.0`, for the button and the label.
    pub disabled_opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `Vertical`: a column of options; `Horizontal` puts them in a row.
    pub axis: Axis,
    /// Tengen: `Some(Edge::Trailing)`; `None` hides the labels, which stay accessible.
    pub label_position: Option<Edge>,
    /// Tengen: `LeftToRight`; `RightToLeft` swaps `Leading` and `Trailing`.
    pub layout_direction: LayoutDirection,
    /// Tengen: `BODY`.
    pub label_font: Font,
}

impl ControlStyle for RadioGroupStyle {
    fn tengen() -> Self {
        Self {
            button_size: SETTINGS.radio_size as f32,
            dot_size: SETTINGS.radio_dot as f32,
            label_gap: SETTINGS.radio_gap as f32,
            spacing: SETTINGS.radio_spacing as f32,
            button_shape: Shape::Circle,
            dot_shape: Shape::Circle,
            button_background: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            fill: StateValue::new(ShapeStyle::Accent),
            dot: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            label: StateValue::new(ShapeStyle::Role(Role::Label))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            button_border: Some((ShapeStyle::Role(Role::Separator), HAIRLINE)),
            button_shadows: Vec::new(),
            opacity: 1.0,
            disabled_opacity: 1.0,
            effect: system_effect,
            axis: Axis::Vertical,
            label_position: Some(Edge::Trailing),
            layout_direction: LayoutDirection::LeftToRight,
            label_font: BODY,
        }
    }
}

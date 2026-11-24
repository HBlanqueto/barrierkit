use ctdesign::{BODY, CHECKBOX, ControlSize, Font, SETTINGS};

use super::{
    Axis, ColorEffect, ControlStyle, Edge, Role, ShadowStyle, Shape, ShapeStyle, StateValue,
    disabled_palette, pressed_palette, system_effect,
};

/// States: each option is hovered, pressed, focused and disabled on its own.
/// `RadioGroup::tint(Tint)` is an instance override of `fill`. The arrow keys
/// move the choice on both axes.
#[derive(Clone, Debug)]
pub struct RadioGroupStyle {
    /// Tengen: `CHECKBOX.size`.
    pub button_size: f32,
    /// Tengen: `CHECKBOX.radio_dot`.
    pub dot_size: f32,
    /// Tengen, with `spacing`: `CHECKBOX.gap` between a button and its label,
    /// `SETTINGS.radio_spacing` between options.
    pub label_gap: f32,
    /// Tengen, with `label_gap`: `CHECKBOX.gap` between a button and its label,
    /// `SETTINGS.radio_spacing` between options.
    pub spacing: f32,
    /// Tengen: `Circle`.
    pub button_shape: Shape,
    /// Tengen: `Circle`.
    pub dot_shape: Shape,
    /// Tengen, with `fill`: `Role(SystemFill)` while not chosen, `Role(ToggleAccent)` while chosen.
    pub button_background: StateValue<ShapeStyle>,
    /// Tengen, with `button_background`: `Role(SystemFill)` while not chosen, `Accent`
    /// while chosen.
    pub fill: StateValue<ShapeStyle>,
    /// Tengen, with `label`: `Role(OnAccent)`, `Role(Label)` (disabled `Role(TertiaryLabel)`).
    pub dot: StateValue<ShapeStyle>,
    /// Tengen, with `dot`: `Role(OnAccent)`, `Role(Label)` (disabled `Role(TertiaryLabel)`).
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `None`; `Some` draws it only while not chosen.
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
    /// Tengen: `BODY` with `CHECKBOX.size` as `line_height`, so the row is as high as the button.
    pub label_font: Font,
}

impl RadioGroupStyle {
    pub(crate) fn sized(self, size: ControlSize) -> Self {
        let row = CHECKBOX.size(size);
        Self {
            button_size: row.size,
            dot_size: row.radio_dot,
            label_gap: row.gap,
            label_font: Font {
                size: row.font_size as u32,
                line_height: row.size as u32,
                ..self.label_font
            },
            ..self
        }
    }
}

impl ControlStyle for RadioGroupStyle {
    fn tengen() -> Self {
        Self {
            button_size: CHECKBOX.size,
            dot_size: CHECKBOX.radio_dot,
            label_gap: CHECKBOX.gap,
            spacing: SETTINGS.radio_spacing as f32,
            button_shape: Shape::Circle,
            dot_shape: Shape::Circle,
            button_background: StateValue::new(ShapeStyle::Role(Role::SystemFill))
                .pressed(ShapeStyle::Color(pressed_palette().off))
                .disabled(ShapeStyle::Color(disabled_palette().off)),
            fill: StateValue::new(ShapeStyle::Role(Role::ToggleAccent))
                .pressed(ShapeStyle::Color(pressed_palette().toggle_accent))
                .disabled(ShapeStyle::Color(disabled_palette().on)),
            dot: StateValue::new(ShapeStyle::Role(Role::OnAccent))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            label: StateValue::new(ShapeStyle::Role(Role::Label))
                .disabled(ShapeStyle::Role(Role::TertiaryLabel)),
            button_border: None,
            button_shadows: Vec::new(),
            opacity: 1.0,
            disabled_opacity: 1.0,
            effect: system_effect,
            axis: Axis::Vertical,
            label_position: Some(Edge::Trailing),
            label_font: Font {
                line_height: CHECKBOX.size as u32,
                ..BODY
            },
        }
    }
}

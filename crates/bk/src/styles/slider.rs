use ctappearance::Resolved;
use ctdesign::{CAPTION, Font, KNOB_SHADOW, SETTINGS, SLIDER, SWITCH};

use super::{
    Animation, Axis, ColorEffect, ControlStyle, Edge, LayoutDirection, Role, ShadowStyle, Shape,
    ShapeStyle, StateValue, reduce_motion, system_effect,
};

/// On a horizontal slider `Leading` is `Top` and `Trailing` is `Bottom`; on a
/// vertical one `Top` is `Leading` and `Bottom` is `Trailing`. The arrow keys
/// follow the axis: ↑ always adds one step and ↓ removes one.
#[derive(Clone, Debug)]
pub struct SliderStyle {
    /// Tengen: `SETTINGS.strength.width`, the length along the axis.
    pub width: f32,
    /// Tengen: `SETTINGS.strength.track`.
    pub track_height: f32,
    /// Tengen: `(SLIDER.knob_width, SLIDER.knob_height)`: along the axis, across it.
    pub knob_size: (f32, f32),
    /// Tengen, with `tick_gap`, `label_gap`: `SETTINGS.strength.{tick, tick_gap, label_gap}`.
    pub tick_size: f32,
    /// Tengen, with `tick_size`, `label_gap`: `SETTINGS.strength.{tick, tick_gap, label_gap}`.
    pub tick_gap: f32,
    /// Tengen, with `tick_size`, `tick_gap`: `SETTINGS.strength.{tick, tick_gap, label_gap}`.
    pub label_gap: f32,
    /// Tengen: `Capsule`; the fill takes the same shape.
    pub track_shape: Shape,
    /// Tengen: `Capsule`.
    pub knob_shape: Shape,
    /// Tengen: `Circle`.
    pub tick_shape: Shape,
    /// Tengen: `Role(ControlBackground)`.
    pub track: StateValue<ShapeStyle>,
    /// Tengen: `Accent`.
    pub fill: StateValue<ShapeStyle>,
    /// Tengen: `Color(SWITCH.knob_color)`.
    pub knob: StateValue<ShapeStyle>,
    /// Tengen: `Role(ControlBorder)`.
    pub tick: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `None`; paint and width, drawn inside the knob in one color.
    pub knob_border: Option<(ShapeStyle, f32)>,
    /// Tengen: `[KNOB_SHADOW]`.
    pub knob_shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`, for every part.
    pub opacity: f32,
    /// Tengen: `SWITCH.disabled_opacity`, for the track, ticks and labels.
    pub disabled_opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `Horizontal`; `Vertical` grows upwards.
    pub axis: Axis,
    /// Tengen: `Some(Edge::Bottom)`; the ticks sit on the same side, below when `None` hides
    /// the labels and their space.
    pub labels: Option<Edge>,
    /// Tengen: `LeftToRight`; `RightToLeft` grows to the left and swaps ← →.
    pub layout_direction: LayoutDirection,
    /// Tengen: `CAPTION`.
    pub label_font: Font,
    /// Tengen: `None`: the knob jumps to the new value, as in Tengen; set `Ease { duration_ms:
    /// DURATION_FAST_MS }` or a `Spring` to animate the knob and the fill.
    pub animation: Animation,
}

impl ControlStyle for SliderStyle {
    fn tengen() -> Self {
        let strength = SETTINGS.strength;
        Self {
            width: strength.width as f32,
            track_height: strength.track as f32,
            knob_size: (SLIDER.knob_width, SLIDER.knob_height),
            tick_size: strength.tick as f32,
            tick_gap: strength.tick_gap as f32,
            label_gap: strength.label_gap as f32,
            track_shape: Shape::Capsule,
            knob_shape: Shape::Capsule,
            tick_shape: Shape::Circle,
            track: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            fill: StateValue::new(ShapeStyle::Accent),
            knob: StateValue::new(ShapeStyle::Color(SWITCH.knob_color)),
            tick: StateValue::new(ShapeStyle::Role(Role::ControlBorder)),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            knob_border: None,
            knob_shadows: vec![KNOB_SHADOW.into()],
            opacity: 1.0,
            disabled_opacity: SWITCH.disabled_opacity,
            effect: system_effect,
            axis: Axis::Horizontal,
            labels: Some(Edge::Bottom),
            layout_direction: LayoutDirection::LeftToRight,
            label_font: CAPTION,
            animation: Animation::None,
        }
    }

    fn accessible(self, appearance: &Resolved) -> Self {
        reduce_motion(self, appearance, |style| Self {
            animation: Animation::None,
            ..style
        })
    }
}

#[cfg(test)]
mod tests {
    use ctappearance::{AppearanceConfig, resolve};

    use super::*;

    #[test]
    fn reduced_motion_removes_any_animation_an_override_sets() {
        let animated = SliderStyle {
            animation: Animation::Spring {
                duration_ms: 400,
                bounce: 0.5,
            },
            ..SliderStyle::tengen()
        };
        let reduced = AppearanceConfig {
            reduced_motion: true,
            ..Default::default()
        };
        let style = animated.accessible(&resolve(&reduced, None));
        assert!(matches!(style.animation, Animation::None));
    }
}

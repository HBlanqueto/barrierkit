use ctappearance::Resolved;
use ctdesign::{CAPTION, Font, KNOB_SHADOW, SETTINGS, SLIDER, SWITCH};

use super::{
    Animation, Axis, ColorEffect, ControlStyle, Edge, LayoutDirection, Role, ShadowStyle, Shape,
    ShapeStyle, StateValue, reduce_motion, system_effect,
};

#[derive(Clone, Debug)]
pub struct SliderStyle {
    pub width: f32,
    pub track_height: f32,
    pub knob_size: (f32, f32),
    pub tick_size: f32,
    pub tick_gap: f32,
    pub label_gap: f32,
    pub track_shape: Shape,
    pub knob_shape: Shape,
    pub tick_shape: Shape,
    pub track: StateValue<ShapeStyle>,
    pub fill: StateValue<ShapeStyle>,
    pub knob: StateValue<ShapeStyle>,
    pub tick: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub knob_border: Option<(ShapeStyle, f32)>,
    pub knob_shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub disabled_opacity: f32,
    pub effect: ColorEffect,
    pub axis: Axis,
    pub labels: Option<Edge>,
    pub layout_direction: LayoutDirection,
    pub label_font: Font,
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

use ctappearance::Resolved;
use ctdesign::{BODY, Font, HAIRLINE, KNOB_SHADOW, SETTINGS, SWITCH};

use super::{
    Animation, ColorEffect, ControlStyle, Edge, LayoutDirection, Role, ShadowStyle, Shape,
    ShapeStyle, StateValue, reduce_motion, system_effect,
};

#[derive(Clone, Debug)]
pub struct SwitchStyle {
    pub width: f32,
    pub height: f32,
    pub knob_size: (f32, f32),
    pub inset: f32,
    pub label_gap: f32,
    pub track_shape: Shape,
    pub knob_shape: Shape,
    pub track: StateValue<ShapeStyle>,
    pub fill: StateValue<ShapeStyle>,
    pub knob: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub track_border: Option<(ShapeStyle, f32)>,
    pub knob_border: Option<(ShapeStyle, f32)>,
    pub knob_shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub disabled_opacity: f32,
    pub effect: ColorEffect,
    pub label_position: Option<Edge>,
    pub layout_direction: LayoutDirection,
    pub label_font: Font,
    pub animation: Animation,
}

impl ControlStyle for SwitchStyle {
    fn tengen() -> Self {
        Self {
            width: SWITCH.width,
            height: SWITCH.height,
            knob_size: (SWITCH.knob_width, SWITCH.knob_height),
            inset: SWITCH.inset,
            label_gap: SETTINGS.check.gap as f32,
            track_shape: Shape::Capsule,
            knob_shape: Shape::Capsule,
            track: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            fill: StateValue::new(ShapeStyle::Accent),
            knob: StateValue::new(ShapeStyle::Color(SWITCH.knob_color)),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            track_border: Some((ShapeStyle::Role(Role::Separator), HAIRLINE)),
            knob_border: None,
            knob_shadows: vec![KNOB_SHADOW.into()],
            opacity: 1.0,
            disabled_opacity: SWITCH.disabled_opacity,
            effect: system_effect,
            label_position: None,
            layout_direction: LayoutDirection::LeftToRight,
            label_font: BODY,
            animation: Animation::Ease {
                duration_ms: SWITCH.transition_ms,
            },
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
    fn reduced_motion_makes_the_knob_jump() {
        let reduced = AppearanceConfig {
            reduced_motion: true,
            ..Default::default()
        };
        let style = SwitchStyle::tengen().accessible(&resolve(&reduced, None));
        assert!(matches!(style.animation, Animation::None));
    }
}

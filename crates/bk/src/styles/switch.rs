use ctappearance::Resolved;
use ctdesign::{BODY, Font, HAIRLINE, KNOB_SHADOW, SETTINGS, SWITCH};

use super::{
    Animation, ColorEffect, ControlStyle, Edge, LayoutDirection, Role, ShadowStyle, Shape,
    ShapeStyle, StateValue, reduce_motion, system_effect,
};

/// States: hovered, pressed, focused, disabled. `Switch::tint(Tint)` is an
/// instance override of `fill`.
#[derive(Clone, Debug)]
pub struct SwitchStyle {
    /// Tengen: `SWITCH.width`.
    pub width: f32,
    /// Tengen: `SWITCH.height`.
    pub height: f32,
    /// Tengen: `(SWITCH.knob_width, SWITCH.knob_height)`.
    pub knob_size: (f32, f32),
    /// Tengen: `SWITCH.inset`: the knob's gap to the ends; it travels `width` − knob width − 2
    /// × `inset`.
    pub inset: f32,
    /// Tengen: `SETTINGS.check.gap`.
    pub label_gap: f32,
    /// Tengen: `Capsule`.
    pub track_shape: Shape,
    /// Tengen: `Capsule`.
    pub knob_shape: Shape,
    /// Tengen, with `fill`: `Role(ControlBackground)` while off, `Accent` while on.
    pub track: StateValue<ShapeStyle>,
    /// Tengen, with `track`: `Role(ControlBackground)` while off, `Accent` while on.
    pub fill: StateValue<ShapeStyle>,
    /// Tengen: `Color(SWITCH.knob_color)`.
    pub knob: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(Separator), HAIRLINE))`, drawn only while off.
    pub track_border: Option<(ShapeStyle, f32)>,
    /// Tengen: `None`.
    pub knob_border: Option<(ShapeStyle, f32)>,
    /// Tengen: `[KNOB_SHADOW]`.
    pub knob_shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `SWITCH.disabled_opacity`, for the switch and its label.
    pub disabled_opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `None`: the label is only accessible, as in a form row
    /// (`docs/barrierkit/11-views.md`); `Leading` and `Trailing` put it beside the switch,
    /// `Top` and `Bottom` above and below.
    pub label_position: Option<Edge>,
    /// Tengen: `LeftToRight`; `RightToLeft` puts the knob of a switch that is on at the left
    /// and swaps `Leading` and `Trailing`.
    pub layout_direction: LayoutDirection,
    /// Tengen: `BODY`.
    pub label_font: Font,
    /// Tengen: `Ease { duration_ms: SWITCH.transition_ms }`: the knob slides; `None` makes it
    /// jump.
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

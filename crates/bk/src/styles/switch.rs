use ctappearance::Resolved;
use ctdesign::{BODY, Font, KNOB_SHADOW, Rgba, SETTINGS, SWITCH};

use super::{
    Animation, ColorEffect, ControlStyle, Edge, LayoutDirection, Role, ShadowStyle, Shape,
    ShapeStyle, StateValue, disabled_palette, reduce_motion, system_effect,
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
    /// Tengen, with `fill`: `Role(SystemFill)` while off, `Role(ToggleAccent)` while on.
    pub track: StateValue<ShapeStyle>,
    /// Tengen, with `track`: `Role(SystemFill)` while off, `Role(ToggleAccent)` while on and
    /// `Role(ToggleAccentDisabled)` while it is disabled; under a drag the track is off only
    /// while the knob is at the off end.
    pub fill: StateValue<ShapeStyle>,
    /// Tengen: `Color(SWITCH.knob_color)`.
    pub knob: StateValue<ShapeStyle>,
    /// Tengen: `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `None`; `Some` draws it only while off.
    pub track_border: Option<(ShapeStyle, f32)>,
    /// Tengen: `None`.
    pub knob_border: Option<(ShapeStyle, f32)>,
    /// Tengen: `[KNOB_SHADOW]`; dropped while the compositor draws the held knob.
    pub knob_shadows: Vec<ShadowStyle>,
    /// Tengen: `SWITCH.knob_pressed_scale`: how much the knob grows while it is held; it ends
    /// as wide as the track and taller than it.
    pub knob_pressed_scale: f32,
    /// Tengen: `Some(Color(SWITCH.knob_color at SWITCH.knob_glass_alpha))`: while held, the
    /// compositor draws the knob as Liquid Glass (`set_knobs`) with its own lens: the paint is the
    /// tint with its own alpha. Without compositor support the knob stays opaque; `None` keeps the opaque
    /// knob.
    pub knob_glass: Option<ShapeStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `SWITCH.disabled_opacity` (`SWITCH.disabled_opacity_contrast` under Increase
    /// Contrast), for the track and the label; the knob stays opaque.
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
    /// Tengen: `Ease { duration_ms: SWITCH.transition_ms }`: the knob slides to an end when it
    /// is released; under a drag it follows the cursor. `None` makes it jump.
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
            track: StateValue::new(ShapeStyle::Role(Role::SystemFill))
                .disabled(ShapeStyle::Color(disabled_palette().switch_off)),
            fill: StateValue::new(ShapeStyle::Role(Role::ToggleAccent))
                .disabled(ShapeStyle::Role(Role::ToggleAccentDisabled)),
            knob: StateValue::new(ShapeStyle::Color(SWITCH.knob_color)),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            track_border: None,
            knob_border: None,
            knob_shadows: vec![KNOB_SHADOW.into()],
            knob_pressed_scale: SWITCH.knob_pressed_scale,
            knob_glass: Some(ShapeStyle::Color(Rgba {
                a: SWITCH.knob_glass_alpha,
                ..SWITCH.knob_color
            })),
            opacity: 1.0,
            disabled_opacity: if crate::theme::resolved().contrast {
                SWITCH.disabled_opacity_contrast
            } else {
                SWITCH.disabled_opacity
            },
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
    fn the_tengen_knob_is_handed_to_the_compositor_without_the_app_asking() {
        assert!(SwitchStyle::tengen().knob_glass.is_some());
    }

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

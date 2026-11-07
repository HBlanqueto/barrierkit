use std::rc::Rc;

use ctdesign::{CAPTION, Font, KNOB_SHADOW, SETTINGS, SLIDER, SWITCH};

use super::{
    Animation, Axis, ColorEffect, Edge, LayoutDirection, Role, ShadowStyle, Shape, ShapeStyle,
    StateValue, Theme, system_effect,
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

impl SliderStyle {
    pub fn tengen() -> Self {
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

    pub(crate) fn resolve(theme: &Theme, instance: &SliderOverride, reduced_motion: bool) -> Self {
        let style = instance.apply(theme.slider.apply(Self::tengen()));
        if reduced_motion {
            Self {
                animation: Animation::None,
                ..style
            }
        } else {
            style
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct SliderOverride(Option<Rc<dyn Fn(SliderStyle) -> SliderStyle>>);

impl SliderOverride {
    pub(crate) fn then(self, next: impl Fn(SliderStyle) -> SliderStyle + 'static) -> Self {
        Self(Some(Rc::new(move |style| next(self.apply(style)))))
    }

    pub(crate) fn apply(&self, style: SliderStyle) -> SliderStyle {
        match &self.0 {
            Some(restyle) => restyle(style),
            None => style,
        }
    }
}

impl PartialEq for SliderOverride {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Some(own), Some(other)) => Rc::ptr_eq(own, other),
            (own, other) => own.is_none() && other.is_none(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doubled_track(style: SliderStyle) -> SliderStyle {
        SliderStyle {
            track_height: style.track_height * 2.0,
            ..style
        }
    }

    #[test]
    fn theme_then_containers_outermost_first_then_instance_each_see_the_previous_level() {
        let theme = Theme::tengen().slider(|s| SliderStyle { width: 100.0, ..s });
        let outer = Theme::tengen().slider(doubled_track);
        let inner = Theme::tengen().slider(|s| SliderStyle {
            track_height: s.track_height + 1.0,
            ..s
        });
        let instance = SliderOverride::default().then(|s| SliderStyle {
            track_height: s.track_height * 10.0,
            ..s
        });
        let style = SliderStyle::resolve(&theme.within(outer).within(inner), &instance, false);
        let tengen = SliderStyle::tengen();
        assert_eq!(style.width, 100.0);
        assert_eq!(style.track_height, (tengen.track_height * 2.0 + 1.0) * 10.0);
        assert_eq!(style.knob_size, tengen.knob_size);
    }

    #[test]
    fn reduced_motion_removes_any_animation_an_override_sets() {
        let instance = SliderOverride::default().then(|s| SliderStyle {
            animation: Animation::Spring {
                duration_ms: 400,
                bounce: 0.5,
            },
            ..s
        });
        let style = SliderStyle::resolve(&Theme::tengen(), &instance, true);
        assert!(matches!(style.animation, Animation::None));
    }
}

use ctdesign::SLIDER_KEY_STEP;
use std::borrow::Cow;

use super::*;
use crate::controlled::{commit, use_controlled_unless};
use crate::level;
use crate::styles::CircularSliderStyle;

#[derive(PartialEq)]
pub struct CircularSlider {
    label: Cow<'static, str>,
    value: f32,
    disabled: bool,
    on_change: Option<Callback<f32, ()>>,
    style: Theme,
    key: DiffKey,
    overrides: Overrides,
}

impl CircularSlider {
    pub fn new(label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            label: label.into(),
            value: 0.0,
            disabled: false,
            on_change: None,
            style: Theme::tengen(),
            key: DiffKey::None,
            overrides: Overrides::default(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(CircularSliderStyle) -> CircularSliderStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = value.clamp(0.0, 1.0);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, callback: impl Into<Callback<f32, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for CircularSlider {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

fn turn_value(from_center_x: f32, from_center_y: f32) -> f32 {
    from_center_x
        .atan2(-from_center_y)
        .to_degrees()
        .rem_euclid(360.0)
        / 360.0
}

forward_overrides!(CircularSlider);

impl Component for CircularSlider {
    fn render(&self) -> impl IntoElement {
        let interaction = use_interaction();
        let style: CircularSliderStyle = resolve(&self.style);
        let disabled = use_disabled(self.disabled);
        let state = control_state(interaction, disabled);
        let paints = Paints::current(state, style.effect);
        let value = use_controlled_unless(&self.value, interaction.held());
        let origin = use_state(CursorPoint::default);
        let on_change = self.on_change.clone();
        let current = *value.read();
        let size = style.size;
        let (from, to, width) = style.dash;
        let radius = style
            .shape
            .corner_radius(size, size, LayoutDirection::LeftToRight);
        let update = move |new_value: f32| commit(value, &on_change, new_value);
        let set_from_point = move |point: CursorPoint| {
            turn_value(point.x as f32 - size / 2.0, point.y as f32 - size / 2.0)
        };
        let mut root = interaction
            .attach(rect(), !disabled)
            .width(Size::px(size))
            .height(Size::px(size))
            .corner_radius(radius)
            .painted(&paints, &style.disc)
            .a11y_role(AccessibilityRole::Slider)
            .a11y_alt(&*self.label)
            .a11y_builder(move |node| {
                node.set_numeric_value(f64::from(current));
                node.set_min_numeric_value(0.0);
                node.set_max_numeric_value(1.0);
                node.set_numeric_value_step(f64::from(SLIDER_KEY_STEP));
            });
        if !disabled {
            let step = update.clone();
            root = level::on_drag(root, interaction, origin, set_from_point, update)
                .on_global_pointer_up(move |_| interaction.release())
                .on_key_down(move |event: Event<KeyboardEventData>| match event.key {
                    Key::Named(NamedKey::ArrowUp | NamedKey::ArrowRight) => {
                        step((current + SLIDER_KEY_STEP).min(1.0))
                    }
                    Key::Named(NamedKey::ArrowDown | NamedKey::ArrowLeft) => {
                        step((current - SLIDER_KEY_STEP).max(0.0))
                    }
                    _ => {}
                });
        }
        let dash = rect()
            .position(
                Position::new_absolute()
                    .left(size / 2.0 + from)
                    .top((size - width) / 2.0),
            )
            .width(Size::px(to - from))
            .height(Size::px(width))
            .corner_radius(width / 2.0)
            .background(paints.fill(&style.dash_ink));
        let turned = rect()
            .position(Position::new_absolute().left(0.0).top(0.0))
            .width(Size::px(size))
            .height(Size::px(size))
            .rotate((current - 0.25) * 360.0)
            .child(dash);
        let root =
            interaction.focus_ring(dimmed(root.child(turned), style.opacity), radius.top_left());
        self.overrides.apply(root)
    }
}

#[cfg(test)]
mod tests {
    use super::turn_value;

    #[test]
    fn the_value_is_the_turn_clockwise_from_the_top() {
        assert_eq!(turn_value(0.0, -1.0), 0.0);
        assert_eq!(turn_value(1.0, 0.0), 0.25);
        assert_eq!(turn_value(0.0, 1.0), 0.5);
        assert_eq!(turn_value(-1.0, 0.0), 0.75);
    }
}

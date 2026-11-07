use ctdesign::SLIDER_KEY_STEP;
use freya_core::prelude::*;
use torin::prelude::{CursorPoint, Position, Size};

use crate::backdrop::Tint;
use crate::controlled::commit;
use crate::controls::{dimmed, inner_border, shadowed};
use crate::interaction::{Interaction, use_interaction};
use crate::styles::{
    ControlState, LayoutDirection, LevelSliderStyle, Paints, Shape, ShapeStyle, StateValue, Theme,
    resolve,
};
use crate::symbol::symbol;

#[derive(PartialEq)]
pub struct LevelSlider {
    label: &'static str,
    value: f32,
    disabled: bool,
    on_change: Option<Callback<f32, ()>>,
    style: Theme,
    key: DiffKey,
}

impl LevelSlider {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            value: 0.0,
            disabled: false,
            on_change: None,
            style: Theme::tengen(),
            key: DiffKey::None,
        }
    }

    pub fn tint(self, tint: Tint) -> Self {
        self.style(move |style| LevelSliderStyle {
            fill: StateValue::new(tint.into()),
            ..style
        })
    }

    pub fn style(
        mut self,
        restyle: impl Fn(LevelSliderStyle) -> LevelSliderStyle + 'static,
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

impl KeyExt for LevelSlider {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for LevelSlider {
    fn render(&self) -> impl IntoElement {
        let interaction = use_interaction();
        let style: LevelSliderStyle = resolve(&self.style);
        let state = ControlState {
            hovered: interaction.hovered(),
            pressed: interaction.pressed(),
            focused: interaction.focused(),
            disabled: self.disabled,
        };
        let paints = Paints::current(state, style.effect);
        let value = crate::controlled::use_controlled(&self.value);
        let mut width = use_state(|| 0.0f32);
        let origin = use_state(CursorPoint::default);
        let enabled = !self.disabled;
        let fraction = *value.read();
        let measured = *width.read();
        let direction = style.layout_direction;
        let right_to_left = direction == LayoutDirection::RightToLeft;

        let height = style.height;
        let track = style.track_height;
        let knob_width = style.knob_size.map_or(0.0, |(width, _)| width);
        let offset = knob_width / 2.0;
        let run = (measured - knob_width).max(0.0);
        let on_change = self.on_change.clone();
        let set = move |new: f32| commit(value, &on_change, new.clamp(0.0, 1.0));
        let from_point = move |point: CursorPoint| {
            let along = ((point.x as f32 - offset) / run.max(1.0)).clamp(0.0, 1.0);
            if right_to_left { 1.0 - along } else { along }
        };
        let forward = if right_to_left {
            NamedKey::ArrowLeft
        } else {
            NamedKey::ArrowRight
        };

        let mut root = interaction
            .attach(rect(), enabled)
            .a11y_role(AccessibilityRole::Slider)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_numeric_value(f64::from(fraction)))
            .width(Size::fill())
            .height(Size::px(height))
            .on_sized(move |event: Event<SizedEventData>| {
                width.set_if_modified(event.data().area.size.width);
            })
            .maybe(enabled, |el| {
                on_drag(el, interaction, origin, from_point, set.clone()).on_key_down(
                    move |event: Event<KeyboardEventData>| {
                        let step = match event.key {
                            Key::Named(NamedKey::ArrowUp) => SLIDER_KEY_STEP,
                            Key::Named(key) if key == forward => SLIDER_KEY_STEP,
                            Key::Named(
                                NamedKey::ArrowLeft | NamedKey::ArrowRight | NamedKey::ArrowDown,
                            ) => -SLIDER_KEY_STEP,
                            _ => return,
                        };
                        set(fraction + step);
                    },
                )
            });

        let knob_x = run * crate::animation::use_animated(fraction, style.animation);
        let fill_width = (knob_x + offset).max(style.fill_min);
        let mirrored = |left: f32, size: f32, parent: f32| {
            if right_to_left {
                parent - left - size
            } else {
                left
            }
        };
        let part = |left: f32,
                    top: f32,
                    width: f32,
                    height: f32,
                    shape: Shape,
                    paint: &StateValue<ShapeStyle>| {
            rect()
                .position(Position::new_absolute().left(left).top(top))
                .width(Size::px(width))
                .height(Size::px(height))
                .corner_radius(shape.corner_radius(width, height, direction))
                .background(paints.fill(paint))
                .map(paints.blur(paint), Rect::blur)
        };
        let parts_opacity = if enabled {
            style.opacity
        } else {
            style.opacity * style.disabled_opacity
        };

        let fill = part(
            mirrored(-offset, fill_width, run),
            0.0,
            fill_width,
            track,
            style.track_shape,
            &style.fill,
        );
        let icon = style.symbol.map(|lead| {
            let icon_size = style.icon_size;
            rect()
                .position(
                    Position::new_absolute()
                        .left(mirrored(style.icon_inset, icon_size, measured) - offset)
                        .top((track - icon_size) / 2.0),
                )
                .child(symbol(
                    lead,
                    icon_size,
                    paints.color(style.icon.resolve(state)),
                ))
        });
        // freya 0.4.3 dims each element on its own (render_pipeline.rs): a dimmed knob would show the fill.
        root = root.child(dimmed(
            part(
                offset,
                (height - track) / 2.0,
                run,
                track,
                style.track_shape,
                &style.track,
            )
            .child(fill)
            .map(icon, |el, icon| el.child(icon)),
            parts_opacity,
        ));
        if let Some((knob_width, knob_height)) = style.knob_size {
            let knob = part(
                mirrored(knob_x, knob_width, measured),
                (height - knob_height) / 2.0,
                knob_width,
                knob_height,
                style.knob_shape,
                &style.knob,
            )
            .map(style.knob_border.as_ref(), |el, border| {
                el.border(inner_border(&paints, border))
            })
            .layer(KNOB_ABOVE_FILL);
            root = root.child(dimmed(shadowed(knob, &style.knob_shadows), style.opacity));
        }
        let radius = style.track_shape.corner_radius(height, height, direction);
        interaction.focus_ring(root, radius.top_left)
    }
}

pub(crate) fn on_drag<T: 'static>(
    root: Rect,
    interaction: Interaction,
    mut origin: State<CursorPoint>,
    from_point: impl Fn(CursorPoint) -> T + Copy + 'static,
    set: impl Fn(T) + Clone + 'static,
) -> Rect {
    let set_move = set.clone();
    root.on_mouse_down(move |event: Event<MouseEventData>| {
        origin.set((event.global_location - event.element_location).to_point());
        set(from_point(event.element_location));
    })
    .on_global_pointer_move(move |event: Event<PointerEventData>| {
        if interaction.held() {
            set_move(from_point(
                event.global_location() - origin.peek().to_vector(),
            ));
        }
    })
}

// freya 0.4.3 paints by depth (data.rs `LayerState`); a sibling knob must outrank the fill, two levels down.
pub(crate) const KNOB_ABOVE_FILL: i16 = 2;

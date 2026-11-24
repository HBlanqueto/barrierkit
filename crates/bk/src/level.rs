use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use ctdesign::{Rgba, SLIDER, SLIDER_KEY_STEP};
use freya_core::prelude::*;
use torin::prelude::{Area, CursorPoint, Position, Size};

use crate::backdrop::Tint;
use crate::controlled::commit;
use crate::controls::{dimmed, opacity_when, shadowed};
use crate::elements::{Overrides, forward_key, forward_overrides};
use crate::environment::use_disabled;
use crate::interaction::{Interaction, control_state, use_interaction};
use crate::layout::ambient;
use crate::platform::{Knob, KnobOutbox, KnobSupport, Knobs};
use crate::styles::Painted;
use crate::styles::{
    Animation, LayoutDirection, LevelSliderStyle, Paints, ShadowStyle, Shape, ShapeStyle,
    StateValue, Theme, resolve,
};
use crate::symbol::symbol;

#[derive(PartialEq)]
pub struct LevelSlider {
    label: Cow<'static, str>,
    value: State<f32>,
    disabled: bool,
    style: Theme,
    key: DiffKey,
    overrides: Overrides,
}

impl LevelSlider {
    pub fn new(label: impl Into<Cow<'static, str>>, value: State<f32>) -> Self {
        Self {
            label: label.into(),
            value,
            disabled: false,
            style: Theme::tengen(),
            key: DiffKey::None,
            overrides: Overrides::default(),
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

forward_key!(LevelSlider);

forward_overrides!(LevelSlider);

impl Component for LevelSlider {
    forward_key!();

    fn render(&self) -> impl IntoElement {
        let interaction = use_interaction();
        let style: LevelSliderStyle = resolve(&self.style);
        let disabled = use_disabled(self.disabled);
        let state = control_state(interaction, disabled);
        let paints = Paints::current(state);
        let value = self.value;
        let mut width = use_state(|| 0.0f32);
        let origin = use_state(CursorPoint::default);
        let enabled = !disabled;
        let fraction = value.read().clamp(0.0, 1.0);
        let measured = *width.read();
        let direction = ambient();
        let right_to_left = direction == LayoutDirection::RightToLeft;

        let height = style.height;
        let track = style.track_height;
        let knob_width = style.knob_size.map_or(0.0, |(width, _)| width);
        let offset = knob_width / 2.0;
        let run = (measured - knob_width).max(0.0);
        let set = move |new: f32| commit(value, new.clamp(0.0, 1.0));
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
            .a11y_alt(&*self.label)
            .a11y_builder(move |node| node.set_numeric_value(f64::from(fraction)))
            .width(Size::fill())
            .height(Size::px(height))
            .on_sized(move |event: Event<SizedEventData>| {
                width.set_if_modified(event.data().area.size.width);
            })
            .maybe(enabled, |el| {
                on_drag(el, interaction, origin, from_point, set).on_key_down(
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

        let animation = crate::animation::effective(style.animation, interaction.held());
        let knob_x = run * crate::animation::use_animated(fraction, animation);
        let knob_scale = use_pressed_scale(
            interaction.held(),
            style.knob_pressed_scale,
            style.animation,
        );
        let glass = style
            .knob_glass
            .as_ref()
            .filter(|_| interaction.held() && knobs_supported())
            .map(|paint| Rgba {
                a: SLIDER.knob_glass_alpha,
                ..paints.color(paint)
            });
        let knob_area = use_knob_glass(Knob::Slider, glass, knob_scale);
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
                .painted(&paints, paint)
        };
        let parts_opacity = opacity_when(disabled, style.opacity, style.disabled_opacity);

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
        // freya 0.5.3 dims each element on its own (render_pipeline.rs): a dimmed knob would show the fill.
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
            let left = mirrored(knob_x, knob_width, measured);
            let top = (height - knob_height) / 2.0;
            let body = match glass {
                Some(_) => glass_knob(knob_width, knob_height, knob_area)
                    .position(Position::new_absolute().left(left).top(top)),
                None => part(
                    left,
                    top,
                    knob_width,
                    knob_height,
                    style.knob_shape,
                    &style.knob,
                ),
            };
            let knob = body
                .bordered(&paints, style.knob_border.as_ref())
                .layer(KNOB_ABOVE_FILL)
                .maybe(knob_scale != 1.0, |el| el.scale(knob_scale));
            root = root.child(dimmed(shadowed(knob, &style.knob_shadows), style.opacity));
        }
        let radius = style.track_shape.corner_radius(height, height, direction);
        self.overrides
            .apply(interaction.focus_ring(root, radius.top_left()))
    }
}

pub(crate) fn use_pressed_scale(held: bool, scale: f32, animation: Animation) -> f32 {
    crate::animation::use_animated(if held { scale } else { 1.0 }, animation)
}

pub(crate) fn knobs_supported() -> bool {
    try_consume_root_context::<KnobSupport>().is_some_and(|support| support.0)
}

pub(crate) fn knob_shadows(glass: Option<Rgba>, shadows: &[ShadowStyle]) -> &[ShadowStyle] {
    if glass.is_some() { &[] } else { shadows }
}

pub(crate) fn use_knob_glass(kind: Knob, tint: Option<Rgba>, scale: f32) -> State<Area> {
    let area = use_state(Area::default);
    let measured = *area.read();
    let knobs = tint
        .filter(|_| measured.width() > 0.0 && measured.height() > 0.0)
        .map_or_else(Knobs::default, |tint| {
            let (width, height) = (measured.width() * scale, measured.height() * scale);
            let left = measured.min_x() + (measured.width() - width) / 2.0;
            let top = measured.min_y() + (measured.height() - height) / 2.0;
            let [red, green, blue, alpha] = tint.to_rgba8();
            Knobs {
                kind,
                tint: u32::from_be_bytes([alpha, red, green, blue]),
                rects: vec![(
                    left.round() as i32,
                    top.round() as i32,
                    width.round() as i32,
                    height.round() as i32,
                )],
            }
        });
    let sent = use_hook(|| Rc::new(RefCell::new(Knobs::default())));
    let outbox = use_hook(KnobOutbox::get);
    if *sent.borrow() != knobs {
        *sent.borrow_mut() = knobs.clone();
        outbox.put(knobs);
    }
    use_drop(move || {
        if *sent.borrow() != Knobs::default() {
            outbox.put(Knobs::default());
        }
    });
    area
}

pub(crate) fn glass_knob(width: f32, height: f32, mut area: State<Area>) -> Rect {
    rect()
        .width(Size::px(width))
        .height(Size::px(height))
        .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
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

// freya 0.5.3 paints by depth (data.rs `LayerState`); a sibling knob must outrank the fill, two levels down.
pub(crate) const KNOB_ABOVE_FILL: i16 = 2;

use ctdesign::{ControlSize, Rgba, SLIDER, SLIDER_KEY_STEP, TEXT_LINE_HEIGHT};
use freya_core::prelude::*;
use std::borrow::Cow;
use std::ops::RangeInclusive;
use torin::prelude::{Alignment, CursorPoint, Direction, Gaps, Position, Size};

use crate::ProbeExt;
use crate::backdrop::Tint;
use crate::controlled::commit;
use crate::controls::{dimmed, inner_border, opacity_when, shadowed};
use crate::elements::{Overrides, forward_key, forward_overrides};
use crate::environment::use_disabled;
use crate::interaction::{control_state, use_interaction};
use crate::layout::ambient;
use crate::level;
use crate::platform::Knob;
use crate::style::with_font;
use crate::styles::{
    Axis, Edge, LayoutDirection, Painted, Paints, Shape, ShapeStyle, SliderStyle, StateValue,
    Theme, resolve,
};

#[derive(PartialEq)]
pub struct Slider {
    label: Cow<'static, str>,
    range: (f32, f32),
    stepped: bool,
    value: State<f32>,
    labels: Vec<Cow<'static, str>>,
    tick_marks: Option<usize>,
    disabled: bool,
    continuous: bool,
    probe: Option<&'static str>,
    style: Theme,
    key: DiffKey,
    overrides: Overrides,
}

impl Slider {
    pub fn new(label: impl Into<Cow<'static, str>>, value: State<f32>) -> Self {
        Self {
            label: label.into(),
            range: (0.0, 1.0),
            stepped: false,
            value,
            labels: Vec::new(),
            tick_marks: None,
            disabled: false,
            continuous: true,
            probe: None,
            style: Theme::tengen(),
            key: DiffKey::None,
            overrides: Overrides::default(),
        }
    }

    /// A slider whose value is a step from 0 to `steps - 1`; the knob stops only at a step.
    pub fn stepped(label: impl Into<Cow<'static, str>>, steps: usize, value: State<f32>) -> Self {
        Self {
            range: (0.0, (steps.max(2) - 1) as f32),
            stepped: true,
            ..Self::new(label, value)
        }
    }

    pub fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = (*range.start(), *range.end());
        self
    }

    pub fn tint(self, tint: Tint) -> Self {
        self.style(move |style| SliderStyle {
            fill: StateValue::new(tint.into()),
            ..style
        })
    }

    pub fn style(mut self, restyle: impl Fn(SliderStyle) -> SliderStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn size(self, size: ControlSize) -> Self {
        self.style(move |style: SliderStyle| style.sized(size))
    }

    pub fn labels<S: Clone + Into<Cow<'static, str>>>(mut self, labels: &[S]) -> Self {
        self.labels = labels.iter().cloned().map(Into::into).collect();
        self
    }

    /// AppKit's `numberOfTickMarks`: the marks spread evenly along the track and never snap the
    /// knob. Without it, a stepped slider has one mark per step and a free one has none.
    pub fn number_of_tick_marks(mut self, count: usize) -> Self {
        self.tick_marks = Some(count);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn continuous(mut self, continuous: bool) -> Self {
        self.continuous = continuous;
        self
    }
}

forward_key!(Slider);

impl ProbeExt for Slider {
    fn probe(mut self, id: &'static str) -> Self {
        self.probe = Some(id);
        self
    }
}

forward_overrides!(Slider);

impl Component for Slider {
    forward_key!();

    fn render(&self) -> impl IntoElement {
        let interaction = use_interaction();
        let style: SliderStyle = resolve(&self.style);
        let disabled = use_disabled(self.disabled);
        let paints = Paints::current(control_state(interaction, disabled));
        let (low, high) = self.range;
        let value = self.value;
        let origin = use_state(CursorPoint::default);
        let continuous = self.continuous;
        let pending = use_state(|| None::<f32>);
        let moves = use_state(|| 0u8);

        let vertical = style.axis == Axis::Vertical;
        let layout_direction = ambient();
        let right_to_left = layout_direction == LayoutDirection::RightToLeft;
        let reversed = vertical || right_to_left;
        let length = style.width;
        let (knob, knob_thickness) = style.knob_size;
        let track = style.track_height;
        let tick = style.tick_size;
        let span = high - low;
        let stepped = self.stepped;
        let steps = stepped.then(|| span.round() as usize + 1);
        let nudge = if stepped { 1.0 } else { span * SLIDER_KEY_STEP };
        let run = (length - knob).max(0.0);
        let current = pending.read().unwrap_or(*value.read()).clamp(low, high);
        let animation = level::click_animation(interaction, moves, style.animation);
        let fraction = (current - low) / span;
        let knob_at = run * crate::animation::use_animated(fraction, animation);
        let knob_scale = level::use_pressed_scale(
            interaction.held(),
            style.knob_pressed_scale,
            style.press_animation,
        );
        let glass = style
            .knob_glass
            .as_ref()
            .filter(|_| interaction.held() && level::knobs_supported())
            .map(|paint| Rgba {
                a: SLIDER.knob_glass_alpha,
                ..paints.color(paint)
            });
        let knob_area = level::use_knob_glass(Knob::Slider, glass, knob_scale);

        let set_from_point = move |point: CursorPoint| {
            let along = if vertical { point.y } else { point.x };
            let at = (along as f32 - knob / 2.0).clamp(0.0, run);
            let at = if reversed { run - at } else { at };
            let value = low + at / run * span;
            if stepped {
                low + (value - low).round()
            } else {
                value
            }
        };

        let update = move |new_value: f32| commit(value, new_value);
        let drag = move |new_value: f32| {
            if continuous {
                update(new_value);
            } else {
                commit(pending, Some(new_value));
            }
        };

        let label_edge = style.labels.filter(|_| !self.labels.is_empty());
        let labels_first =
            matches!(label_edge, Some(Edge::Top | Edge::Leading)) != (vertical && right_to_left);
        let label_line = style.label_font.size as f32 * TEXT_LINE_HEIGHT;
        let (width, height, direction) = match (vertical, label_edge) {
            (true, _) => (Size::auto(), Size::px(length), Direction::Horizontal),
            (false, Some(_)) => (
                Size::px(length),
                Size::px(knob_thickness + style.label_gap + label_line),
                Direction::Vertical,
            ),
            (false, None) => (
                Size::px(length),
                Size::px(knob_thickness),
                Direction::Vertical,
            ),
        };
        let step_name = self
            .labels
            .get((current - low).round() as usize)
            .cloned()
            .filter(|_| steps == Some(self.labels.len()));
        let forward = if right_to_left && !vertical {
            NamedKey::ArrowLeft
        } else {
            NamedKey::ArrowRight
        };
        let root = interaction
            .attach(rect(), !disabled)
            .width(width)
            .height(height)
            .direction(direction)
            .a11y_role(AccessibilityRole::Slider)
            .a11y_alt(&*self.label)
            .a11y_builder(move |node| {
                node.set_numeric_value(f64::from(current));
                node.set_min_numeric_value(f64::from(low));
                node.set_max_numeric_value(f64::from(high));
                node.set_numeric_value_step(f64::from(nudge));
                if let Some(name) = step_name {
                    node.set_value(name);
                }
            })
            .maybe(!disabled, |el| {
                level::on_drag(el, interaction, origin, moves, set_from_point, drag)
                    .on_global_pointer_up(move |_| {
                        interaction.release();
                        let mut pending = pending;
                        let released = *pending.peek();
                        if let Some(done) = released {
                            pending.set(None);
                            update(done);
                        }
                    })
                    .on_key_down(move |event: Event<KeyboardEventData>| match event.key {
                        Key::Named(NamedKey::ArrowUp) => update((current + nudge).min(high)),
                        Key::Named(key) if key == forward => update((current + nudge).min(high)),
                        Key::Named(
                            NamedKey::ArrowLeft | NamedKey::ArrowRight | NamedKey::ArrowDown,
                        ) => update((current - nudge).max(low)),
                        _ => {}
                    })
            });

        let place = |main: f32, size: f32, parent: f32, cross: f32, thickness: f32| {
            let main = if reversed { parent - main - size } else { main };
            let (left, top, width, height) = if vertical {
                (cross, main, thickness, size)
            } else {
                (main, cross, size, thickness)
            };
            (Position::new_absolute().left(left).top(top), width, height)
        };
        let part = |main: f32,
                    size: f32,
                    parent: f32,
                    cross: f32,
                    thickness: f32,
                    shape: Shape,
                    paint: &StateValue<ShapeStyle>| {
            let (position, width, height) = place(main, size, parent, cross, thickness);
            rect()
                .position(position)
                .width(Size::px(width))
                .height(Size::px(height))
                .corner_radius(shape.corner_radius(width, height, layout_direction))
                .painted(&paints, paint)
        };
        // freya 0.5.3 dims each element on its own (render_pipeline.rs): a dimmed knob would show the fill.
        let parts_opacity = opacity_when(disabled, style.opacity, style.disabled_opacity);

        let track_rect = dimmed(
            part(
                0.0,
                length,
                length,
                (knob_thickness - track) / 2.0,
                track,
                style.track_shape,
                &style.track,
            ),
            parts_opacity,
        );
        let fill_rect = part(
            0.0,
            knob_at + knob / 2.0,
            length,
            0.0,
            track,
            style.track_shape,
            &style.fill,
        );
        let tick_cross = if labels_first {
            (knob_thickness - track) / 2.0 - style.tick_gap - tick
        } else {
            (knob_thickness + track) / 2.0 + style.tick_gap
        };
        let marks = self.tick_marks.or(steps).unwrap_or(0);
        let mark_segments = marks.saturating_sub(1).max(1) as f32;
        let ticks = (0..marks).map(|i| {
            let x = knob / 2.0 + run * (i as f32 / mark_segments);
            dimmed(
                part(
                    x - tick / 2.0,
                    tick,
                    length,
                    tick_cross,
                    tick,
                    style.tick_shape,
                    &style.tick,
                ),
                parts_opacity,
            )
            .into_element()
        });
        let knob_body = match glass {
            Some(_) => {
                let (position, width, height) = place(knob_at, knob, length, 0.0, knob_thickness);
                level::glass_knob(width, height, knob_area).position(position)
            }
            None => part(
                knob_at,
                knob,
                length,
                0.0,
                knob_thickness,
                style.knob_shape,
                &style.knob,
            ),
        };
        let knob_rect = shadowed(knob_body, level::knob_shadows(glass, &style.knob_shadows))
            .map(style.knob_border.as_ref(), |el, border| {
                el.border(inner_border(&paints, border))
            })
            .layer(level::KNOB_ABOVE_FILL)
            .maybe(knob_scale != 1.0, |el| el.scale(knob_scale));
        let knob_rect = dimmed(knob_rect, style.opacity);
        let (band_width, band_height) = if vertical {
            (knob_thickness, length)
        } else {
            (length, knob_thickness)
        };
        let band = rect()
            .width(Size::px(band_width))
            .height(Size::px(band_height))
            .children(ticks)
            .child(track_rect.child(fill_rect))
            .child(knob_rect);

        let shown = steps.unwrap_or(self.labels.len());
        let labels_row = label_edge.map(|_| {
            let names = self.labels.iter().take(shown).map(|text| {
                let label_node = with_font(label().text(text.clone()), style.label_font)
                    .max_lines(1)
                    .color(paints.fill(&style.label));
                rect().child(label_node).into_element()
            });
            let names: Vec<Element> = if reversed {
                names.rev().collect()
            } else {
                names.collect()
            };
            let gap = style.label_gap;
            let margin = match (vertical, labels_first) {
                (false, false) => Gaps::new(gap, 0.0, 0.0, 0.0),
                (false, true) => Gaps::new(0.0, 0.0, gap, 0.0),
                (true, false) => Gaps::new(0.0, 0.0, 0.0, gap),
                (true, true) => Gaps::new(0.0, gap, 0.0, 0.0),
            };
            let row = if vertical {
                rect()
                    .height(Size::px(length))
                    .direction(Direction::Vertical)
                    .padding(Gaps::new(knob / 2.0, 0.0, knob / 2.0, 0.0))
            } else {
                rect()
                    .width(Size::px(length))
                    .height(Size::px(label_line))
                    .direction(Direction::Horizontal)
                    .padding(Gaps::new(0.0, knob / 2.0, 0.0, knob / 2.0))
            };
            dimmed(
                row.main_align(Alignment::SpaceBetween)
                    .margin(margin)
                    .children(names),
                parts_opacity,
            )
        });

        let root = if labels_first {
            root.map(labels_row, |el, row| el.child(row)).child(band)
        } else {
            root.child(band).map(labels_row, |el, row| el.child(row))
        };
        let root = interaction.focus_ring(root, knob / 2.0);

        self.overrides.apply(root.map(self.probe, ProbeExt::probe))
    }
}

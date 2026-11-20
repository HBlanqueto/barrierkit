use std::ops::RangeInclusive;

use ctdesign::{SLIDER, SLIDER_KEY_STEP};

use super::*;
use crate::platform::Knob;
use crate::styles::Animation;

#[derive(PartialEq)]
pub struct Slider {
    label: &'static str,
    range: (f32, f32),
    stepped: bool,
    value: f32,
    labels: &'static [&'static str],
    tick_marks: Option<usize>,
    disabled: bool,
    continuous: bool,
    on_change: Option<Callback<f32, ()>>,
    probe: Option<&'static str>,
    style: Theme,
    key: DiffKey,
}

impl Slider {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            range: (0.0, 1.0),
            stepped: false,
            value: 0.0,
            labels: &[],
            tick_marks: None,
            disabled: false,
            continuous: true,
            on_change: None,
            probe: None,
            style: Theme::tengen(),
            key: DiffKey::None,
        }
    }

    /// A slider whose value is a step from 0 to `steps - 1`; the knob stops only at a step.
    pub fn stepped(label: &'static str, steps: usize) -> Self {
        Self {
            range: (0.0, (steps.max(2) - 1) as f32),
            stepped: true,
            ..Self::new(label)
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

    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    pub fn labels(mut self, labels: &'static [&'static str]) -> Self {
        self.labels = labels;
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

    pub fn on_change(mut self, callback: impl Into<Callback<f32, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for Slider {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl ProbeExt for Slider {
    fn probe(mut self, id: &'static str) -> Self {
        self.probe = Some(id);
        self
    }
}

impl Component for Slider {
    fn render(&self) -> impl IntoElement {
        let interaction = use_interaction();
        let style: SliderStyle = resolve(&self.style);
        let paints = Paints::current(control_state(interaction, self.disabled), style.effect);
        let (low, high) = self.range;
        let value = crate::controlled::use_controlled_unless(
            &self.value.clamp(low, high),
            interaction.held(),
        );
        let origin = use_state(CursorPoint::default);
        let on_change = self.on_change.clone();
        let on_release = self.on_change.clone();
        let continuous = self.continuous;
        let pending = use_state(|| None::<f32>);

        let vertical = style.axis == Axis::Vertical;
        let right_to_left = style.layout_direction == LayoutDirection::RightToLeft;
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
        let current = (*value.read()).clamp(low, high);
        let animation = if interaction.held() {
            Animation::None
        } else {
            style.animation
        };
        let fraction = (current - low) / span;
        let knob_at = run * crate::animation::use_animated(fraction, animation);
        let knob_scale = level::use_pressed_scale(
            interaction.held(),
            style.knob_pressed_scale,
            style.animation,
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

        let update = move |new_value: f32| commit(value, &on_change, new_value);
        let drag = {
            let update = update.clone();
            move |new_value: f32| {
                if continuous {
                    update(new_value);
                } else if new_value != *value.peek() {
                    let (mut value, mut pending) = (value, pending);
                    value.set(new_value);
                    pending.set(Some(new_value));
                }
            }
        };

        let labels_first =
            matches!(style.labels, Some(Edge::Top | Edge::Leading)) != (vertical && right_to_left);
        let label_line = style.label_font.size as f32 * TEXT_LINE_HEIGHT;
        let (width, height, direction) = match (vertical, style.labels) {
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
            .get(current.round() as usize)
            .copied()
            .filter(|_| steps == Some(self.labels.len()));
        let mut root = interaction
            .attach(rect(), !self.disabled)
            .width(width)
            .height(height)
            .direction(direction)
            .a11y_role(AccessibilityRole::Slider)
            .a11y_alt(self.label)
            .a11y_builder(move |node| {
                node.set_numeric_value(f64::from(current));
                node.set_min_numeric_value(f64::from(low));
                node.set_max_numeric_value(f64::from(high));
                node.set_numeric_value_step(f64::from(nudge));
                if let Some(name) = step_name {
                    node.set_value(name);
                }
            });

        if !self.disabled {
            let forward = if right_to_left && !vertical {
                NamedKey::ArrowLeft
            } else {
                NamedKey::ArrowRight
            };
            root = level::on_drag(root, interaction, origin, set_from_point, drag)
                .on_global_pointer_up(move |_| {
                    interaction.release();
                    let mut pending = pending;
                    let released = *pending.peek();
                    if let (Some(done), Some(callback)) = (released, &on_release) {
                        pending.set(None);
                        callback.call(done);
                    }
                })
                .on_key_down(move |event: Event<KeyboardEventData>| match event.key {
                    Key::Named(NamedKey::ArrowUp) => update((current + nudge).min(high)),
                    Key::Named(key) if key == forward => update((current + nudge).min(high)),
                    Key::Named(
                        NamedKey::ArrowLeft | NamedKey::ArrowRight | NamedKey::ArrowDown,
                    ) => update((current - nudge).max(low)),
                    _ => {}
                });
        }

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
                .corner_radius(shape.corner_radius(width, height, style.layout_direction))
                .painted(&paints, paint)
        };
        let dim = |el: Rect, opacity: f32| el.maybe(opacity < 1.0, |el| el.opacity(opacity));
        // freya 0.5.0 dims each element on its own (render_pipeline.rs): a dimmed knob would show the fill.
        let parts_opacity = opacity_when(self.disabled, style.opacity, style.disabled_opacity);

        let track_rect = dim(
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
            dim(
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
        let knob_rect = level::knob_shadows(glass, &style.knob_shadows)
            .iter()
            .fold(knob_body, |el, shadow| el.shadow(*shadow))
            .map(style.knob_border.as_ref(), |el, (paint, width)| {
                el.border(
                    Border::new()
                        .width(*width)
                        .alignment(BorderAlignment::Inner)
                        .fill(color(paints.color(paint))),
                )
            })
            .layer(level::KNOB_ABOVE_FILL)
            .maybe(knob_scale != 1.0, |el| el.scale(knob_scale));
        let knob_rect = dim(knob_rect, style.opacity);
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
        let labels_row = style.labels.map(|_| {
            let names = self.labels.iter().take(shown).map(|text| {
                let label_node = with_font(label().text(*text), style.label_font)
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
            dim(
                row.main_align(Alignment::SpaceBetween)
                    .margin(margin)
                    .children(names),
                parts_opacity,
            )
        });

        root = if labels_first {
            root.map(labels_row, |el, row| el.child(row)).child(band)
        } else {
            root.child(band).map(labels_row, |el, row| el.child(row))
        };
        root = interaction.focus_ring(root, knob / 2.0);

        root.map(self.probe, ProbeExt::probe)
    }
}

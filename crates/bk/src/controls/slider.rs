use super::*;

#[derive(PartialEq)]
pub struct Slider {
    label: &'static str,
    steps: usize,
    value: usize,
    labels: &'static [&'static str],
    disabled: bool,
    on_change: Option<Callback<usize, ()>>,
    probe: Option<&'static str>,
    style: Theme,
    key: DiffKey,
}

impl Slider {
    pub fn stepped(label: &'static str, steps: usize) -> Self {
        Self {
            label,
            steps: steps.max(2),
            value: 0,
            labels: &[],
            disabled: false,
            on_change: None,
            probe: None,
            style: Theme::tengen(),
            key: DiffKey::None,
        }
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

    pub fn value(mut self, value: usize) -> Self {
        self.value = value.min(self.steps.saturating_sub(1));
        self
    }

    pub fn labels(mut self, labels: &'static [&'static str]) -> Self {
        self.labels = labels;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, callback: impl Into<Callback<usize, ()>>) -> Self {
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
        let paints = Paints::current(
            ControlState {
                hovered: interaction.hovered(),
                pressed: interaction.pressed(),
                focused: interaction.focused(),
                disabled: self.disabled,
            },
            style.effect,
        );
        let value = crate::controlled::use_controlled(&self.value);
        let origin = use_state(CursorPoint::default);
        let on_change = self.on_change.clone();

        let vertical = style.axis == Axis::Vertical;
        let right_to_left = style.layout_direction == LayoutDirection::RightToLeft;
        let reversed = vertical || right_to_left;
        let length = style.width;
        let (knob, knob_thickness) = style.knob_size;
        let track = style.track_height;
        let tick = style.tick_size;
        let steps = self.steps;
        let segments = steps.saturating_sub(1).max(1) as f32;
        let run = length - knob;
        let current = *value.read();
        let knob_at =
            run * crate::animation::use_animated(current as f32 / segments, style.animation);

        let set_from_point = move |point: CursorPoint| {
            let along = if vertical { point.y } else { point.x };
            let at = (along as f32 - knob / 2.0).clamp(0.0, run);
            let at = if reversed { run - at } else { at };
            let step = ((at / run) * segments).round() as usize;
            step.min(steps - 1)
        };

        let update = move |new_value: usize| commit(value, &on_change, new_value);

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
        let step_name = self.labels.get(current).copied();
        let mut root = interaction
            .attach(rect(), !self.disabled)
            .width(width)
            .height(height)
            .direction(direction)
            .a11y_role(AccessibilityRole::Slider)
            .a11y_alt(self.label)
            .a11y_builder(move |node| {
                node.set_numeric_value(current as f64);
                node.set_min_numeric_value(0.0);
                node.set_max_numeric_value((steps - 1) as f64);
                node.set_numeric_value_step(1.0);
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
            root = level::on_drag(root, interaction, origin, set_from_point, update.clone())
                .on_key_down(move |event: Event<KeyboardEventData>| match event.key {
                    Key::Named(NamedKey::ArrowUp) => update((current + 1).min(steps - 1)),
                    Key::Named(key) if key == forward => update((current + 1).min(steps - 1)),
                    Key::Named(
                        NamedKey::ArrowLeft | NamedKey::ArrowRight | NamedKey::ArrowDown,
                    ) => update(current.saturating_sub(1)),
                    _ => {}
                });
        }

        let part = |main: f32,
                    size: f32,
                    parent: f32,
                    cross: f32,
                    thickness: f32,
                    shape: Shape,
                    paint: &StateValue<ShapeStyle>| {
            let main = if reversed { parent - main - size } else { main };
            let (left, top, width, height) = if vertical {
                (cross, main, thickness, size)
            } else {
                (main, cross, size, thickness)
            };
            rect()
                .position(Position::new_absolute().left(left).top(top))
                .width(Size::px(width))
                .height(Size::px(height))
                .corner_radius(shape.corner_radius(width, height, style.layout_direction))
                .painted(&paints, paint)
        };
        let dim = |el: Rect, opacity: f32| el.maybe(opacity < 1.0, |el| el.opacity(opacity));
        // freya 0.5.0 dims each element on its own (render_pipeline.rs): a dimmed knob would show the fill.
        let parts_opacity = if self.disabled {
            style.opacity * style.disabled_opacity
        } else {
            style.opacity
        };

        let track_rect = dim(
            part(
                knob / 2.0,
                run,
                length,
                (knob_thickness - track) / 2.0,
                track,
                style.track_shape,
                &style.track,
            ),
            parts_opacity,
        );
        let fill_rect = part(
            -knob / 2.0,
            knob_at + knob / 2.0,
            run,
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
        let ticks = (0..steps).map(|i| {
            let x = knob / 2.0 + run * (i as f32 / segments);
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
        let knob_rect = style
            .knob_shadows
            .iter()
            .fold(
                part(
                    knob_at,
                    knob,
                    length,
                    0.0,
                    knob_thickness,
                    style.knob_shape,
                    &style.knob,
                ),
                |el, shadow| el.shadow(*shadow),
            )
            .map(style.knob_border.as_ref(), |el, (paint, width)| {
                el.border(
                    Border::new()
                        .width(*width)
                        .alignment(BorderAlignment::Inner)
                        .fill(color(paints.color(paint))),
                )
            })
            .layer(level::KNOB_ABOVE_FILL);
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

        let labels_row = style.labels.map(|_| {
            let names = self.labels.iter().take(steps).map(|text| {
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
            } else {
                rect()
                    .width(Size::px(length))
                    .height(Size::px(label_line))
                    .direction(Direction::Horizontal)
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

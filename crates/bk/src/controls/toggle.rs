use super::*;

#[derive(PartialEq)]
pub struct Switch {
    label: &'static str,
    on: bool,
    disabled: bool,
    on_toggle: Option<Callback<bool, ()>>,
    probe: Option<&'static str>,
    style: Theme,
    key: DiffKey,
}

impl Switch {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            on: false,
            disabled: false,
            on_toggle: None,
            probe: None,
            style: Theme::tengen(),
            key: DiffKey::None,
        }
    }

    pub fn tint(self, tint: Tint) -> Self {
        self.style(move |style| SwitchStyle {
            fill: StateValue::new(tint.into()),
            ..style
        })
    }

    pub fn style(mut self, restyle: impl Fn(SwitchStyle) -> SwitchStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_toggle(mut self, callback: impl Into<Callback<bool, ()>>) -> Self {
        self.on_toggle = Some(callback.into());
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for Switch {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl ProbeExt for Switch {
    fn probe(mut self, id: &'static str) -> Self {
        self.probe = Some(id);
        self
    }
}

impl Component for Switch {
    fn render(&self) -> impl IntoElement {
        let on = crate::controlled::use_controlled(&self.on);
        let interaction = use_interaction();
        let style: SwitchStyle = resolve(&self.style);
        let state = control_state(interaction, self.disabled);
        let paints = Paints::current(state, style.effect);
        let is_on = *on.read();
        let progress = crate::animation::use_animated(f32::from(u8::from(is_on)), style.animation);

        let disabled = self.disabled;
        let on_toggle = self.on_toggle.clone();
        let direction = style.layout_direction;
        let (width, height) = (style.width, style.height);
        let (knob_width, knob_height) = style.knob_size;
        let travel = width - knob_width - 2.0 * style.inset;
        let along = if direction == LayoutDirection::RightToLeft {
            1.0 - progress
        } else {
            progress
        };
        let opacity = opacity_when(disabled, style.opacity, style.disabled_opacity);

        let knob = rect()
            .position(
                Position::new_absolute()
                    .left(style.inset + travel * along)
                    .top((height - knob_height) / 2.0),
            )
            .width(Size::px(knob_width))
            .height(Size::px(knob_height))
            .corner_radius(
                style
                    .knob_shape
                    .corner_radius(knob_width, knob_height, direction),
            )
            .painted(&paints, &style.knob)
            .bordered(&paints, style.knob_border.as_ref());
        let track_paint = if is_on { &style.fill } else { &style.track };
        let radius = style.track_shape.corner_radius(width, height, direction);
        let track = rect()
            .width(Size::px(width))
            .height(Size::px(height))
            .corner_radius(radius)
            .painted(&paints, track_paint)
            .map(
                style.track_border.as_ref().filter(|_| !is_on),
                |el, border| el.border(inner_border(&paints, border)),
            )
            .child(shadowed(knob, &style.knob_shadows));
        let track = interaction.focus_ring(dimmed(track, opacity), radius.top_left());
        let control = match style.label_position {
            None => track,
            Some(edge) => {
                let text = with_font(label().text(self.label), style.label_font)
                    .color(paints.fill(&style.label));
                labeled(
                    rect(),
                    track,
                    dimmed_label(text, opacity),
                    edge,
                    direction,
                    style.label_gap,
                )
            }
        };

        interaction
            .attach(control, !disabled)
            .a11y_role(AccessibilityRole::Switch)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_toggled(is_on.into()))
            .on_press(move |_| {
                if !disabled {
                    let next = !*on.peek();
                    commit(on, &on_toggle, next);
                }
            })
            .map(self.probe, ProbeExt::probe)
    }
}

#[derive(PartialEq)]
pub struct Checkbox {
    label: &'static str,
    checked: bool,
    disabled: bool,
    on_toggle: Option<Callback<bool, ()>>,
    probe: Option<&'static str>,
    width: Size,
    style: Theme,
    key: DiffKey,
}

impl Checkbox {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            checked: false,
            disabled: false,
            on_toggle: None,
            probe: None,
            width: Size::auto(),
            style: Theme::tengen(),
            key: DiffKey::None,
        }
    }

    pub fn tint(self, tint: Tint) -> Self {
        self.style(move |style| CheckboxStyle {
            fill: StateValue::new(tint.into()),
            ..style
        })
    }

    pub fn style(mut self, restyle: impl Fn(CheckboxStyle) -> CheckboxStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_toggle(mut self, callback: impl Into<Callback<bool, ()>>) -> Self {
        self.on_toggle = Some(callback.into());
        self
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for Checkbox {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl ProbeExt for Checkbox {
    fn probe(mut self, id: &'static str) -> Self {
        self.probe = Some(id);
        self
    }
}

impl Component for Checkbox {
    fn render(&self) -> impl IntoElement {
        let checked = crate::controlled::use_controlled(&self.checked);
        let on_toggle = self.on_toggle.clone();
        let disabled = self.disabled;
        let label_text = self.label;
        let interaction = use_interaction();
        let style: CheckboxStyle = resolve(&self.style);
        let state = control_state(interaction, disabled);
        let paints = Paints::current(state, style.effect);
        let is_checked = *checked.read();
        let opacity = opacity_when(disabled, style.opacity, style.disabled_opacity);

        let size = style.box_size;
        let radius = style
            .box_shape
            .corner_radius(size, size, style.layout_direction);
        let paint = if is_checked {
            &style.fill
        } else {
            &style.box_background
        };
        let checkbox = rect()
            .width(Size::px(size))
            .height(Size::px(size))
            .corner_radius(radius)
            .painted(&paints, paint)
            .map(
                style.box_border.as_ref().filter(|_| !is_checked),
                |el, border| el.border(inner_border(&paints, border)),
            )
            .center()
            .maybe(is_checked, |el| {
                el.child(symbol(
                    Symbol::Checkmark,
                    style.mark_size,
                    paints.color(style.mark.resolve(state)),
                ))
            });
        let checkbox = interaction.focus_ring(
            dimmed(shadowed(checkbox, &style.box_shadows), opacity),
            radius.top_left(),
        );

        let row = interaction
            .attach(rect(), !disabled)
            .width(self.width.clone())
            .a11y_role(AccessibilityRole::CheckBox)
            .a11y_alt(label_text)
            .a11y_builder(move |node| node.set_toggled(is_checked.into()))
            .on_press(move |_| {
                if !disabled {
                    let next = !*checked.peek();
                    commit(checked, &on_toggle, next);
                }
            });
        let root = match style.label_position {
            None => row
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center)
                .child(checkbox),
            Some(edge) => {
                let text = with_font(label().text(label_text), style.label_font)
                    .color(paints.fill(&style.label));
                labeled(
                    row,
                    checkbox,
                    dimmed_label(text, opacity),
                    edge,
                    style.layout_direction,
                    style.label_gap,
                )
            }
        };

        root.map(self.probe, ProbeExt::probe)
    }
}

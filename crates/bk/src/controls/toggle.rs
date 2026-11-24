use ctdesign::DRAG;
use std::borrow::Cow;

use super::*;
use crate::elements::forward_key;
use crate::platform::Knob;

#[derive(Clone, Copy, PartialEq)]
struct Grab {
    from: f32,
    shift: f32,
    was_on: bool,
    moved: bool,
}

fn grabbed_progress(was_on: bool, shift: f32, travel: f32, right_to_left: bool) -> f32 {
    let mirrored = |fraction: f32| {
        if right_to_left {
            1.0 - fraction
        } else {
            fraction
        }
    };
    mirrored((mirrored(f32::from(u8::from(was_on))) + shift / travel.max(1.0)).clamp(0.0, 1.0))
}

fn track_is_on(is_on: bool, dragged: Option<f32>) -> bool {
    dragged.map_or(is_on, |progress| progress > 0.0)
}

fn released_on(grab: Grab, travel: f32, right_to_left: bool) -> bool {
    if grab.moved {
        grabbed_progress(grab.was_on, grab.shift, travel, right_to_left) >= 0.5
    } else {
        !grab.was_on
    }
}

#[derive(PartialEq)]
pub struct Switch {
    label: Cow<'static, str>,
    on: bool,
    disabled: bool,
    on_toggle: Option<Callback<bool, ()>>,
    probe: Option<&'static str>,
    style: Theme,
    key: DiffKey,
    overrides: Overrides,
}

impl Switch {
    pub fn new(label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            label: label.into(),
            on: false,
            disabled: false,
            on_toggle: None,
            probe: None,
            style: Theme::tengen(),
            key: DiffKey::None,
            overrides: Overrides::default(),
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

    pub fn size(self, size: ControlSize) -> Self {
        self.style(move |style: SwitchStyle| style.sized(size))
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
}

forward_key!(Switch);

impl ProbeExt for Switch {
    fn probe(mut self, id: &'static str) -> Self {
        self.probe = Some(id);
        self
    }
}

forward_overrides!(Switch);

impl Component for Switch {
    forward_key!();

    fn render(&self) -> impl IntoElement {
        let interaction = use_interaction();
        let on = crate::controlled::use_controlled_unless(&self.on, interaction.held());
        let grab = use_state(|| None::<Grab>);
        let style: SwitchStyle = resolve(&self.style);
        let disabled = use_disabled(self.disabled);
        let state = control_state(interaction, disabled);
        let paints = Paints::current(state);
        let is_on = *on.read();

        let on_toggle = self.on_toggle.clone();
        let on_drag = self.on_toggle.clone();
        let on_key = self.on_toggle.clone();
        let direction = ambient();
        let right_to_left = direction == LayoutDirection::RightToLeft;
        let (width, height) = (style.width, style.height);
        let (knob_width, knob_height) = style.knob_size;
        let travel = width - knob_width - 2.0 * style.inset;
        let dragged = (*grab.read())
            .map(|current| grabbed_progress(current.was_on, current.shift, travel, right_to_left));
        let animation = crate::animation::effective(style.animation, interaction.held());
        let progress = crate::animation::use_animated(
            dragged.unwrap_or(f32::from(u8::from(is_on))),
            animation,
        );
        let knob_scale = level::use_pressed_scale(
            interaction.held(),
            style.knob_pressed_scale,
            style.animation,
        );
        let glass = style
            .knob_glass
            .as_ref()
            .filter(|_| interaction.held() && level::knobs_supported())
            .map(|paint| paints.color(paint));
        let knob_area = level::use_knob_glass(Knob::Switch, glass, knob_scale);
        let shows_on = track_is_on(is_on, dragged);
        let along = if right_to_left {
            1.0 - progress
        } else {
            progress
        };
        let opacity = opacity_when(disabled, style.opacity, style.disabled_opacity);

        let knob_radius = style
            .knob_shape
            .corner_radius(knob_width, knob_height, direction);
        let knob_body = match glass {
            Some(_) => level::glass_knob(knob_width, knob_height, knob_area),
            None => rect()
                .width(Size::px(knob_width))
                .height(Size::px(knob_height))
                .corner_radius(knob_radius)
                .painted(&paints, &style.knob),
        };
        let knob = knob_body
            .position(
                Position::new_absolute()
                    .left(style.inset + travel * along)
                    .top((height - knob_height) / 2.0),
            )
            .bordered(&paints, style.knob_border.as_ref())
            .maybe(knob_scale != 1.0, |el| el.scale(knob_scale))
            .layer(1);
        let track_paint = if shows_on { &style.fill } else { &style.track };
        let radius = style.track_shape.corner_radius(width, height, direction);
        let track_body = rect()
            .width(Size::px(width))
            .height(Size::px(height))
            .corner_radius(radius)
            .painted(&paints, track_paint)
            .map(
                style.track_border.as_ref().filter(|_| !shows_on),
                |el, border| el.border(inner_border(&paints, border)),
            );
        let track = rect()
            .width(Size::px(width))
            .height(Size::px(height))
            .child(dimmed(track_body, opacity))
            .child(shadowed(
                knob,
                level::knob_shadows(glass, &style.knob_shadows),
            ));
        let track = interaction.focus_ring(track, radius.top_left());
        let control = match style.label_position {
            None => track,
            Some(edge) => labeled(
                rect(),
                track,
                edge_label(
                    self.label.clone(),
                    style.label_font,
                    paints.fill(&style.label),
                    opacity,
                ),
                edge,
                style.label_gap,
            ),
        };

        let root = interaction
            .attach(control, !disabled)
            .a11y_role(AccessibilityRole::Switch)
            .a11y_alt(&*self.label)
            .a11y_builder(move |node| node.set_toggled(is_on.into()))
            .maybe(!disabled, |el| {
                el.on_mouse_down(move |event: Event<MouseEventData>| {
                    if event.button == Some(MouseButton::Left) {
                        let mut grab = grab;
                        grab.set(Some(Grab {
                            from: event.global_location.x as f32,
                            shift: 0.0,
                            was_on: *on.peek(),
                            moved: false,
                        }));
                    }
                })
                .on_global_pointer_move(move |event: Event<PointerEventData>| {
                    let mut grab = grab;
                    let Some(current) = *grab.peek() else {
                        return;
                    };
                    let shift = event.global_location().x as f32 - current.from;
                    let current = Grab {
                        shift,
                        moved: current.moved || shift.abs() > DRAG.threshold,
                        ..current
                    };
                    grab.set_if_modified(Some(current));
                    if current.moved {
                        commit(on, &on_drag, released_on(current, travel, right_to_left));
                    }
                })
                .on_global_pointer_up(move |_| {
                    interaction.release();
                    let mut grab = grab;
                    let Some(current) = *grab.peek() else {
                        return;
                    };
                    grab.set(None);
                    let next = released_on(current, travel, right_to_left);
                    commit(on, &on_toggle, next);
                })
                .on_key_down(move |event: Event<KeyboardEventData>| {
                    if event.key == Key::Character(" ".into()) {
                        let next = !*on.peek();
                        commit(on, &on_key, next);
                    }
                })
            })
            .map(self.probe, ProbeExt::probe);
        self.overrides.apply(root)
    }
}

#[derive(PartialEq)]
pub struct Checkbox {
    label: Cow<'static, str>,
    checked: bool,
    mixed: bool,
    disabled: bool,
    on_toggle: Option<Callback<bool, ()>>,
    probe: Option<&'static str>,
    width: Size,
    style: Theme,
    key: DiffKey,
    overrides: Overrides,
}

impl Checkbox {
    pub fn new(label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            label: label.into(),
            checked: false,
            mixed: false,
            disabled: false,
            on_toggle: None,
            probe: None,
            width: Size::auto(),
            style: Theme::tengen(),
            key: DiffKey::None,
            overrides: Overrides::default(),
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

    pub fn size(self, size: ControlSize) -> Self {
        self.style(move |style: CheckboxStyle| style.sized(size))
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn mixed(mut self, mixed: bool) -> Self {
        self.mixed = mixed;
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
}

forward_key!(Checkbox);

impl ProbeExt for Checkbox {
    fn probe(mut self, id: &'static str) -> Self {
        self.probe = Some(id);
        self
    }
}

forward_overrides!(Checkbox);

impl Component for Checkbox {
    forward_key!();

    fn render(&self) -> impl IntoElement {
        let checked = crate::controlled::use_controlled(&self.checked);
        let on_toggle = self.on_toggle.clone();
        let disabled = use_disabled(self.disabled);
        let label_text = self.label.clone();
        let interaction = use_interaction();
        let style: CheckboxStyle = resolve(&self.style);
        let state = control_state(interaction, disabled);
        let paints = Paints::current(state);
        let is_checked = *checked.read();
        let opacity = opacity_when(disabled, style.opacity, style.disabled_opacity);

        let size = style.box_size;
        let radius = style.box_shape.corner_radius(size, size, ambient());
        let mixed = self.mixed;
        let filled = is_checked || mixed;
        let paint = if filled {
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
                style
                    .box_border
                    .as_ref()
                    .filter(|_| !filled && !disabled && !state.pressed),
                |el, border| el.border(inner_border(&paints, border)),
            )
            .center()
            .maybe(filled, |el| {
                let (symbol, size, stroke) = if mixed {
                    (Symbol::Minus, style.mixed_mark.0, style.mixed_mark.1)
                } else {
                    (Symbol::Checkmark, style.mark_size, style.mark_stroke)
                };
                el.child(stroked_symbol(
                    symbol,
                    size,
                    stroke,
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
            .a11y_alt(&*label_text)
            .a11y_builder(move |node| {
                node.set_toggled(if mixed {
                    accesskit::Toggled::Mixed
                } else {
                    is_checked.into()
                })
            })
            // Return goes to the default button, as in AppKit; Space toggles.
            .on_press(move |event: Event<PressEventData>| {
                let return_key = matches!(&*event, PressEventData::Keyboard(key) if key.key == Key::Named(NamedKey::Enter));
                if !disabled && !return_key {
                    let next = !*checked.peek();
                    commit(checked, &on_toggle, next);
                }
            });
        let root = match style.label_position {
            None => row
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center)
                .child(checkbox),
            Some(edge) => labeled(
                row,
                checkbox,
                edge_label(
                    label_text,
                    style.label_font,
                    paints.fill(&style.label),
                    opacity,
                ),
                edge,
                style.label_gap,
            ),
        };

        self.overrides.apply(root.map(self.probe, ProbeExt::probe))
    }
}

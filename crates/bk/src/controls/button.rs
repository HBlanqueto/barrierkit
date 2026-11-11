use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControlSize {
    Small,
    #[default]
    Regular,
    Large,
}

#[derive(PartialEq)]
pub struct Button {
    label: String,
    image: Option<Symbol>,
    disabled: bool,
    on: Option<bool>,
    width: Size,
    on_press: Option<Callback<(), ()>>,
    pub(crate) style: Theme,
    key: DiffKey,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            image: None,
            disabled: false,
            on: None,
            width: Size::auto(),
            on_press: None,
            style: Theme::tengen(),
            key: DiffKey::None,
        }
    }

    pub fn system_image(label: impl Into<String>, image: Symbol) -> Self {
        Self {
            image: Some(image),
            ..Self::new(label)
        }
    }

    pub fn toggled(mut self, on: bool) -> Self {
        self.on = Some(on);
        self
    }

    pub fn style(mut self, restyle: impl Fn(ButtonStyle) -> ButtonStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn size(self, size: ControlSize) -> Self {
        self.style(move |style| style.sized(size))
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }

    pub fn on_press(mut self, callback: impl Into<Callback<(), ()>>) -> Self {
        self.on_press = Some(callback.into());
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for Button {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for Button {
    fn render(&self) -> impl IntoElement {
        let interaction = use_interaction();
        let enabled = !self.disabled;
        let style: ButtonStyle = resolve(&self.style);
        let style = if self.on == Some(true) {
            style.bordered_prominent()
        } else {
            style
        };
        let state = ControlState {
            hovered: interaction.hovered(),
            pressed: interaction.pressed(),
            focused: interaction.focused(),
            disabled: self.disabled,
        };
        let paints = Paints::current(state, style.effect);
        let pressed = enabled && interaction.pressed();
        let scale = crate::animation::use_animated(
            if pressed { style.pressed_scale } else { 1.0 },
            style.animation,
        );
        let on_press = self.on_press.clone();
        let radius =
            style
                .shape
                .corner_radius(style.height, style.height, LayoutDirection::LeftToRight);
        let opacity = if enabled {
            style.opacity
        } else {
            style.opacity * style.disabled_opacity
        };
        let font = style.font;

        let button = interaction
            .attach(rect(), enabled)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label.clone())
            .width(self.width.clone())
            .height(Size::px(style.height))
            .padding(Gaps::new(0.0, style.padding_x, 0.0, style.padding_x))
            .corner_radius(radius)
            .center()
            .direction(Direction::Horizontal)
            .spacing(ctdesign::SPACE_1)
            .painted(&paints, &style.background)
            .bordered(&paints, style.border.as_ref())
            .map(self.on, |el, on| {
                el.a11y_builder(move |node| {
                    node.set_toggled(if on {
                        accesskit::Toggled::True
                    } else {
                        accesskit::Toggled::False
                    })
                })
            })
            .maybe(scale != 1.0, |el| el.scale(scale))
            .maybe(enabled, |el| {
                el.on_press(move |event: Event<PressEventData>| {
                    // A focused button consumes Enter/Space so global key handlers (Alert) skip it.
                    if matches!(*event, PressEventData::Keyboard(_)) {
                        event.prevent_default();
                    }
                    if let Some(cb) = &on_press {
                        cb.call(());
                    }
                })
            })
            .maybe_child(self.image.map(|image| {
                crate::symbol::Image::system(image)
                    .size(font.size as f32)
                    .color(paints.color(style.label.resolve(state)))
            }))
            .child(
                label()
                    .text(self.label.clone())
                    .font_family(font.family)
                    .font_size(font.size as f32)
                    .font_weight(font.weight as i32)
                    .max_lines(1)
                    .color(paints.fill(&style.label)),
            );
        let button = dimmed(shadowed(button, &style.shadows), opacity);
        interaction.focus_ring(button, radius.top_left())
    }
}

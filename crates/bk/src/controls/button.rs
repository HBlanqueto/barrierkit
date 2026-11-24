use super::*;
use std::borrow::Cow;

pub use ctdesign::ControlSize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonRole {
    Destructive,
}

#[derive(PartialEq)]
pub struct Button {
    label: Cow<'static, str>,
    image: Option<Symbol>,
    content: Option<Callback<(), Element>>,
    disabled: bool,
    on: Option<bool>,
    width: Size,
    on_press: Option<Callback<(), ()>>,
    shortcut: Option<Shortcut>,
    role: Option<ButtonRole>,
    pub(crate) style: Theme,
    key: DiffKey,
    overrides: Overrides,
}

impl Button {
    pub fn new(label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            label: label.into(),
            image: None,
            content: None,
            disabled: false,
            on: None,
            width: Size::auto(),
            on_press: None,
            shortcut: None,
            role: None,
            style: Theme::tengen(),
            key: DiffKey::None,
            overrides: Overrides::default(),
        }
    }

    pub fn system_image(label: impl Into<Cow<'static, str>>, image: Symbol) -> Self {
        Self {
            image: Some(image),
            ..Self::new(label)
        }
    }

    pub fn label<V: IntoElement>(mut self, label: impl Fn() -> V + 'static) -> Self {
        self.content = Some(Callback::new(move |()| label().into_element()));
        self
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

    pub fn keyboard_shortcut(mut self, shortcut: Shortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn role(mut self, role: ButtonRole) -> Self {
        self.role = Some(role);
        self
    }
}

forward_overrides!(Button);

impl KeyExt for Button {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for Button {
    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }

    fn render(&self) -> impl IntoElement {
        let interaction = use_interaction();
        let disabled = use_disabled(self.disabled);
        let enabled = !disabled;
        let style: ButtonStyle = resolve(&self.style);
        let style = match self.role {
            Some(ButtonRole::Destructive) => style.destructive(),
            _ => style,
        };
        let style = if self.on == Some(true) {
            style.toggled()
        } else {
            style
        };
        let state = control_state(interaction, disabled);
        let paints = Paints::current(state, style.effect);
        let pressed = enabled && interaction.pressed();
        let scale = crate::animation::use_animated(
            if pressed { style.pressed_scale } else { 1.0 },
            style.animation,
        );
        let shortcut = self
            .shortcut
            .or(crate::environment::use_environment::<Option<Shortcut>>());
        let on_press = self.on_press.clone();
        let shortcut_press = on_press.clone();
        let radius = style
            .shape
            .corner_radius(style.height, style.height, ambient());
        let opacity = opacity_when(disabled, style.opacity, style.disabled_opacity);
        let font = style.font.scaled(crate::theme::resolved().text_size);
        let glass = style.glass.filter(|_| crate::glass::native_glass());
        // SwiftUI's toolbar shows a button that has an image by its image alone.
        let icon_only = glass.is_some()
            && crate::compact::toolbar_glass()
            && self.image.is_some()
            && self.content.is_none();
        let width = if style.shape == Shape::Circle || icon_only {
            Size::px(style.height)
        } else {
            self.width.clone()
        };
        let padding_x = if icon_only { 0.0 } else { style.padding_x };

        let button = interaction
            .attach(rect(), enabled)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label.clone())
            .width(width.clone())
            .height(Size::px(style.height))
            .padding(Gaps::new(0.0, padding_x, 0.0, padding_x))
            .corner_radius(radius)
            .center()
            .direction(Direction::Horizontal)
            .spacing(ctdesign::SPACE_1)
            .maybe(glass.is_none(), |el| {
                el.painted(&paints, &style.background)
                    .bordered(&paints, style.border.as_ref())
            })
            .maybe(
                glass.is_some_and(|glass| glass.tint.is_none()) && enabled && state.hovered,
                |el| el.background(crate::theme::color(crate::styles::glass_hover())),
            )
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
            .map(shortcut.filter(|_| enabled), |el, shortcut| {
                el.on_global_key_down(move |event: Event<KeyboardEventData>| {
                    if shortcut.matches(event.modifiers, &event.key)
                        && let Some(on_press) = &shortcut_press
                    {
                        on_press.call(());
                    }
                })
            })
            .maybe(self.content.is_some(), |el| {
                text_style(el, font).color(paints.fill(&style.label))
            })
            .maybe_child(self.content.clone().map(|content| {
                TextStyleScope::new(style.font, style.label.resolve(state).clone(), content)
            }))
            .maybe_child(self.image.filter(|_| self.content.is_none()).map(|image| {
                crate::symbol::Image::system(image)
                    .size(if icon_only {
                        ctdesign::ICON_BUTTON.toolbar_icon
                    } else {
                        font.size as f32
                    })
                    .color(paints.color(style.label.resolve(state)))
            }))
            .maybe_child(
                (self.content.is_none() && !self.label.is_empty() && !icon_only).then(|| {
                    with_font(label().text(self.label.clone()), font)
                        .max_lines(1)
                        .color(paints.fill(&style.label))
                }),
            );
        let button = interaction.focus_ring(
            dimmed(shadowed(button, &style.shadows), opacity),
            radius.top_left(),
        );
        self.overrides.apply(match glass {
            Some(glass) => rect().child(crate::glass::GlassBehind {
                radius: radius.top_left(),
                look: glass.look(),
                width,
                child: button.into_element(),
            }),
            None => button,
        })
    }
}

use freya_core::prelude::*;
use torin::prelude::{Gaps, Size};

use crate::controls::{dimmed, inner_border, shadowed};
use crate::elements::forward_container;
use crate::interaction::{control_state, use_interaction};
use crate::style::{text, with_font};
use crate::styles::{
    AvatarStyle, BadgeStyle, ControlState, DividerStyle, LayoutDirection, LinkStyle, Paints, Theme,
    resolve,
};

pub struct Badge(Rect, String, Theme);

impl Badge {
    pub fn new(content: impl Into<String>) -> Self {
        Self::styled(content.into(), Theme::tengen())
    }

    pub fn style(self, restyle: impl Fn(BadgeStyle) -> BadgeStyle + 'static) -> Self {
        Self::styled(self.1, self.2.style(restyle))
    }

    fn styled(content: String, theme: Theme) -> Self {
        let style: BadgeStyle = resolve(&theme);
        let paints = Paints::current(ControlState::default(), style.effect);
        let ink = paints.color(style.label.resolve(ControlState::default()));
        let badge = rect()
            .height(Size::px(style.height))
            .min_width(Size::px(style.height))
            .padding(Gaps::new(0.0, style.padding_x, 0.0, style.padding_x))
            .corner_radius(style.shape.corner_radius(
                style.height,
                style.height,
                LayoutDirection::LeftToRight,
            ))
            .center()
            .background(paints.fill(&style.background))
            .map(paints.blur(&style.background), Rect::blur)
            .map(style.border.as_ref(), |el, border| {
                el.border(inner_border(&paints, border))
            })
            .child(text(content.clone(), style.font, ink));
        Self(shadowed(badge, &style.shadows), content, theme)
    }
}

forward_container!(Badge);

pub struct Divider(Rect, Theme);

impl Divider {
    pub fn new() -> Self {
        Self::styled(Theme::tengen())
    }

    pub fn style(self, restyle: impl Fn(DividerStyle) -> DividerStyle + 'static) -> Self {
        Self::styled(self.1.style(restyle))
    }

    fn styled(theme: Theme) -> Self {
        let style: DividerStyle = resolve(&theme);
        let paints = Paints::current(ControlState::default(), style.effect);
        Self(
            rect()
                .a11y_role(AccessibilityRole::Splitter)
                .width(Size::fill())
                .height(Size::px(style.thickness))
                .background(paints.fill(&style.line)),
            theme,
        )
    }
}

impl Default for Divider {
    fn default() -> Self {
        Self::new()
    }
}

forward_container!(Divider);

pub struct Avatar(Rect, String, Theme);

impl Avatar {
    pub fn new(initials: &str) -> Self {
        Self::styled(initials.to_string(), Theme::tengen())
    }

    pub fn style(self, restyle: impl Fn(AvatarStyle) -> AvatarStyle + 'static) -> Self {
        Self::styled(self.1, self.2.style(restyle))
    }

    fn styled(initials: String, theme: Theme) -> Self {
        let style: AvatarStyle = resolve(&theme);
        let paints = Paints::current(ControlState::default(), style.effect);
        let ink = paints.color(style.label.resolve(ControlState::default()));
        let avatar = rect()
            .a11y_role(AccessibilityRole::Image)
            .a11y_alt(initials.clone())
            .width(Size::px(style.size))
            .height(Size::px(style.size))
            .corner_radius(style.shape.corner_radius(
                style.size,
                style.size,
                LayoutDirection::LeftToRight,
            ))
            .center()
            .background(paints.fill(&style.background))
            .map(paints.blur(&style.background), Rect::blur)
            .map(style.border.as_ref(), |el, border| {
                el.border(inner_border(&paints, border))
            })
            .child(text(initials.clone(), style.font, ink));
        Self(shadowed(avatar, &style.shadows), initials, theme)
    }
}

forward_container!(Avatar);

#[derive(PartialEq)]
pub struct Link {
    text: &'static str,
    on_press: Option<Callback<(), ()>>,
    style: Theme,
}

impl Link {
    pub fn new(text: &'static str) -> Self {
        Self {
            text,
            on_press: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(LinkStyle) -> LinkStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn on_press(mut self, callback: impl Into<Callback<(), ()>>) -> Self {
        self.on_press = Some(callback.into());
        self
    }
}

impl Component for Link {
    fn render(&self) -> impl IntoElement {
        let style: LinkStyle = resolve(&self.style);
        let interaction = use_interaction();
        let paints = Paints::current(control_state(interaction, false), style.effect);
        let on_press = self.on_press.clone();
        let link = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Link)
            .a11y_alt(self.text)
            .map(on_press, |el, on_press| {
                el.on_press(move |_| on_press.call(()))
            })
            .child(
                with_font(label().text(self.text), style.font)
                    .max_lines(1)
                    .color(paints.fill(&style.label)),
            );
        interaction.focus_ring(dimmed(link, style.opacity), 0.0)
    }
}

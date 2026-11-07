use ctdesign::{BADGE, CAPTION, HEADLINE, SETTINGS};
use freya_core::prelude::*;
use torin::prelude::{Gaps, Size};

use crate::controls::dimmed;
use crate::elements::forward_container;
use crate::interaction::{control_state, use_interaction};
use crate::style::{Palette, text, with_font};
use crate::styles::{LinkStyle, Paints, Theme, resolve};
use crate::theme::color;

pub struct Badge(Rect);

impl Badge {
    pub fn new(content: impl Into<String>) -> Self {
        let palette = Palette::current();
        Self(
            rect()
                .height(Size::px(BADGE.height))
                .min_width(Size::px(BADGE.height))
                .padding(Gaps::new(0.0, BADGE.padding_x, 0.0, BADGE.padding_x))
                .corner_radius(BADGE.height / 2.0)
                .center()
                .background(color(palette.destructive))
                .child(text(content.into(), CAPTION, palette.on_accent)),
        )
    }
}

forward_container!(Badge);

pub struct Divider(Rect);

impl Divider {
    pub fn new() -> Self {
        Self(
            rect()
                .a11y_role(AccessibilityRole::Splitter)
                .width(Size::fill())
                .height(Size::px(ctdesign::HAIRLINE))
                .background(color(Palette::current().separator)),
        )
    }
}

impl Default for Divider {
    fn default() -> Self {
        Self::new()
    }
}

forward_container!(Divider);

pub struct Avatar(Rect);

impl Avatar {
    pub fn new(initials: &str) -> Self {
        let palette = Palette::current();
        let size = SETTINGS.account_avatar as f32;
        Self(
            rect()
                .a11y_role(AccessibilityRole::Image)
                .a11y_alt(initials.to_string())
                .width(Size::px(size))
                .height(Size::px(size))
                .corner_radius(size / 2.0)
                .center()
                .background(color(palette.secondary_label))
                .child(text(
                    initials.to_string(),
                    HEADLINE,
                    ctappearance::backdrop::ink_on(palette.secondary_label),
                )),
        )
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

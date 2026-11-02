//! Small read-mostly pieces: badge, divider, link and avatar.

use freya_core::prelude::*;
use torin::prelude::{Gaps, Size};
use vow_design::{BADGE, CAPTION, HEADLINE, SETTINGS, TEXT};

use crate::interaction::use_interaction;
use crate::style::{Palette, text};
use crate::theme::color;

pub fn badge(content: impl Into<String>) -> Rect {
    let palette = Palette::current();
    rect()
        .height(Size::px(BADGE.height))
        .min_width(Size::px(BADGE.height))
        .padding(Gaps::new(0.0, BADGE.padding_x, 0.0, BADGE.padding_x))
        .corner_radius(BADGE.height / 2.0)
        .center()
        .background(color(palette.destructive))
        .child(text(content.into(), CAPTION, palette.on_accent))
}

pub fn divider() -> Rect {
    let palette = Palette::current();
    rect()
        .a11y_role(AccessibilityRole::Splitter)
        .width(Size::fill())
        .height(Size::px(1.0))
        .background(color(palette.separator))
}

pub fn avatar(initials: &str) -> Rect {
    let palette = Palette::current();
    let size = SETTINGS.account_avatar as f32;
    rect()
        .a11y_role(AccessibilityRole::Image)
        .a11y_alt(initials.to_string())
        .width(Size::px(size))
        .height(Size::px(size))
        .corner_radius(size / 2.0)
        .center()
        .background(color(palette.secondary_label))
        .child(text(initials.to_string(), HEADLINE, palette.on_accent))
}

#[derive(PartialEq)]
pub struct Link {
    text: &'static str,
    on_press: Option<Callback<(), ()>>,
}

impl Link {
    pub fn new(text: &'static str) -> Self {
        Self {
            text,
            on_press: None,
        }
    }

    pub fn on_press(mut self, callback: impl Into<Callback<(), ()>>) -> Self {
        self.on_press = Some(callback.into());
        self
    }
}

impl Component for Link {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let on_press = self.on_press.clone();
        let link = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Link)
            .a11y_alt(self.text)
            .on_press(move |_| {
                if let Some(cb) = &on_press {
                    cb.call(());
                }
            })
            .child(text(self.text, TEXT, palette.accent));
        interaction.focus_ring(link, 0.0)
    }
}

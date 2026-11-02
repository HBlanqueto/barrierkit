//! Small read-mostly pieces: badge, divider, link and avatar.

use freya_core::prelude::*;
use torin::prelude::{Gaps, Size};
use vow_design::{BADGE, CAPTION, HEADLINE, SETTINGS, TEXT};

use crate::interaction::use_interaction;
use crate::theme::color;

pub fn badge(text: impl Into<String>) -> Rect {
    let resolved = crate::theme::resolved();
    let base = vow_appearance::palette(resolved.scheme);
    rect()
        .height(Size::px(BADGE.height))
        .min_width(Size::px(BADGE.height))
        .padding(Gaps::new(0.0, BADGE.padding_x, 0.0, BADGE.padding_x))
        .corner_radius(BADGE.height / 2.0)
        .center()
        .background(color(base.danger))
        .child(
            label()
                .text(text.into())
                .font_family(CAPTION.family)
                .font_size(CAPTION.size as f32)
                .font_weight(CAPTION.weight as i32)
                .max_lines(1)
                .color(color(base.on_accent)),
        )
}

pub fn divider() -> Rect {
    let resolved = crate::theme::resolved();
    let base = vow_appearance::palette(resolved.scheme);
    rect()
        .a11y_role(AccessibilityRole::Splitter)
        .width(Size::fill())
        .height(Size::px(1.0))
        .background(color(base.line))
}

pub fn avatar(initials: &str) -> Rect {
    let resolved = crate::theme::resolved();
    let base = vow_appearance::palette(resolved.scheme);
    let size = SETTINGS.account_avatar as f32;
    rect()
        .a11y_role(AccessibilityRole::Image)
        .a11y_alt(initials.to_string())
        .width(Size::px(size))
        .height(Size::px(size))
        .corner_radius(size / 2.0)
        .center()
        .background(color(resolved.accent))
        .child(
            label()
                .text(initials.to_string())
                .font_family(HEADLINE.family)
                .font_size(HEADLINE.size as f32)
                .font_weight(HEADLINE.weight as i32)
                .color(color(base.on_accent)),
        )
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
        let resolved = crate::theme::resolved();
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
            .child(
                label()
                    .text(self.text)
                    .font_family(TEXT.family)
                    .font_size(TEXT.size as f32)
                    .font_weight(TEXT.weight as i32)
                    .color(color(resolved.accent)),
            );
        interaction.focus_ring(link, 0.0)
    }
}

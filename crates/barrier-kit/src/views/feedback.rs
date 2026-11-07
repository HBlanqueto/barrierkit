use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};
use vow_design::{
    DIALOG, DIALOG_ALERT_WIDTH, DIALOG_PADDING, HEADLINE, SETTINGS, SPACE_2, SPACE_4, TEXT,
};

use crate::controls::{Button, ButtonStyle, Checkbox, ControlSize};
use crate::i18n::text;
use crate::style::{Palette, text};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

pub fn empty_state(icon: Symbol, title: &'static str) -> Rect {
    let palette = Palette::current();
    rect()
        .width(Size::fill())
        .direction(Direction::Vertical)
        .cross_align(Alignment::Center)
        .padding(Gaps::new(
            SETTINGS.empty_padding_y as f32,
            0.0,
            SETTINGS.empty_padding_y as f32,
            0.0,
        ))
        .spacing(SETTINGS.empty_gap as f32)
        .child(symbol(
            icon,
            SETTINGS.empty_icon as f32,
            palette.tertiary_label,
        ))
        .child(text(title, TEXT, palette.secondary_label))
}

pub fn sheet(content: impl IntoElement) -> Rect {
    let palette = Palette::current();
    rect()
        .position(Position::new_absolute().left(0.0).top(0.0))
        .width(Size::fill())
        .height(Size::percent(100.0))
        .layer(Layer::Overlay)
        .background(color(palette.scrim))
        .cross_align(Alignment::Center)
        .child(
            rect()
                .a11y_role(AccessibilityRole::Dialog)
                .padding(Gaps::new_all(DIALOG_PADDING))
                .corner_radius(crate::style::continuous_corner(DIALOG.corner_radius as f32))
                .background(color(palette.raised_background))
                .border(
                    Border::new()
                        .width(vow_design::HAIRLINE)
                        .alignment(BorderAlignment::Outer)
                        .fill(color(palette.separator)),
                )
                .child(content),
        )
}

#[derive(PartialEq)]
pub struct Alert {
    title: &'static str,
    message: String,
    confirm: &'static str,
    cancel: &'static str,
    on_confirm: Option<Callback<(), ()>>,
    on_cancel: Option<Callback<(), ()>>,
    suppressed: Option<State<bool>>,
}

impl Alert {
    pub fn new(title: &'static str, message: impl Into<String>) -> Self {
        Self {
            title,
            message: message.into(),
            confirm: text!("ok"),
            cancel: text!("cancel"),
            on_confirm: None,
            on_cancel: None,
            suppressed: None,
        }
    }

    pub fn suppression(mut self, suppressed: State<bool>) -> Self {
        self.suppressed = Some(suppressed);
        self
    }

    pub fn confirm(mut self, label: &'static str, callback: impl Into<Callback<(), ()>>) -> Self {
        self.confirm = label;
        self.on_confirm = Some(callback.into());
        self
    }

    pub fn cancel(mut self, label: &'static str, callback: impl Into<Callback<(), ()>>) -> Self {
        self.cancel = label;
        self.on_cancel = Some(callback.into());
        self
    }
}

impl Component for Alert {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let call = |callback: &Option<Callback<(), ()>>| {
            let callback = callback.clone();
            move || {
                if let Some(cb) = &callback {
                    cb.call(());
                }
            }
        };
        let (confirm, cancel) = (call(&self.on_confirm), call(&self.on_cancel));
        let (on_confirm, on_cancel) = (confirm.clone(), cancel.clone());

        sheet(
            rect()
                .a11y_role(AccessibilityRole::AlertDialog)
                .a11y_alt(self.title)
                .a11y_focusable(true)
                .a11y_auto_focus(true)
                .width(Size::px(DIALOG_ALERT_WIDTH - 2.0 * DIALOG_PADDING))
                .direction(Direction::Vertical)
                .spacing(SPACE_2)
                .on_global_key_down(move |event: Event<KeyboardEventData>| match event.key {
                    Key::Named(NamedKey::Enter) => confirm(),
                    Key::Named(NamedKey::Escape) => cancel(),
                    _ => {}
                })
                .child(text(self.title, HEADLINE, palette.label))
                .child(text(self.message.clone(), TEXT, palette.secondary_label).max_lines(None))
                .maybe_child(self.suppressed.map(|mut suppressed| {
                    Checkbox::new(text!("alert-suppress"))
                        .checked(*suppressed.read())
                        .on_toggle(move |checked| suppressed.set(checked))
                }))
                .child(
                    rect()
                        .width(Size::fill())
                        .direction(Direction::Horizontal)
                        .content(Content::flex())
                        .margin(Gaps::new(SPACE_4, 0.0, 0.0, 0.0))
                        .spacing(SPACE_2)
                        .child(
                            Button::new(self.cancel)
                                .size(ControlSize::Large)
                                .width(Size::flex(1.0))
                                .on_press(move |_| on_cancel()),
                        )
                        .child(
                            Button::new(self.confirm)
                                .style(ButtonStyle::Primary)
                                .size(ControlSize::Large)
                                .width(Size::flex(1.0))
                                .on_press(move |_| on_confirm()),
                        ),
                ),
        )
    }
}

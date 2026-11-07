use ctdesign::{BODY, BUTTON, ICON_BUTTON, SPACE_1};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Size};

use crate::i18n::text;
use crate::interaction::use_interaction;
use crate::style::{Palette, text};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

#[derive(PartialEq)]
pub struct WindowTabs {
    titles: Vec<String>,
    selected: usize,
    on_select: Option<Callback<usize, ()>>,
    on_close: Option<Callback<usize, ()>>,
    on_new: Option<Callback<(), ()>>,
}

impl WindowTabs {
    pub fn new(titles: Vec<String>, selected: usize) -> Self {
        Self {
            titles,
            selected,
            on_select: None,
            on_close: None,
            on_new: None,
        }
    }

    pub fn on_select(mut self, callback: impl Into<Callback<usize, ()>>) -> Self {
        self.on_select = Some(callback.into());
        self
    }

    pub fn on_close(mut self, callback: impl Into<Callback<usize, ()>>) -> Self {
        self.on_close = Some(callback.into());
        self
    }

    pub fn on_new(mut self, callback: impl Into<Callback<(), ()>>) -> Self {
        self.on_new = Some(callback.into());
        self
    }
}

impl Component for WindowTabs {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let tabs = self.titles.iter().enumerate().map(|(index, title)| {
            WindowTab {
                title: title.clone(),
                selected: index == self.selected,
                closable: self.titles.len() > 1,
                on_select: self
                    .on_select
                    .clone()
                    .map(|callback| Callback::new(move |_: ()| callback.call(index))),
                on_close: self
                    .on_close
                    .clone()
                    .map(|callback| Callback::new(move |_: ()| callback.call(index))),
            }
            .into_element()
        });
        let on_new = self.on_new.clone();
        rect()
            .a11y_role(AccessibilityRole::TabList)
            .width(Size::fill())
            .height(Size::px(BUTTON.height + 2.0 * SPACE_1))
            .padding(Gaps::new_all(SPACE_1))
            .direction(Direction::Horizontal)
            .content(Content::flex())
            .cross_align(Alignment::Center)
            .spacing(SPACE_1)
            .background(color(palette.control_background))
            .children(tabs)
            .map(on_new, |el, on_new| {
                el.child(
                    rect()
                        .a11y_role(AccessibilityRole::Button)
                        .a11y_alt(text!("new-tab"))
                        .width(Size::px(ICON_BUTTON.size))
                        .center()
                        .on_press(move |_| on_new.call(()))
                        .child(symbol(
                            Symbol::Plus,
                            ICON_BUTTON.icon,
                            palette.secondary_label,
                        )),
                )
            })
    }
}

#[derive(PartialEq)]
struct WindowTab {
    title: String,
    selected: bool,
    closable: bool,
    on_select: Option<Callback<(), ()>>,
    on_close: Option<Callback<(), ()>>,
}

impl Component for WindowTab {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let show_close = self.closable && (interaction.hovered() || self.selected);
        let background = if self.selected {
            Some(palette.raised_background)
        } else if interaction.hovered() {
            Some(palette.control_hover)
        } else {
            None
        };
        let on_select = self.on_select.clone();
        let on_close = self.on_close.clone();
        let selected = self.selected;
        let tab = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Tab)
            .a11y_alt(self.title.clone())
            .a11y_builder(move |node| node.set_selected(selected))
            .width(Size::flex(1.0))
            .height(Size::px(BUTTON.height))
            .corner_radius(BUTTON.height / 2.0)
            .maybe(background.is_some(), |el| {
                el.background(color(background.unwrap_or(palette.control_background)))
            })
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .padding(Gaps::new(0.0, SPACE_1, 0.0, SPACE_1))
            .content(Content::flex())
            .map(on_select, |el, on_select| {
                el.on_press(move |_| on_select.call(()))
            })
            .child(
                rect()
                    .width(Size::px(ICON_BUTTON.icon))
                    .center()
                    .maybe(show_close, |el| {
                        el.a11y_role(AccessibilityRole::Button)
                            .a11y_alt(text!("close-tab"))
                            .map(on_close, |el, on_close| {
                                el.on_press(move |event: Event<PressEventData>| {
                                    event.stop_propagation();
                                    on_close.call(());
                                })
                            })
                            .child(symbol(
                                Symbol::XMark,
                                ICON_BUTTON.icon,
                                palette.secondary_label,
                            ))
                    }),
            )
            .child(
                rect()
                    .width(Size::flex(1.0))
                    .main_align(Alignment::Center)
                    .direction(Direction::Horizontal)
                    .child(text(self.title.clone(), BODY, palette.label).max_lines(1)),
            )
            .child(rect().width(Size::px(ICON_BUTTON.icon)));
        interaction.focus_ring(tab, BUTTON.height / 2.0)
    }
}

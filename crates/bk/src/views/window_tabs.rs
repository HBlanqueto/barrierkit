use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Size};

use crate::i18n::text;
use crate::interaction::{control_state, use_interaction};
use crate::style::text;
use crate::styles::Painted;
use crate::styles::{ControlState, LayoutDirection, Paints, Theme, WindowTabsStyle, resolve};
use crate::symbol::{Symbol, symbol};

#[derive(PartialEq)]
pub struct WindowTabs {
    titles: Vec<String>,
    selected: usize,
    on_select: Option<Callback<usize, ()>>,
    on_close: Option<Callback<usize, ()>>,
    on_new: Option<Callback<(), ()>>,
    style: Theme,
}

impl WindowTabs {
    pub fn new(titles: Vec<String>, selected: usize) -> Self {
        Self {
            titles,
            selected,
            on_select: None,
            on_close: None,
            on_new: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(WindowTabsStyle) -> WindowTabsStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
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
        let style: WindowTabsStyle = resolve(&self.style);
        let state = ControlState::default();
        let paints = Paints::current(state, style.effect);
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
                style: self.style.clone(),
            }
            .into_element()
        });
        let on_new = self.on_new.clone();
        rect()
            .a11y_role(AccessibilityRole::TabList)
            .width(Size::fill())
            .height(Size::px(style.tab_height + 2.0 * style.padding))
            .padding(Gaps::new_all(style.padding))
            .direction(Direction::Horizontal)
            .content(Content::flex())
            .cross_align(Alignment::Center)
            .spacing(style.spacing)
            .painted(&paints, &style.background)
            .children(tabs)
            .map(on_new, |el, on_new| {
                el.child(
                    rect()
                        .a11y_role(AccessibilityRole::Button)
                        .a11y_alt(text!("new-tab"))
                        .width(Size::px(style.new_tab_width))
                        .center()
                        .on_press(move |_| on_new.call(()))
                        .child(symbol(
                            Symbol::Plus,
                            style.symbol_size,
                            paints.color(style.symbol.resolve(state)),
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
    style: Theme,
}

impl Component for WindowTab {
    fn render(&self) -> impl IntoElement {
        let style: WindowTabsStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state, style.effect);
        let show_close = self.closable && (interaction.hovered() || self.selected);
        let background = if self.selected {
            &style.selected_tab_background
        } else {
            &style.tab_background
        };
        let radius = style.tab_shape.corner_radius(
            style.tab_height,
            style.tab_height,
            LayoutDirection::LeftToRight,
        );
        let on_select = self.on_select.clone();
        let on_close = self.on_close.clone();
        let selected = self.selected;
        let tab = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Tab)
            .a11y_alt(self.title.clone())
            .a11y_builder(move |node| node.set_selected(selected))
            .width(Size::flex(1.0))
            .height(Size::px(style.tab_height))
            .corner_radius(radius)
            .painted(&paints, background)
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .padding(Gaps::new(
                0.0,
                style.tab_padding_x,
                0.0,
                style.tab_padding_x,
            ))
            .content(Content::flex())
            .map(on_select, |el, on_select| {
                el.on_press(move |_| on_select.call(()))
            })
            .child(
                rect()
                    .width(Size::px(style.symbol_size))
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
                                style.symbol_size,
                                paints.color(style.symbol.resolve(state)),
                            ))
                    }),
            )
            .child(
                rect()
                    .width(Size::flex(1.0))
                    .main_align(Alignment::Center)
                    .direction(Direction::Horizontal)
                    .child(
                        text(
                            self.title.clone(),
                            style.font,
                            paints.color(style.label.resolve(state)),
                        )
                        .max_lines(1),
                    ),
            )
            .child(rect().width(Size::px(style.symbol_size)));
        interaction.focus_ring(tab, radius.top_left())
    }
}

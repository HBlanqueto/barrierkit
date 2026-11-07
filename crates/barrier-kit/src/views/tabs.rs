use ct_design::{CAPTION, SPACE_2, TOOLBAR_TABS};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Gaps, Size};

use crate::choices::use_choice;
use crate::interaction::use_interaction;
use crate::style::Palette;
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

#[derive(PartialEq)]
pub struct TabView {
    label: &'static str,
    tabs: Vec<(Symbol, &'static str, Element)>,
    selected: usize,
    on_change: Option<Callback<usize, ()>>,
}

impl TabView {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            tabs: Vec::new(),
            selected: 0,
            on_change: None,
        }
    }

    pub fn tab(mut self, icon: Symbol, title: &'static str, content: impl IntoElement) -> Self {
        self.tabs.push((icon, title, content.into_element()));
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    pub fn on_change(mut self, callback: impl Into<Callback<usize, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }
}

impl Component for TabView {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let choice = use_choice(Some(self.selected), self.on_change.clone());
        let current = (0..self.tabs.len()).find(|&i| choice.is(i)).unwrap_or(0);
        let tabs = self
            .tabs
            .iter()
            .enumerate()
            .map(|(index, (icon, title, _))| {
                ToolbarTab {
                    icon: *icon,
                    title,
                    active: index == current,
                    on_select: choice.select(index),
                }
                .into_element()
            });

        rect()
            .expanded()
            .direction(Direction::Vertical)
            .spacing(SPACE_2)
            .child(
                rect()
                    .a11y_role(AccessibilityRole::TabList)
                    .a11y_alt(self.label)
                    .width(Size::fill())
                    .direction(Direction::Horizontal)
                    .main_align(Alignment::Center)
                    .padding(Gaps::new_all(TOOLBAR_TABS.padding))
                    .spacing(TOOLBAR_TABS.gap)
                    .corner_radius(TOOLBAR_TABS.radius)
                    .background(color(palette.sunken_background))
                    .border(
                        Border::new()
                            .width(ct_design::HAIRLINE)
                            .alignment(BorderAlignment::Inner)
                            .fill(color(palette.separator)),
                    )
                    .children(tabs),
            )
            .maybe_child(self.tabs.get(current).map(|tab| tab.2.clone()))
    }
}

#[derive(PartialEq)]
struct ToolbarTab {
    icon: Symbol,
    title: &'static str,
    active: bool,
    on_select: Callback<(), ()>,
}

impl Component for ToolbarTab {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let active = self.active;
        let on_select = self.on_select.clone();
        let ink = if active {
            palette.accent
        } else {
            palette.secondary_label
        };
        let background = if active {
            Some(palette.control_hover)
        } else if interaction.hovered() {
            Some(palette.control_background)
        } else {
            None
        };

        let tab = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Tab)
            .a11y_alt(self.title)
            .a11y_builder(move |node| node.set_selected(active))
            .min_width(Size::px(TOOLBAR_TABS.tab_min_width))
            .padding(Gaps::new(
                TOOLBAR_TABS.tab_padding_top,
                0.0,
                TOOLBAR_TABS.tab_padding_bottom,
                0.0,
            ))
            .direction(Direction::Vertical)
            .cross_align(Alignment::Center)
            .spacing(TOOLBAR_TABS.icon_gap)
            .corner_radius(TOOLBAR_TABS.tab_radius)
            .map(background, |el, background| {
                el.background(color(background))
            })
            .on_press(move |_| on_select.call(()))
            .child(symbol(self.icon, TOOLBAR_TABS.icon, ink))
            .child(
                label()
                    .text(self.title)
                    .font_family(CAPTION.family)
                    .font_size(TOOLBAR_TABS.font_size)
                    .font_weight(TOOLBAR_TABS.font_weight)
                    .max_lines(1)
                    .color(color(ink)),
            );
        interaction.focus_ring(tab, TOOLBAR_TABS.tab_radius)
    }
}

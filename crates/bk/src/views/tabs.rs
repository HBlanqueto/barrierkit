use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Gaps, Size};

use crate::choices::use_choice;
use crate::controls::inner_border;
use crate::interaction::{control_state, use_interaction};
use crate::styles::Painted;
use crate::styles::{ControlState, LayoutDirection, Paints, TabViewStyle, Theme, resolve};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

#[derive(PartialEq)]
pub struct TabView {
    label: &'static str,
    tabs: Vec<(Symbol, &'static str, Element)>,
    selected: usize,
    on_change: Option<Callback<usize, ()>>,
    style: Theme,
}

fn tab_height(style: &TabViewStyle) -> f32 {
    style.tab_padding.0
        + style.icon_size
        + style.icon_spacing
        + style.font.line_height as f32
        + style.tab_padding.1
}

impl TabView {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            tabs: Vec::new(),
            selected: 0,
            on_change: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(TabViewStyle) -> TabViewStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
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
        let style: TabViewStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default(), style.effect);
        let bar_height = tab_height(&style) + 2.0 * style.padding;
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
                    style: self.style.clone(),
                }
                .into_element()
            });

        rect()
            .expanded()
            .direction(Direction::Vertical)
            .spacing(style.content_spacing)
            .child(
                rect()
                    .a11y_role(AccessibilityRole::TabList)
                    .a11y_alt(self.label)
                    .width(Size::fill())
                    .direction(Direction::Horizontal)
                    .main_align(Alignment::Center)
                    .padding(Gaps::new_all(style.padding))
                    .spacing(style.spacing)
                    .corner_radius(style.shape.corner_radius(
                        bar_height,
                        bar_height,
                        LayoutDirection::LeftToRight,
                    ))
                    .painted(&paints, &style.background)
                    .map(style.border.as_ref(), |el, border| {
                        el.border(inner_border(&paints, border))
                    })
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
    style: Theme,
}

impl Component for ToolbarTab {
    fn render(&self) -> impl IntoElement {
        let style: TabViewStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state, style.effect);
        let active = self.active;
        let on_select = self.on_select.clone();
        let (background, ink) = if active {
            (&style.selected_tab_background, &style.selected_label)
        } else {
            (&style.tab_background, &style.label)
        };
        let ink = paints.color(ink.resolve(state));
        let height = tab_height(&style);
        let radius = style
            .tab_shape
            .corner_radius(height, height, LayoutDirection::LeftToRight);

        let tab = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Tab)
            .a11y_alt(self.title)
            .a11y_builder(move |node| node.set_selected(active))
            .min_width(Size::px(style.tab_min_width))
            .padding(Gaps::new(
                style.tab_padding.0,
                0.0,
                style.tab_padding.1,
                0.0,
            ))
            .direction(Direction::Vertical)
            .cross_align(Alignment::Center)
            .spacing(style.icon_spacing)
            .corner_radius(radius)
            .painted(&paints, background)
            .on_press(move |_| on_select.call(()))
            .child(symbol(self.icon, style.icon_size, ink))
            .child(
                label()
                    .text(self.title)
                    .font_family(style.font.family)
                    .font_size(style.font.size as f32)
                    .font_weight(style.font.weight as i32)
                    .max_lines(1)
                    .color(color(ink)),
            );
        interaction.focus_ring(tab, radius.top_left)
    }
}

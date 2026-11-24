use ctdesign::{ControlSize, TAB_VIEW};
use freya_core::prelude::*;
use std::borrow::Cow;
use torin::prelude::{Alignment, Direction, Gaps, Position, Size};

use crate::choices::Choice;
use crate::controls::shadowed;
use crate::elements::{Overrides, forward_overrides};
use crate::interaction::{control_state, use_interaction};
use crate::menu::measure_text;
use crate::style::text_style;
use crate::styles::Painted;
use crate::styles::{ControlState, LayoutDirection, Paints, TabType, TabViewStyle, Theme, resolve};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

#[derive(PartialEq)]
pub struct TabView {
    label: Cow<'static, str>,
    tabs: Vec<(Symbol, Cow<'static, str>, Element)>,
    selection: Option<State<usize>>,
    style: Theme,
    overrides: Overrides,
}

fn tab_height(style: &TabViewStyle) -> f32 {
    style.tab_padding.0
        + style.icon_size
        + style.icon_spacing
        + style.font.line_height as f32
        + style.tab_padding.1
}

impl TabView {
    pub fn new(label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            label: label.into(),
            tabs: Vec::new(),
            selection: None,
            style: Theme::tengen(),
            overrides: Overrides::default(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(TabViewStyle) -> TabViewStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn size(self, size: ControlSize) -> Self {
        self.style(move |style| style.sized(size))
    }

    pub fn tab(
        mut self,
        icon: Symbol,
        title: impl Into<Cow<'static, str>>,
        content: impl IntoElement,
    ) -> Self {
        self.tabs.push((icon, title.into(), content.into_element()));
        self
    }

    pub fn selection(mut self, selection: State<usize>) -> Self {
        self.selection = Some(selection);
        self
    }
}

forward_overrides!(TabView);

impl Component for TabView {
    fn render(&self) -> impl IntoElement {
        let style: TabViewStyle = resolve(&self.style);
        let own = use_state(|| 0);
        let choice = Choice(self.selection.unwrap_or(own));
        let current = (0..self.tabs.len()).find(|&i| choice.is(i)).unwrap_or(0);
        let page = self.tabs.get(current).map(|tab| tab.2.clone());
        let root = match style.tab_type {
            TabType::Toolbar => self.toolbar(&style, &choice, current, page),
            tab_type => self.boxed(&style, &choice, current, page, tab_type),
        };
        self.overrides.apply(root)
    }
}

impl TabView {
    fn toolbar(
        &self,
        style: &TabViewStyle,
        choice: &Choice,
        current: usize,
        page: Option<Element>,
    ) -> Rect {
        let paints = Paints::current(ControlState::default());
        let bar_height = tab_height(style) + 2.0 * style.padding;
        let tabs = self
            .tabs
            .iter()
            .enumerate()
            .map(|(index, (icon, title, _))| {
                ToolbarTab {
                    icon: *icon,
                    title: title.clone(),
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
                    .a11y_alt(&*self.label)
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
                    .bordered(&paints, style.border.as_ref())
                    .children(tabs),
            )
            .maybe_child(page)
    }

    fn boxed(
        &self,
        style: &TabViewStyle,
        choice: &Choice,
        current: usize,
        page: Option<Element>,
        tab_type: TabType,
    ) -> Rect {
        let paints = Paints::current(ControlState::default());
        let [far, near, side] = [
            TAB_VIEW.box_inset_far,
            TAB_VIEW.box_inset_near,
            TAB_VIEW.box_inset_x,
        ];
        let page_in = TAB_VIEW.page_inset;
        let half = style.strip_height / 2.0;
        let past = style.strip_inset + style.strip_height + style.page_gap;
        let (frame, content) = match tab_type {
            TabType::Top => (
                Gaps::new(style.strip_inset + half, side, far, side),
                Gaps::new(past, side + page_in, far + page_in, side + page_in),
            ),
            TabType::Bottom => (
                Gaps::new(near, side, style.strip_inset + half + 1.0, side),
                Gaps::new(near + page_in, side + page_in, past - 2.0, side + page_in),
            ),
            TabType::NoTabsBezelBorder => {
                let [left, top, right, bottom] = TAB_VIEW.bezel_inset;
                let [page_left, page_top, page_right, page_bottom] = TAB_VIEW.bezel_page_inset;
                (
                    Gaps::new(top, right, bottom, left),
                    Gaps::new(page_top, page_right, page_bottom, page_left),
                )
            }
            TabType::NoTabsLineBorder => (Gaps::new_all(0.0), Gaps::new_all(1.0)),
            TabType::Toolbar | TabType::NoTabsNoBorder => (Gaps::new_all(0.0), Gaps::new_all(0.0)),
        };
        let (background, border, radius) = match tab_type {
            TabType::NoTabsLineBorder => {
                (None, style.line_border.as_ref(), CornerRadius::new_all(0.0))
            }
            TabType::NoTabsNoBorder => (None, None, CornerRadius::new_all(0.0)),
            _ => (
                Some(&style.box_background),
                style.box_border.as_ref(),
                style
                    .box_shape
                    .corner_radius(0.0, 0.0, LayoutDirection::LeftToRight),
            ),
        };
        let draws_box = tab_type != TabType::NoTabsNoBorder;
        let tabs = self.tabs.iter().enumerate().map(|(index, (_, title, _))| {
            StripTab {
                title: title.clone(),
                active: index == current,
                on_select: choice.select(index),
                style: self.style.clone(),
            }
            .into_element()
        });
        let has_strip = matches!(tab_type, TabType::Top | TabType::Bottom);
        let strip_radius = style.shape.corner_radius(
            style.strip_height,
            style.strip_height,
            LayoutDirection::LeftToRight,
        );
        let strip = rect()
            .a11y_role(AccessibilityRole::TabList)
            .a11y_alt(&*self.label)
            .direction(Direction::Horizontal)
            .corner_radius(strip_radius)
            .painted(&paints, &style.background)
            .children(tabs);
        let edge = if tab_type == TabType::Top {
            Position::new_absolute().top(style.strip_inset).left(0.0)
        } else {
            Position::new_absolute().bottom(style.strip_inset).left(0.0)
        };
        let strip = rect()
            .position(edge)
            .layer(2)
            .width(Size::percent(100.0))
            .direction(Direction::Horizontal)
            .main_align(Alignment::Center)
            .child(strip);
        let fill = |padding: Gaps, child: Rect| {
            rect()
                .position(Position::new_absolute().left(0.0).top(0.0))
                .width(Size::percent(100.0))
                .height(Size::percent(100.0))
                .padding(padding)
                .child(child)
        };
        rect()
            .expanded()
            .maybe_child(draws_box.then(|| {
                fill(
                    frame,
                    rect()
                        .expanded()
                        .corner_radius(radius)
                        .map(background, |el, background| el.painted(&paints, background))
                        .bordered(&paints, border),
                )
            }))
            .child(fill(content, rect().expanded().maybe_child(page)))
            .maybe_child(has_strip.then_some(strip))
    }
}

#[derive(PartialEq)]
struct ToolbarTab {
    icon: Symbol,
    title: Cow<'static, str>,
    active: bool,
    on_select: Callback<(), ()>,
    style: Theme,
}

impl Component for ToolbarTab {
    fn render(&self) -> impl IntoElement {
        let style: TabViewStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state);
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
            .a11y_alt(&*self.title)
            .a11y_builder(move |node| node.set_selected(active))
            .min_width(Size::px(style.tab_min_width))
            .padding(Gaps::new(
                style.tab_padding.0,
                style.tab_padding_x,
                style.tab_padding.1,
                style.tab_padding_x,
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
                    .text(self.title.clone())
                    .with(|el| text_style(el, style.font))
                    .max_lines(1)
                    .color(color(ink)),
            );
        interaction.focus_ring(tab, radius.top_left())
    }
}

#[derive(PartialEq)]
struct StripTab {
    title: Cow<'static, str>,
    active: bool,
    on_select: Callback<(), ()>,
    style: Theme,
}

impl Component for StripTab {
    fn render(&self) -> impl IntoElement {
        let style: TabViewStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state);
        let active = self.active;
        let on_select = self.on_select.clone();
        let (background, ink) = if active {
            (&style.selected_tab_background, &style.selected_label)
        } else {
            (&style.tab_background, &style.label)
        };
        let ink = paints.color(ink.resolve(state));
        let along = measure_text(&self.title, style.font.size as f32) + 2.0 * style.tab_padding_x;
        let across = style.strip_height;
        let (inset_x, inset_y) = style.tab_inset;
        let pill_height = across - 2.0 * inset_y;
        let radius =
            style
                .tab_shape
                .corner_radius(pill_height, pill_height, LayoutDirection::LeftToRight);
        let pill = rect()
            .expanded()
            .corner_radius(radius)
            .center()
            .painted(&paints, background)
            .child(
                label()
                    .text(self.title.clone())
                    .with(|el| text_style(el, style.font))
                    .max_lines(1)
                    .color(color(ink)),
            );
        let pill = if active {
            shadowed(pill, &style.tab_shadows)
        } else {
            pill
        };
        let face = rect()
            .width(Size::px(along))
            .height(Size::px(across))
            .padding(Gaps::new(inset_y, inset_x, inset_y, inset_x))
            .child(pill);
        let cell = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Tab)
            .a11y_alt(&*self.title)
            .a11y_builder(move |node| node.set_selected(active))
            .on_press(move |_| on_select.call(()));
        let cell = cell
            .width(Size::px(along))
            .height(Size::px(across))
            .child(face);
        interaction.focus_ring(cell, radius.top_left())
    }
}

use ctdesign::{ControlSize, TAB_VIEW};
use freya_core::prelude::*;
use std::borrow::Cow;
use torin::prelude::{Alignment, Direction, Gaps, Position, Size};

use crate::choices::{Choice, use_choice};
use crate::controls::shadowed;
use crate::elements::{Overrides, forward_overrides};
use crate::interaction::{control_state, use_interaction};
use crate::menu::measure_text;
use crate::styles::Painted;
use crate::styles::{ControlState, LayoutDirection, Paints, TabType, TabViewStyle, Theme, resolve};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

#[derive(PartialEq)]
pub struct TabView {
    label: Cow<'static, str>,
    tabs: Vec<(Symbol, Cow<'static, str>, Element)>,
    selected: usize,
    on_change: Option<Callback<usize, ()>>,
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
            selected: 0,
            on_change: None,
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

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    pub fn on_change(mut self, callback: impl Into<Callback<usize, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }
}

forward_overrides!(TabView);

impl Component for TabView {
    fn render(&self) -> impl IntoElement {
        let style: TabViewStyle = resolve(&self.style);
        let choice = use_choice(Some(self.selected), self.on_change.clone());
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
        let paints = Paints::current(ControlState::default(), style.effect);
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
        let paints = Paints::current(ControlState::default(), style.effect);
        let [far, near, side] = [
            TAB_VIEW.box_inset_far,
            TAB_VIEW.box_inset_near,
            TAB_VIEW.box_inset_x,
        ];
        let page_in = 3.0;
        let half = style.strip_height / 2.0;
        let past = style.strip_inset + style.strip_height + style.page_gap;
        // Gaps::new is (top, right, bottom, left).
        let (frame, content) = match tab_type {
            TabType::Top => (
                Gaps::new(style.strip_inset + half, side, far, side),
                Gaps::new(past, side + page_in, far + page_in, side + page_in),
            ),
            TabType::Bottom => (
                Gaps::new(near, side, style.strip_inset + half + 1.0, side),
                Gaps::new(near + page_in, side + page_in, past - 2.0, side + page_in),
            ),
            TabType::Left => (
                Gaps::new(near, side, far, style.strip_inset + half),
                Gaps::new(near + page_in, side + page_in, far + page_in, past - 1.0),
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
                vertical: tab_type == TabType::Left,
                on_select: choice.select(index),
                style: self.style.clone(),
            }
            .into_element()
        });
        let has_strip = matches!(tab_type, TabType::Top | TabType::Bottom | TabType::Left);
        let strip_radius = style.shape.corner_radius(
            style.strip_height,
            style.strip_height,
            LayoutDirection::LeftToRight,
        );
        let strip = rect()
            .a11y_role(AccessibilityRole::TabList)
            .a11y_alt(&*self.label)
            .direction(if tab_type == TabType::Left {
                Direction::Vertical
            } else {
                Direction::Horizontal
            })
            .corner_radius(strip_radius)
            .painted(&paints, &style.background)
            .children(tabs);
        let strip = match tab_type {
            TabType::Top | TabType::Bottom => {
                let edge = if tab_type == TabType::Top {
                    Position::new_absolute().top(style.strip_inset).left(0.0)
                } else {
                    Position::new_absolute().bottom(style.strip_inset).left(0.0)
                };
                rect()
                    .position(edge)
                    .layer(2)
                    .width(Size::percent(100.0))
                    .direction(Direction::Horizontal)
                    .main_align(Alignment::Center)
                    .child(strip)
            }
            _ => rect()
                .position(
                    Position::new_absolute()
                        .left(style.strip_inset)
                        .top(TAB_VIEW.left_strip_top),
                )
                .layer(2)
                .child(strip),
        };
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
                    .font_family(style.font.family)
                    .font_size(style.font.size as f32)
                    .font_weight(style.font.weight as i32)
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
    vertical: bool,
    on_select: Callback<(), ()>,
    style: Theme,
}

impl Component for StripTab {
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
        let along = measure_text(&self.title, style.font.size as f32)
            + 2.0
                * if self.vertical {
                    style.tab_padding_y
                } else {
                    style.tab_padding_x
                };
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
                    .font_family(style.font.family)
                    .font_size(style.font.size as f32)
                    .font_weight(style.font.weight as i32)
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
        let cell = if self.vertical {
            cell.width(Size::px(across)).height(Size::px(along)).child(
                face.position(
                    Position::new_absolute()
                        .left((across - along) / 2.0)
                        .top((along - across) / 2.0),
                )
                .rotate(-90.0),
            )
        } else {
            cell.width(Size::px(along))
                .height(Size::px(across))
                .child(face)
        };
        interaction.focus_ring(cell, radius.top_left())
    }
}

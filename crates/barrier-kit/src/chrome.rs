//! Vertical titlebars for client-side windows: rail, unified, pill, rail with
//! actions and reactive edge, on the left or the right.

use freya_animation::prelude::{AnimNum, Ease, Function, use_animation_transition};
use freya_core::prelude::*;
use serde::{Deserialize, Serialize};
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};
use vow_appearance::{SidebarStyle, palette};
use vow_design::{DURATION_BASE, Rgba, TITLEBAR, TITLEBAR_RAIL};

use crate::platform::WindowControls;
use crate::style::Palette;
use crate::theme::{SurfaceKind, color, surface_background};
use crate::titlebar::{window_drag_handler, window_traffic_lights};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TitlebarLayout {
    #[default]
    Top,
    Rail,
    Unified,
    Pill,
    Actions,
    Reactive,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    #[default]
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestingEdge {
    #[default]
    None,
    Line,
    Accent,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChromeStyle {
    pub layout: TitlebarLayout,
    pub side: Side,
    pub rail_width: f32,
    pub round_lights: bool,
    pub resting_edge: RestingEdge,
}

impl Default for ChromeStyle {
    fn default() -> Self {
        Self {
            layout: TitlebarLayout::Top,
            side: Side::Left,
            rail_width: TITLEBAR_RAIL.width,
            round_lights: false,
            resting_edge: RestingEdge::None,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Chrome {
    style: State<ChromeStyle>,
    sidebar_shown: State<bool>,
}

fn chrome() -> Option<Chrome> {
    try_consume_context()
}

pub(crate) fn chrome_style() -> ChromeStyle {
    chrome()
        .map(|chrome| *chrome.style.read())
        .unwrap_or_default()
}

pub(crate) fn report_sidebar_shown(shown: bool) {
    if let Some(mut chrome) = chrome() {
        chrome.sidebar_shown.set_if_modified(shown);
    }
}

#[derive(PartialEq)]
pub struct WindowChrome {
    style: ChromeStyle,
    footer: Option<Element>,
    child: Element,
}

impl WindowChrome {
    pub fn new(style: ChromeStyle, child: impl IntoElement) -> Self {
        Self {
            style,
            footer: None,
            child: child.into_element(),
        }
    }

    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_element());
        self
    }
}

impl Component for WindowChrome {
    fn render(&self) -> impl IntoElement {
        let chrome = use_hook(|| {
            let chrome = Chrome {
                style: State::create(self.style),
                sidebar_shown: State::create(false),
            };
            provide_context(chrome);
            chrome
        });
        let mut style_state = chrome.style;
        style_state.set_if_modified(self.style);
        let style = self.style;
        let sidebar_shown = *chrome.sidebar_shown.read();
        let mut revealed = use_state(|| false);
        let reduced_motion = crate::theme::resolved().reduced_motion;
        let reveal = use_animation_transition(revealed, move |from: bool, to: bool| {
            AnimNum::new(f32::from(u8::from(from)), f32::from(u8::from(to)))
                .time(if reduced_motion { 0 } else { DURATION_BASE })
                .ease(Ease::Out)
                .function(Function::Cubic)
        });

        let layout = match style.layout {
            TitlebarLayout::Unified if !sidebar_shown => TitlebarLayout::Rail,
            layout => layout,
        };
        let pill_gutter = layout == TitlebarLayout::Pill && !sidebar_shown;
        let gutter = if pill_gutter {
            TITLEBAR_RAIL.pill_clearance
        } else {
            0.0
        };
        let content = rect()
            .width(Size::flex(1.0))
            .height(Size::fill())
            .padding(match style.side {
                Side::Left => Gaps::new(0.0, 0.0, 0.0, gutter),
                Side::Right => Gaps::new(0.0, gutter, 0.0, 0.0),
            })
            .child(self.child.clone());

        let edge = match layout {
            TitlebarLayout::Rail => Some(rail(style, None).into_element()),
            TitlebarLayout::Actions => Some(rail(style, self.footer.clone()).into_element()),
            TitlebarLayout::Reactive => Some(resting_edge(style.resting_edge).into_element()),
            _ => None,
        };
        let mut root = rect()
            .width(Size::fill())
            .height(Size::fill())
            .direction(Direction::Horizontal)
            .content(Content::flex());
        root = match style.side {
            Side::Left => root.maybe_child(edge).child(content),
            Side::Right => root.child(content).maybe_child(edge),
        };
        if layout == TitlebarLayout::Pill {
            root = root.child(pill(style.side));
        }
        if layout == TitlebarLayout::Reactive {
            let shown = reveal.get().value();
            root = root.child(
                at_side(
                    style.side,
                    rect().width(Size::px(TITLEBAR_RAIL.reveal_zone)),
                )
                .height(Size::fill())
                .layer(Layer::Overlay)
                .on_pointer_enter(move |_| revealed.set_if_modified(true)),
            );
            if shown > 0.0 {
                root = root.child(
                    at_side(
                        style.side,
                        rect()
                            .width(Size::px(style.rail_width * shown))
                            .overflow(Overflow::Clip),
                    )
                    .height(Size::fill())
                    .layer(Layer::Overlay)
                    .on_pointer_leave(move |_| revealed.set_if_modified(false))
                    .child(rail(style, None)),
                );
            }
        }
        root
    }
}

fn at_side(side: Side, el: Rect) -> Rect {
    let position = Position::new_absolute().top(0.0);
    el.position(match side {
        Side::Left => position.left(0.0),
        Side::Right => position.right(0.0),
    })
}

fn window_state() -> (bool, bool, String) {
    try_consume_root_context::<WindowControls>()
        .map(|controls| {
            (
                controls.status.read().activated,
                controls.fixed,
                controls.title.clone(),
            )
        })
        .unwrap_or((true, false, String::new()))
}

fn rail(style: ChromeStyle, footer: Option<Element>) -> Rect {
    let resolved = crate::theme::resolved();
    let (activated, fixed, title) = window_state();
    let width = if footer.is_some() {
        TITLEBAR_RAIL.actions_width
    } else {
        style.rail_width
    };
    let island = resolved.sidebar.style == SidebarStyle::Island;
    let background = if island {
        Rgba::rgba8(0, 0, 0, 0.0)
    } else if activated {
        surface_background(&resolved, SurfaceKind::Toplevel)
    } else {
        resolved.surfaces.titlebar_inactive
    };
    let title_ink = if activated {
        palette(resolved.scheme).title.focused
    } else {
        palette(resolved.scheme).title.unfocused
    };
    let line = (!island).then(|| {
        at_side(
            match style.side {
                Side::Left => Side::Right,
                Side::Right => Side::Left,
            },
            rect().width(Size::px(1.0)),
        )
        .height(Size::fill())
        .background(color(resolved.sidebar.palette.line))
    });
    rect()
        .width(Size::px(width))
        .height(Size::fill())
        .background(color(background))
        .direction(Direction::Vertical)
        .content(Content::flex())
        .cross_align(Alignment::Center)
        .spacing(TITLEBAR.buttons.spacing as f32)
        .padding(Gaps::new(
            TITLEBAR_RAIL.padding_top,
            0.0,
            TITLEBAR_RAIL.padding_bottom,
            0.0,
        ))
        .child(window_traffic_lights(None, activated, fixed).vertical(true))
        .child(RotatedTitle {
            title,
            ink: title_ink,
            side: style.side,
            fixed,
        })
        .maybe_child(footer)
        .maybe_child(line)
}

#[derive(PartialEq)]
struct RotatedTitle {
    title: String,
    ink: Rgba,
    side: Side,
    fixed: bool,
}

impl Component for RotatedTitle {
    fn render(&self) -> impl IntoElement {
        let mut area = use_state(|| (0.0f32, 0.0f32));
        let (width, height) = *area.read();
        let angle = match self.side {
            Side::Left => -90.0,
            Side::Right => 90.0,
        };
        rect()
            .width(Size::fill())
            .height(Size::flex(1.0))
            .on_sized(move |event: Event<SizedEventData>| {
                let size = event.area.size;
                area.set_if_modified((size.width, size.height));
            })
            .on_mouse_down(window_drag_handler(self.fixed))
            .child(
                rect()
                    .position(
                        Position::new_absolute()
                            .left((width - height) / 2.0)
                            .top((height - width) / 2.0),
                    )
                    .width(Size::px(height))
                    .height(Size::px(width))
                    .rotate(angle)
                    .center()
                    .child(
                        label()
                            .text(self.title.clone())
                            .font_family(TITLEBAR.font.family)
                            .font_size(TITLEBAR_RAIL.title_size)
                            .font_weight(FontWeight::from(TITLEBAR.font.weight as i32))
                            .color(color(self.ink))
                            .max_lines(1)
                            .text_overflow(TextOverflow::Ellipsis),
                    ),
            )
    }
}

fn resting_edge(edge: RestingEdge) -> Rect {
    let resolved = crate::theme::resolved();
    let (width, fill) = match edge {
        RestingEdge::None => (0.0, Rgba::rgba8(0, 0, 0, 0.0)),
        RestingEdge::Line => (TITLEBAR_RAIL.edge_line, resolved.sidebar.palette.line),
        RestingEdge::Accent => (TITLEBAR_RAIL.edge_accent, Palette::current().accent),
    };
    rect()
        .width(Size::px(width))
        .height(Size::fill())
        .background(color(fill))
}

fn pill(side: Side) -> Rect {
    let (activated, fixed, _) = window_state();
    let palette = Palette::current();
    at_side(
        side,
        rect()
            .padding(Gaps::new(
                TITLEBAR_RAIL.pill_padding_y,
                TITLEBAR_RAIL.pill_padding_x,
                TITLEBAR_RAIL.pill_padding_y,
                TITLEBAR_RAIL.pill_padding_x,
            ))
            .margin(Gaps::new_all(TITLEBAR_RAIL.pill_inset)),
    )
    .corner_radius(TITLEBAR_RAIL.pill_radius)
    .background(color(palette.raised_background))
    .border(
        Border::new()
            .width(1.0)
            .alignment(BorderAlignment::Inner)
            .fill(color(palette.separator)),
    )
    .layer(Layer::Overlay)
    .child(window_traffic_lights(None, activated, fixed).vertical(true))
}

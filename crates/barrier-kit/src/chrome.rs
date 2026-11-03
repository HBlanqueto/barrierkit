//! Titlebar layouts for client-side windows (top, rail, unified, pill and rail
//! with actions, on the left or the right) and how each one hides and reveals.

use freya_animation::prelude::{AnimNum, Ease, Function, use_animation_transition};
use freya_core::prelude::*;
use serde::{Deserialize, Serialize};
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};
use vow_appearance::{SidebarStyle, palette};
use vow_design::{
    DURATION_BASE, FULLSCREEN_GLASS, Rgba, SETTINGS, SHADOW, SHADOW_DARK, TITLEBAR,
    TITLEBAR_CLIENT_HEIGHT, TITLEBAR_RAIL,
};

use crate::glass::{GlassEdge, fullscreen_backdrop};
use crate::platform::FullscreenBar;
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
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    #[default]
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reveal {
    #[default]
    Always,
    Drawer,
    Push,
    Corner,
    Popup,
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
    pub pill_vertical: bool,
    pub reveal: Reveal,
    pub resting_edge: RestingEdge,
}

impl Default for ChromeStyle {
    fn default() -> Self {
        Self {
            layout: TitlebarLayout::Top,
            side: Side::Left,
            rail_width: TITLEBAR_RAIL.width,
            round_lights: false,
            pill_vertical: true,
            reveal: Reveal::Always,
            resting_edge: RestingEdge::None,
        }
    }
}

impl ChromeStyle {
    fn effective(self, sidebar_shown: bool) -> (TitlebarLayout, Reveal) {
        let layout = match self.layout {
            TitlebarLayout::Unified if !sidebar_shown => TitlebarLayout::Rail,
            layout => layout,
        };
        let reveal = match (layout, self.reveal) {
            (TitlebarLayout::Unified, Reveal::Drawer | Reveal::Push | Reveal::Popup) => {
                Reveal::Corner
            }
            (TitlebarLayout::Pill, Reveal::Drawer | Reveal::Push) => Reveal::Popup,
            (_, reveal) => reveal,
        };
        (layout, reveal)
    }

    fn rail_extent(self) -> f32 {
        if self.layout == TitlebarLayout::Actions {
            TITLEBAR_RAIL.actions_width
        } else {
            self.rail_width
        }
    }
}

// ADR 0054: the overlaid titlebar slides and fades in with the compositor bar instead of popping in.
pub(crate) fn use_focus_bar_reveal() {
    let controls = try_consume_root_context::<WindowControls>();
    let fallback = use_state(|| false);
    let shown = controls
        .as_ref()
        .map_or(fallback, |controls| controls.focus_bar);
    let reduced_motion = crate::theme::resolved().reduced_motion;
    let animation = use_animation_transition(shown, move |from: bool, to: bool| {
        AnimNum::new(f32::from(u8::from(from)), f32::from(u8::from(to)))
            .time(if reduced_motion { 0 } else { DURATION_BASE })
            .ease(Ease::Out)
            .function(Function::Cubic)
    });
    let progress = animation.get().value();
    if let Some(controls) = controls {
        let mut state = controls.focus_bar_progress;
        state.set_if_modified(progress);
    }
}

fn bar_height() -> f32 {
    TITLEBAR_CLIENT_HEIGHT.max(SETTINGS.toolbar_height) as f32
}

fn wants_reveal(
    layout: TitlebarLayout,
    reveal: Reveal,
    style: ChromeStyle,
    revealed: bool,
    point: (f32, f32),
    size: (f32, f32),
) -> bool {
    let (x, y) = point;
    let vertical = layout != TitlebarLayout::Top;
    let from_side = match (vertical, style.side) {
        (true, Side::Right) => size.0 - x,
        _ => x,
    };
    let corner = TITLEBAR_RAIL.reveal_corner;
    let in_corner = from_side < corner && y < corner;
    match reveal {
        Reveal::Always => false,
        Reveal::Corner => in_corner,
        Reveal::Popup if revealed => in_corner,
        Reveal::Drawer | Reveal::Push if revealed && vertical => {
            from_side < style.rail_extent() + TITLEBAR_RAIL.reveal_zone
        }
        Reveal::Drawer | Reveal::Push if revealed => y < bar_height() + TITLEBAR_RAIL.reveal_zone,
        _ if vertical => from_side < TITLEBAR_RAIL.reveal_zone,
        _ => y < TITLEBAR_RAIL.reveal_zone,
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Chrome {
    style: State<ChromeStyle>,
    sidebar_shown: State<bool>,
    progress: State<f32>,
    size: State<(f32, f32)>,
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

pub(crate) fn fullscreen_bar() -> FullscreenBar {
    try_consume_root_context::<WindowControls>()
        .map_or(FullscreenBar::Solid, |controls| controls.fullscreen_bar)
}

fn fullscreen_overlay() -> (bool, Option<f32>) {
    try_consume_root_context::<WindowControls>().map_or((false, None), |controls| {
        (
            controls.titlebar_overlaid(),
            controls.titlebar_overlay_top(),
        )
    })
}

// ADR 0054: in full screen the chrome shows exactly while the compositor bar is down.
fn shown_as(style: ChromeStyle, sidebar_shown: bool) -> (TitlebarLayout, Reveal) {
    let (layout, reveal) = style.effective(sidebar_shown);
    if fullscreen_overlay().0 {
        (layout, Reveal::Always)
    } else {
        (layout, reveal)
    }
}

fn reveal_state() -> (TitlebarLayout, Reveal, f32) {
    chrome().map_or((TitlebarLayout::Top, Reveal::Always, 1.0), |chrome| {
        let (layout, reveal) = shown_as(*chrome.style.read(), *chrome.sidebar_shown.read());
        (layout, reveal, *chrome.progress.read())
    })
}

pub(crate) fn rail_overlap((start, end): (f32, f32)) -> (f32, f32) {
    let Some(chrome) = chrome() else {
        return (0.0, 0.0);
    };
    let style = *chrome.style.read();
    let (layout, _) = style.effective(*chrome.sidebar_shown.read());
    if !matches!(layout, TitlebarLayout::Rail | TitlebarLayout::Actions) {
        return (0.0, 0.0);
    }
    let extent = style.rail_extent();
    match style.side {
        Side::Left => ((extent - start).max(0.0), 0.0),
        Side::Right => (0.0, (end - (chrome.size.read().0 - extent)).max(0.0)),
    }
}

pub(crate) fn lights_opacity() -> f32 {
    match reveal_state() {
        (_, Reveal::Always, _) => 1.0,
        (_, Reveal::Popup, _) => 0.0,
        (_, _, progress) => progress,
    }
}

pub(crate) fn revealable_bar(bar: Element, height: f32) -> Element {
    match reveal_state() {
        (TitlebarLayout::Top, Reveal::Push, progress) => rect()
            .width(Size::fill())
            .height(Size::px(height * progress))
            .overflow(Overflow::Clip)
            .child(bar)
            .into_element(),
        (TitlebarLayout::Top, Reveal::Drawer, progress) => rect()
            .width(Size::fill())
            .height(Size::px(0.0))
            .maybe(progress > 0.0, |el| {
                el.child(
                    rect()
                        .position(
                            Position::new_absolute()
                                .top(-height * (1.0 - progress))
                                .left(0.0)
                                .right(0.0),
                        )
                        .width(Size::fill())
                        .layer(Layer::Overlay)
                        .shadow(drop_shadow())
                        .child(bar),
                )
            })
            .into_element(),
        _ => bar,
    }
}

fn drop_shadow() -> Shadow {
    let resolved = crate::theme::resolved();
    let layer = if resolved.scheme == vow_appearance::ColorScheme::Light {
        SHADOW.contact
    } else {
        SHADOW_DARK.contact
    };
    let [r, g, b, _] = layer.color.to_rgba8();
    Shadow::new()
        .y(layer.y_offset as f32)
        .blur(layer.blur as f32)
        .color(Color::from_argb((layer.opacity * 255.0) as u8, r, g, b))
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
                progress: State::create(0.0),
                size: State::create((0.0, 0.0)),
            };
            provide_context(chrome);
            chrome
        });
        let mut style_state = chrome.style;
        style_state.set_if_modified(self.style);
        let style = self.style;
        let sidebar_shown = *chrome.sidebar_shown.read();
        let mut revealed = use_state(|| false);
        let mut size = chrome.size;
        let reduced_motion = crate::theme::resolved().reduced_motion;
        let animation = use_animation_transition(revealed, move |from: bool, to: bool| {
            AnimNum::new(f32::from(u8::from(from)), f32::from(u8::from(to)))
                .time(if reduced_motion { 0 } else { DURATION_BASE })
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        let progress = animation.get().value();
        let mut progress_state = chrome.progress;
        progress_state.set_if_modified(progress);

        let (layout, reveal) = shown_as(style, sidebar_shown);
        let (overlaid, overlay_top) = fullscreen_overlay();
        let hidden = reveal != Reveal::Always && reveal != Reveal::Corner;
        let gutter =
            if layout == TitlebarLayout::Pill && style.pill_vertical && !sidebar_shown && !overlaid
            {
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

        let footer = (layout == TitlebarLayout::Actions)
            .then(|| self.footer.clone())
            .flatten();
        let vertical_rail = matches!(layout, TitlebarLayout::Rail | TitlebarLayout::Actions);
        let lights_opacity = if reveal == Reveal::Corner {
            progress
        } else {
            1.0
        };
        let edge = match (vertical_rail && !overlaid, reveal) {
            (false, _) => None,
            (true, Reveal::Always | Reveal::Corner) => {
                Some(rail(style, footer.clone(), lights_opacity).into_element())
            }
            (true, Reveal::Push) => Some(
                rect()
                    .direction(Direction::Horizontal)
                    .height(Size::fill())
                    .child(resting_edge(style.resting_edge, true))
                    .child(
                        rect()
                            .width(Size::px(style.rail_extent() * progress))
                            .height(Size::fill())
                            .overflow(Overflow::Clip)
                            .child(rail(style, footer.clone(), 1.0)),
                    )
                    .into_element(),
            ),
            (true, _) => Some(resting_edge(style.resting_edge, true).into_element()),
        };
        let mut root = rect()
            .width(Size::fill())
            .height(Size::fill())
            .direction(Direction::Horizontal)
            .content(Content::flex())
            .on_sized(move |event: Event<SizedEventData>| {
                let area = event.area.size;
                size.set_if_modified((area.width, area.height));
            })
            .on_global_pointer_move(move |event: Event<PointerEventData>| {
                let point = event.global_location();
                let open = *revealed.peek();
                let wanted = wants_reveal(
                    layout,
                    reveal,
                    style,
                    open,
                    (point.x as f32, point.y as f32),
                    *size.peek(),
                );
                revealed.set_if_modified(wanted);
            })
            .on_pointer_leave(move |_| revealed.set_if_modified(false));
        root = match style.side {
            Side::Left => root.maybe_child(edge).child(content),
            Side::Right => root.child(content).maybe_child(edge),
        };
        if layout == TitlebarLayout::Top && hidden && reveal != Reveal::Popup {
            root = root.child(
                rect()
                    .position(Position::new_absolute().top(0.0).left(0.0).right(0.0))
                    .width(Size::fill())
                    .layer(Layer::Overlay)
                    .child(resting_edge(style.resting_edge, false)),
            );
        }
        if vertical_rail && reveal == Reveal::Drawer && progress > 0.0 {
            let extent = style.rail_extent();
            let position = Position::new_absolute().top(0.0);
            root =
                root.child(
                    rect()
                        .position(match style.side {
                            Side::Left => position.left(-extent * (1.0 - progress)),
                            Side::Right => position.right(-extent * (1.0 - progress)),
                        })
                        .height(Size::fill())
                        .layer(Layer::Overlay)
                        .shadow(drop_shadow())
                        .child(rail(style, footer.clone(), 1.0).background(color(
                            surface_background(&crate::theme::resolved(), SurfaceKind::Toplevel),
                        ))),
                );
        }
        if let Some(top) = overlay_top.filter(|_| vertical_rail) {
            let position = Position::new_absolute().top(top);
            let (position, edge) = match style.side {
                Side::Left => (position.left(0.0), GlassEdge::Left),
                Side::Right => (position.right(0.0), GlassEdge::Right),
            };
            root = root.child(
                rect()
                    .position(position)
                    .height(Size::func(move |context| Some(context.parent - top)))
                    .layer(Layer::Overlay)
                    .child(fullscreen_backdrop(
                        fullscreen_bar(),
                        edge,
                        style.rail_extent(),
                    ))
                    .child(rail(style, footer.clone(), 1.0)),
            );
        }
        let pill_progress = match (layout, reveal) {
            (TitlebarLayout::Pill, _) if overlaid && overlay_top.is_none() => None,
            (TitlebarLayout::Pill, Reveal::Always) => Some(1.0),
            (TitlebarLayout::Pill, Reveal::Corner) => Some(progress),
            (_, Reveal::Popup) => Some(progress),
            _ => None,
        };
        if let Some(shown) = pill_progress.filter(|shown| *shown > 0.0) {
            let slides = reveal == Reveal::Popup;
            root = root.child(pill(
                if layout == TitlebarLayout::Top {
                    Side::Left
                } else {
                    style.side
                },
                layout != TitlebarLayout::Top && style.pill_vertical,
                if slides { shown } else { 1.0 },
                if slides { 1.0 } else { shown },
            ));
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

fn rail(style: ChromeStyle, footer: Option<Element>, lights_opacity: f32) -> Rect {
    let resolved = crate::theme::resolved();
    let (activated, fixed, title) = window_state();
    let width = style.rail_extent();
    let island = resolved.sidebar.style == SidebarStyle::Island;
    let overlaid = fullscreen_overlay().0;
    let background = if island || overlaid {
        Rgba::TRANSPARENT
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
    let line = (!island && !overlaid).then(|| {
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
        .child(
            rect()
                .opacity(lights_opacity)
                .child(window_traffic_lights(activated, fixed).vertical(true)),
        )
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

fn resting_edge(edge: RestingEdge, vertical: bool) -> Rect {
    let resolved = crate::theme::resolved();
    let (thickness, fill) = match edge {
        RestingEdge::None => (0.0, Rgba::TRANSPARENT),
        RestingEdge::Line => (TITLEBAR_RAIL.edge_line, resolved.sidebar.palette.line),
        RestingEdge::Accent => (TITLEBAR_RAIL.edge_accent, Palette::current().accent),
    };
    let (width, height) = if vertical {
        (Size::px(thickness), Size::fill())
    } else {
        (Size::fill(), Size::px(thickness))
    };
    rect().width(width).height(height).background(color(fill))
}

fn pill(side: Side, vertical: bool, slide: f32, opacity: f32) -> Rect {
    let (activated, fixed, _) = window_state();
    let palette = Palette::current();
    let (overlaid, overlay_top) = fullscreen_overlay();
    let top = overlay_top.unwrap_or_default();
    let hidden_by = TITLEBAR_RAIL.reveal_corner * (1.0 - slide);
    let position = Position::new_absolute();
    let position = match (vertical, side) {
        (false, Side::Left) => position.top(top - hidden_by).left(0.0),
        (false, Side::Right) => position.top(top - hidden_by).right(0.0),
        (true, Side::Left) => position.top(top).left(-hidden_by),
        (true, Side::Right) => position.top(top).right(-hidden_by),
    };
    let glassy = matches!(
        fullscreen_bar(),
        FullscreenBar::Glass | FullscreenBar::ProgressiveGlass
    );
    let background = match (overlaid, glassy) {
        (true, true) => Rgba {
            a: FULLSCREEN_GLASS.tint[0].1,
            ..crate::glass::glass_tint()
        },
        (true, false) => Rgba {
            a: 1.0,
            ..crate::glass::glass_tint()
        },
        (false, _) => palette.raised_background,
    };
    rect()
        .position(position)
        .margin(Gaps::new_all(TITLEBAR_RAIL.pill_inset))
        .padding(Gaps::new(
            TITLEBAR_RAIL.pill_padding_y,
            TITLEBAR_RAIL.pill_padding_x,
            TITLEBAR_RAIL.pill_padding_y,
            TITLEBAR_RAIL.pill_padding_x,
        ))
        .corner_radius(TITLEBAR_RAIL.pill_radius)
        .background(color(background))
        .maybe(overlaid && glassy, |el| el.blur(FULLSCREEN_GLASS.blur))
        .border(
            Border::new()
                .width(1.0)
                .alignment(BorderAlignment::Inner)
                .fill(color(palette.separator)),
        )
        .maybe(slide < 1.0 || vertical, |el| el.shadow(drop_shadow()))
        .opacity(opacity)
        .layer(Layer::Overlay)
        .child(window_traffic_lights(activated, fixed).vertical(vertical))
}

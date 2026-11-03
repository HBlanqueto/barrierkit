//! Titlebar layouts for client-side windows (top, or a rail on the left, as a
//! strip or unified with the sidebar) and how each one hides and reveals.

use freya_animation::prelude::{AnimNum, Ease, Function, use_animation_transition};
use freya_core::prelude::*;
use serde::{Deserialize, Serialize};
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};
use vow_appearance::{SidebarStyle, palette};
use vow_design::{
    DURATION_BASE, Rgba, SETTINGS, SHADOW, SHADOW_DARK, TITLEBAR, TITLEBAR_CLIENT_HEIGHT,
    TITLEBAR_RAIL,
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
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RailStyle {
    #[default]
    Normal,
    Unified,
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
    pub rail_width: f32,
    pub round_lights: bool,
    pub rail_style: RailStyle,
    pub reveal: Reveal,
    pub resting_edge: RestingEdge,
}

impl Default for ChromeStyle {
    fn default() -> Self {
        Self {
            layout: TitlebarLayout::Top,
            rail_width: TITLEBAR_RAIL.width,
            round_lights: false,
            rail_style: RailStyle::Normal,
            reveal: Reveal::Always,
            resting_edge: RestingEdge::None,
        }
    }
}

impl ChromeStyle {
    pub(crate) fn unified(self, sidebar_shown: bool) -> bool {
        self.layout == TitlebarLayout::Rail
            && self.rail_style == RailStyle::Unified
            && sidebar_shown
    }

    fn effective(self, sidebar_shown: bool) -> (TitlebarLayout, Reveal) {
        let reveal = match self.reveal {
            Reveal::Drawer | Reveal::Push | Reveal::Popup if self.unified(sidebar_shown) => {
                Reveal::Corner
            }
            reveal => reveal,
        };
        (self.layout, reveal)
    }
}

pub(crate) fn use_switch_progress(on: State<bool>) -> f32 {
    let reduced_motion = crate::theme::resolved().reduced_motion;
    let animation = use_animation_transition(on, move |from: bool, to: bool| {
        AnimNum::new(f32::from(u8::from(from)), f32::from(u8::from(to)))
            .time(if reduced_motion { 0 } else { DURATION_BASE })
            .ease(Ease::Out)
            .function(Function::Cubic)
    });
    animation.get().value()
}

// ADR 0054: the overlaid titlebar slides and fades in with the compositor bar instead of popping in.
pub(crate) fn use_focus_bar_reveal() {
    let controls = try_consume_root_context::<WindowControls>();
    let fallback = use_state(|| false);
    let shown = controls
        .as_ref()
        .map_or(fallback, |controls| controls.focus_bar);
    let progress = use_switch_progress(shown);
    if let Some(controls) = controls {
        let mut state = controls.focus_bar_progress;
        state.set_if_modified(progress);
    }
}

pub(crate) fn window_root(app: impl IntoElement) -> Rect {
    use_focus_bar_reveal();
    let pushed =
        try_consume_root_context::<WindowControls>().map_or(0.0, |controls| controls.push_offset());
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .padding(Gaps::new(pushed, 0.0, 0.0, 0.0))
        .child(app)
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
) -> bool {
    let (x, y) = point;
    let vertical = layout != TitlebarLayout::Top;
    let corner = TITLEBAR_RAIL.reveal_corner;
    let in_corner = x < corner && y < corner;
    match reveal {
        Reveal::Always => false,
        Reveal::Corner => in_corner,
        Reveal::Popup if revealed => in_corner,
        Reveal::Drawer | Reveal::Push if revealed && vertical => {
            x < style.rail_width + TITLEBAR_RAIL.reveal_zone
        }
        Reveal::Drawer | Reveal::Push if revealed => y < bar_height() + TITLEBAR_RAIL.reveal_zone,
        _ if vertical => x < TITLEBAR_RAIL.reveal_zone,
        _ => y < TITLEBAR_RAIL.reveal_zone,
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Chrome {
    style: State<ChromeStyle>,
    rail_items: State<RailItems>,
    sidebar_shown: State<bool>,
    progress: State<f32>,
}

fn chrome() -> Option<Chrome> {
    try_consume_context()
}

pub(crate) fn chrome_style() -> ChromeStyle {
    chrome()
        .map(|chrome| *chrome.style.read())
        .unwrap_or_default()
}

pub(crate) fn rail_items() -> RailItems {
    chrome()
        .map(|chrome| chrome.rail_items.read().clone())
        .unwrap_or_default()
}

pub(crate) fn report_sidebar_shown(shown: bool) {
    if let Some(mut chrome) = chrome() {
        chrome.sidebar_shown.set_if_modified(shown);
    }
}

pub(crate) fn fullscreen_push() -> bool {
    try_consume_root_context::<WindowControls>()
        .is_some_and(|controls| *controls.fullscreen_push.read())
}

pub(crate) fn fullscreen_bar() -> FullscreenBar {
    try_consume_root_context::<WindowControls>().map_or(FullscreenBar::Solid, |controls| {
        *controls.fullscreen_bar.read()
    })
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

#[derive(Clone, Default, PartialEq)]
pub(crate) struct RailItems {
    pub(crate) top: Option<Element>,
    pub(crate) bottom: Option<Element>,
}

impl RailItems {
    fn any(&self) -> bool {
        self.top.is_some() || self.bottom.is_some()
    }
}

#[derive(PartialEq)]
pub struct WindowChrome {
    style: ChromeStyle,
    rail_items: RailItems,
    child: Element,
}

impl WindowChrome {
    pub fn new(style: ChromeStyle, child: impl IntoElement) -> Self {
        Self {
            style,
            rail_items: RailItems::default(),
            child: child.into_element(),
        }
    }

    pub fn rail_top(mut self, items: impl IntoElement) -> Self {
        self.rail_items.top = Some(items.into_element());
        self
    }

    pub fn rail_bottom(mut self, items: impl IntoElement) -> Self {
        self.rail_items.bottom = Some(items.into_element());
        self
    }
}

impl Component for WindowChrome {
    fn render(&self) -> impl IntoElement {
        let chrome = use_hook(|| {
            let chrome = Chrome {
                style: State::create(self.style),
                rail_items: State::create(RailItems::default()),
                sidebar_shown: State::create(false),
                progress: State::create(0.0),
            };
            provide_context(chrome);
            chrome
        });
        let sidebar_shown = *chrome.sidebar_shown.read();
        let style = ChromeStyle {
            rail_width: if self.rail_items.any() {
                self.style.rail_width.max(TITLEBAR_RAIL.actions_width)
            } else {
                self.style.rail_width
            },
            ..self.style
        };
        let mut style_state = chrome.style;
        style_state.set_if_modified(style);
        let mut items_state = chrome.rail_items;
        items_state.set_if_modified(self.rail_items.clone());
        let mut revealed = use_state(|| false);
        let progress = use_switch_progress(revealed);
        let mut progress_state = chrome.progress;
        progress_state.set_if_modified(progress);

        let (layout, reveal) = shown_as(style, sidebar_shown);
        let overlaid = fullscreen_overlay().0;
        let hidden = reveal != Reveal::Always && reveal != Reveal::Corner;
        let content = rect()
            .width(Size::flex(1.0))
            .height(Size::fill())
            .child(self.child.clone());

        let items = &self.rail_items;
        let vertical_rail = layout == TitlebarLayout::Rail && !style.unified(sidebar_shown);
        let bar = fullscreen_bar();
        let focus_progress = try_consume_root_context::<WindowControls>()
            .map_or(0.0, |controls| *controls.focus_bar_progress.read());
        let push = fullscreen_push();
        let rail_over = overlaid && vertical_rail && !push;
        let rail_pushed = overlaid && vertical_rail && push;
        let lights_opacity = if reveal == Reveal::Corner {
            progress
        } else {
            1.0
        };
        let edge = match (vertical_rail && !rail_over, reveal) {
            (false, _) => None,
            (true, _) if rail_pushed => Some(
                rect()
                    .width(Size::px(style.rail_width * focus_progress))
                    .height(Size::fill())
                    .overflow(Overflow::Clip)
                    .child(rail(style, items, 1.0))
                    .into_element(),
            ),
            (true, Reveal::Always | Reveal::Corner) => {
                Some(rail(style, items, lights_opacity).into_element())
            }
            (true, Reveal::Push) => Some(
                rect()
                    .direction(Direction::Horizontal)
                    .height(Size::fill())
                    .child(resting_edge(style.resting_edge, true))
                    .child(
                        rect()
                            .width(Size::px(style.rail_width * progress))
                            .height(Size::fill())
                            .overflow(Overflow::Clip)
                            .child(rail(style, items, 1.0)),
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
            .on_global_pointer_move(move |event: Event<PointerEventData>| {
                let point = event.global_location();
                let open = *revealed.peek();
                let wanted = wants_reveal(
                    layout,
                    reveal,
                    style,
                    open,
                    (point.x as f32, point.y as f32),
                );
                revealed.set_if_modified(wanted);
            })
            .on_pointer_leave(move |_| revealed.set_if_modified(false));
        root = root.maybe_child(edge).child(content);
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
            let extent = style.rail_width;
            let position = Position::new_absolute().top(0.0);
            root = root.child(
                rect()
                    .position(position.left(-extent * (1.0 - progress)))
                    .height(Size::fill())
                    .layer(Layer::Overlay)
                    .shadow(drop_shadow())
                    .child(rail(style, items, 1.0).background(color(surface_background(
                        &crate::theme::resolved(),
                        SurfaceKind::Toplevel,
                    )))),
            );
        }
        if rail_over && focus_progress > 0.0 {
            let extent = style.rail_width;
            let position = Position::new_absolute().top(0.0);
            let position = position.left(-extent * (1.0 - focus_progress));
            root = root.child(
                rect()
                    .position(position)
                    .width(Size::px(extent))
                    .height(Size::percent(100.0))
                    .layer(Layer::Overlay)
                    .child(fullscreen_backdrop(bar, GlassEdge::Left, extent, 1.0))
                    .child(rail(style, items, 1.0)),
            );
        }
        if reveal == Reveal::Popup && progress > 0.0 && !overlaid {
            root = root.child(pill(layout == TitlebarLayout::Rail, progress));
        }
        root
    }
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

fn rail(style: ChromeStyle, items: &RailItems, lights_opacity: f32) -> Rect {
    let resolved = crate::theme::resolved();
    let (activated, fixed, title) = window_state();
    let width = style.rail_width;
    let island = resolved.sidebar.style == SidebarStyle::Island;
    let controls = try_consume_root_context::<WindowControls>();
    let overlaid = fullscreen_overlay().0;
    let lights_shown = controls
        .as_ref()
        .is_none_or(|controls| controls.traffic_lights_shown());
    let below_bar = controls
        .as_ref()
        .filter(|_| overlaid && !fullscreen_push())
        .map_or(0.0, |controls| {
            *controls.focus_bar_height.read() * *controls.focus_bar_progress.read()
        });
    let floating = overlaid && !fullscreen_push();
    let background = if island || floating {
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
    let line = (!island && !floating).then(|| {
        rect()
            .position(Position::new_absolute().top(0.0).right(0.0))
            .width(Size::px(1.0))
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
            TITLEBAR_RAIL.padding_top + below_bar,
            0.0,
            TITLEBAR_RAIL.padding_bottom,
            0.0,
        ))
        .maybe_child(lights_shown.then(|| {
            rect()
                .maybe(overlaid, |el| el.layer(Layer::Overlay))
                .opacity(lights_opacity)
                .child(window_traffic_lights(activated, fixed).vertical(true))
        }))
        .maybe_child(items.top.clone())
        .child(RotatedTitle {
            title,
            ink: title_ink,
            fixed,
        })
        .maybe_child(items.bottom.clone())
        .maybe_child(line)
}

#[derive(PartialEq)]
struct RotatedTitle {
    title: String,
    ink: Rgba,
    fixed: bool,
}

impl Component for RotatedTitle {
    fn render(&self) -> impl IntoElement {
        let mut area = use_state(|| (0.0f32, 0.0f32));
        let (width, height) = *area.read();
        let angle = -90.0;
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

fn pill(vertical: bool, slide: f32) -> Rect {
    let (activated, fixed, _) = window_state();
    let palette = Palette::current();
    let hidden_by = TITLEBAR_RAIL.reveal_corner * (1.0 - slide);
    let position = Position::new_absolute();
    let position = match vertical {
        false => position.top(-hidden_by).left(0.0),
        true => position.top(0.0).left(-hidden_by),
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
        .background(color(palette.raised_background))
        .border(
            Border::new()
                .width(1.0)
                .alignment(BorderAlignment::Inner)
                .fill(color(palette.separator)),
        )
        .maybe(slide < 1.0 || vertical, |el| el.shadow(drop_shadow()))
        .layer(Layer::Overlay)
        .child(window_traffic_lights(activated, fixed).vertical(vertical))
}

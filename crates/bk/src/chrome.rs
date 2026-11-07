use ctdesign::{SETTINGS, TITLEBAR_CLIENT_HEIGHT, TITLEBAR_RAIL};
use freya_core::prelude::*;
use serde::{Deserialize, Serialize};
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};

use crate::controls::shadowed;
use crate::glass::{GlassEdge, fullscreen_backdrop};
use crate::platform::FullscreenBar;
use crate::platform::WindowControls;
use crate::styles::Painted;
use crate::styles::{
    ControlState, ControlStyle, FullscreenBarStyle, LayoutDirection, Paints, ShadowStyle,
    ShapeStyle, StateValue, Theme, TrafficLightsStyle, WindowChromeStyle, resolve,
    use_provide_theme,
};
use crate::titlebar::window_drag_handler;
use crate::traffic::TrafficLights;

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ChromeLayout {
    pub(crate) layout: TitlebarLayout,
    pub(crate) rail_width: f32,
    pub(crate) round_lights: bool,
    pub(crate) rail_style: RailStyle,
    pub(crate) reveal: Reveal,
    pub(crate) rail_spacing: f32,
    pub(crate) rail_padding_bottom: f32,
}

impl From<&WindowChromeStyle> for ChromeLayout {
    fn from(style: &WindowChromeStyle) -> Self {
        Self {
            layout: style.layout,
            rail_width: style.rail_width,
            round_lights: style.round_lights,
            rail_style: style.rail_style,
            reveal: style.reveal,
            rail_spacing: style.rail_spacing,
            rail_padding_bottom: style.rail_padding_bottom,
        }
    }
}

impl ChromeLayout {
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

// ADR 0054: the overlaid titlebar slides and fades in with the compositor bar instead of popping in.
pub(crate) fn use_focus_bar_reveal() {
    let controls = try_consume_root_context::<WindowControls>();
    let fallback = use_state(|| false);
    let shown = controls
        .as_ref()
        .map_or(fallback, |controls| controls.focus_bar);
    let style: FullscreenBarStyle = resolve(&Theme::tengen());
    let progress = crate::animation::use_switch_progress(shown, style.animation);
    if let Some(controls) = controls {
        let mut state = controls.focus_bar_progress;
        state.set_if_modified(progress);
    }
}

pub(crate) fn window_root(theme: Theme, app: impl IntoElement) -> Rect {
    use_provide_theme(theme);
    use_focus_bar_reveal();
    if try_consume_root_context::<WindowControls>().is_some_and(|controls| controls.hud) {
        crate::theme::use_appearance(|config| {
            config.color_scheme = ctappearance::ColorScheme::Dark
        });
    }
    let pushed =
        try_consume_root_context::<WindowControls>().map_or(0.0, |controls| controls.push_offset());
    rect()
        .expanded()
        .padding(Gaps::new(pushed, 0.0, 0.0, 0.0))
        .child(app)
}

fn bar_height() -> f32 {
    TITLEBAR_CLIENT_HEIGHT.max(SETTINGS.toolbar_height) as f32
}

fn wants_reveal(
    layout: TitlebarLayout,
    reveal: Reveal,
    style: ChromeLayout,
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
    style: State<ChromeLayout>,
    shadows: State<Vec<ShadowStyle>>,
    rail_items: State<RailItems>,
    sidebar_shown: State<bool>,
    progress: State<f32>,
}

fn chrome() -> Option<Chrome> {
    try_consume_context()
}

pub(crate) fn chrome_style() -> ChromeLayout {
    chrome().map_or_else(
        || ChromeLayout::from(&WindowChromeStyle::tengen()),
        |chrome| *chrome.style.read(),
    )
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
fn shown_as(style: ChromeLayout, sidebar_shown: bool) -> (TitlebarLayout, Reveal) {
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
                        .map(chrome(), |el, chrome| shadowed(el, &chrome.shadows.read()))
                        .child(bar),
                )
            })
            .into_element(),
        _ => bar,
    }
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
    style: Theme,
    rail_items: RailItems,
    child: Element,
}

impl WindowChrome {
    pub fn new(child: impl IntoElement) -> Self {
        Self {
            style: Theme::tengen(),
            rail_items: RailItems::default(),
            child: child.into_element(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(WindowChromeStyle) -> WindowChromeStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
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
        let resolved: WindowChromeStyle = resolve(&self.style);
        let chrome = use_hook(|| {
            let chrome = Chrome {
                style: State::create(ChromeLayout::from(&resolved)),
                shadows: State::create(resolved.shadows.clone()),
                rail_items: State::create(RailItems::default()),
                sidebar_shown: State::create(false),
                progress: State::create(0.0),
            };
            provide_context(chrome);
            chrome
        });
        let sidebar_shown = *chrome.sidebar_shown.read();
        let style = WindowChromeStyle {
            rail_width: if self.rail_items.any() {
                resolved.rail_width.max(TITLEBAR_RAIL.actions_width)
            } else {
                resolved.rail_width
            },
            ..resolved
        };
        let layout_style = ChromeLayout::from(&style);
        let mut style_state = chrome.style;
        style_state.set_if_modified(layout_style);
        let mut shadows_state = chrome.shadows;
        shadows_state.set_if_modified(style.shadows.clone());
        let mut items_state = chrome.rail_items;
        items_state.set_if_modified(self.rail_items.clone());
        let mut revealed = use_state(|| false);
        let progress =
            crate::animation::use_animated(f32::from(u8::from(*revealed.read())), style.animation);
        let mut progress_state = chrome.progress;
        progress_state.set_if_modified(progress);

        let (layout, reveal) = shown_as(layout_style, sidebar_shown);
        let overlaid = fullscreen_overlay().0;
        let hidden = reveal != Reveal::Always && reveal != Reveal::Corner;
        let content = rect()
            .width(Size::flex(1.0))
            .height(Size::fill())
            .child(self.child.clone());

        let items = &self.rail_items;
        let vertical_rail = layout == TitlebarLayout::Rail && !layout_style.unified(sidebar_shown);
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
                    .child(rail(&style, items, 1.0))
                    .into_element(),
            ),
            (true, Reveal::Always | Reveal::Corner) => {
                Some(rail(&style, items, lights_opacity).into_element())
            }
            (true, Reveal::Push) => Some(
                rect()
                    .direction(Direction::Horizontal)
                    .height(Size::fill())
                    .child(resting_edge(&style, true))
                    .child(
                        rect()
                            .width(Size::px(style.rail_width * progress))
                            .height(Size::fill())
                            .overflow(Overflow::Clip)
                            .child(rail(&style, items, 1.0)),
                    )
                    .into_element(),
            ),
            (true, _) => Some(resting_edge(&style, true).into_element()),
        };
        let mut root = rect()
            .expanded()
            .direction(Direction::Horizontal)
            .content(Content::flex())
            .on_global_pointer_move(move |event: Event<PointerEventData>| {
                let point = event.global_location();
                let open = *revealed.peek();
                let wanted = wants_reveal(
                    layout,
                    reveal,
                    layout_style,
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
                    .child(resting_edge(&style, false)),
            );
        }
        if vertical_rail && reveal == Reveal::Drawer && progress > 0.0 {
            let extent = style.rail_width;
            let position = Position::new_absolute().top(0.0);
            let drawer = rect()
                .position(position.left(-extent * (1.0 - progress)))
                .height(Size::fill())
                .layer(Layer::Overlay);
            let background = window_paints(&style).fill(&style.drawer_background);
            root = root.child(
                shadowed(drawer, &style.shadows)
                    .child(rail(&style, items, 1.0).background(background)),
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
                    .child(rail(&style, items, 1.0)),
            );
        }
        if reveal == Reveal::Popup && progress > 0.0 && !overlaid {
            root = root.child(pill(&style, layout == TitlebarLayout::Rail, progress));
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

fn window_paints(style: &WindowChromeStyle) -> Paints {
    let state = ControlState {
        focused: window_state().0,
        ..ControlState::default()
    };
    Paints::current(state, style.effect)
}

fn line(paints: &Paints, (paint, thickness): &(ShapeStyle, f32), vertical: bool) -> Rect {
    let (width, height) = if vertical {
        (Size::px(*thickness), Size::fill())
    } else {
        (Size::fill(), Size::px(*thickness))
    };
    rect()
        .width(width)
        .height(height)
        .background(paints.fill(&StateValue::new(paint.clone())))
}

fn rail(style: &WindowChromeStyle, items: &RailItems, lights_opacity: f32) -> Rect {
    let (activated, fixed, title) = window_state();
    let paints = window_paints(style);
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
    let edge = style
        .rail_line
        .as_ref()
        .filter(|_| !floating)
        .map(|rail_line| {
            line(&paints, rail_line, true).position(Position::new_absolute().top(0.0).right(0.0))
        });
    rect()
        .width(Size::px(style.rail_width))
        .height(Size::fill())
        .maybe(!floating, |el| {
            el.background(paints.fill(&style.rail_background))
        })
        .direction(Direction::Vertical)
        .content(Content::flex())
        .cross_align(Alignment::Center)
        .spacing(style.rail_spacing)
        .padding(Gaps::new(
            style.rail_padding_top + below_bar,
            0.0,
            style.rail_padding_bottom,
            0.0,
        ))
        .maybe_child(lights_shown.then(|| {
            rect()
                .maybe(overlaid, |el| el.layer(Layer::Overlay))
                .opacity(lights_opacity)
                .child(TrafficLights::for_window(activated, fixed).vertical(true))
        }))
        .maybe_child(items.top.clone())
        .child(RotatedTitle {
            title,
            ink: paints.color(style.title.resolve(ControlState {
                focused: activated,
                ..ControlState::default()
            })),
            font: style.title_font,
            angle: style.title_angle,
            fixed,
        })
        .maybe_child(items.bottom.clone())
        .maybe_child(edge)
}

#[derive(PartialEq)]
struct RotatedTitle {
    title: String,
    ink: ctdesign::Rgba,
    font: ctdesign::Font,
    angle: f32,
    fixed: bool,
}

impl Component for RotatedTitle {
    fn render(&self) -> impl IntoElement {
        let mut area = use_state(|| (0.0f32, 0.0f32));
        let (width, height) = *area.read();
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
                    .rotate(self.angle)
                    .center()
                    .child(
                        label()
                            .text(self.title.clone())
                            .font_family(self.font.family)
                            .font_size(self.font.size as f32)
                            .font_weight(FontWeight::from(self.font.weight as i32))
                            .color(crate::theme::color(self.ink))
                            .max_lines(1)
                            .text_overflow(TextOverflow::Ellipsis),
                    ),
            )
    }
}

fn resting_edge(style: &WindowChromeStyle, vertical: bool) -> Rect {
    let paints = window_paints(style);
    match style.resting_edge {
        RestingEdge::None => line(&paints, &(ShapeStyle::Clear, 0.0), vertical),
        RestingEdge::Line => line(&paints, &style.line_edge, vertical),
        RestingEdge::Accent => line(&paints, &style.accent_edge, vertical),
    }
}

fn pill(style: &WindowChromeStyle, vertical: bool, slide: f32) -> Rect {
    let (activated, fixed, _) = window_state();
    let paints = window_paints(style);
    let hidden_by = TITLEBAR_RAIL.reveal_corner * (1.0 - slide);
    let position = Position::new_absolute();
    let position = match vertical {
        false => position.top(-hidden_by).left(0.0),
        true => position.top(0.0).left(-hidden_by),
    };
    let (padding_x, padding_y) = style.pill_padding;
    let lights: TrafficLightsStyle = resolve(&Theme::tengen());
    let (lights_width, lights_height) = if vertical {
        (lights.diameter, lights.length())
    } else {
        (lights.length(), lights.diameter)
    };
    let (width, height) = (
        lights_width + 2.0 * padding_x,
        lights_height + 2.0 * padding_y,
    );
    rect()
        .position(position)
        .margin(Gaps::new_all(style.pill_inset))
        .padding(Gaps::new(padding_y, padding_x, padding_y, padding_x))
        .corner_radius(
            style
                .pill_shape
                .corner_radius(width, height, LayoutDirection::LeftToRight),
        )
        .background(paints.fill(&style.pill_background))
        .bordered(&paints, style.pill_border.as_ref())
        .maybe(slide < 1.0 || vertical, |el| shadowed(el, &style.shadows))
        .layer(Layer::Overlay)
        .child(TrafficLights::for_window(activated, fixed).vertical(vertical))
}

use ctdesign::{SETTINGS, TITLEBAR_CLIENT_HEIGHT, TITLEBAR_REVEAL};
use freya_core::prelude::*;
use serde::{Deserialize, Serialize};
use torin::prelude::{Content, Direction, Gaps, Position, Size};

use crate::controls::shadowed;
use crate::elements::{Overrides, forward_overrides};
use crate::platform::WindowControls;
use crate::styles::{
    ControlState, FullscreenBarStyle, LayoutDirection, Painted, Paints, ShadowStyle, ShapeStyle,
    StateValue, Theme, TrafficLightsStyle, WindowChromeStyle, resolve, use_provide_theme,
};
use crate::traffic::TrafficLights;

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
    rect().expanded().child(app)
}

fn bar_height() -> f32 {
    TITLEBAR_CLIENT_HEIGHT.max(SETTINGS.toolbar_height) as f32
}

fn wants_reveal(reveal: Reveal, revealed: bool, (x, y): (f32, f32)) -> bool {
    let corner = TITLEBAR_REVEAL.reveal_corner;
    let in_corner = x < corner && y < corner;
    match reveal {
        Reveal::Always => false,
        Reveal::Corner => in_corner,
        Reveal::Popup if revealed => in_corner,
        Reveal::Drawer | Reveal::Push if revealed => y < bar_height() + TITLEBAR_REVEAL.reveal_zone,
        _ => y < TITLEBAR_REVEAL.reveal_zone,
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Chrome {
    reveal: State<Reveal>,
    shadows: State<Vec<ShadowStyle>>,
    progress: State<f32>,
}

fn chrome() -> Option<Chrome> {
    try_consume_context()
}

fn fullscreen_overlay() -> bool {
    try_consume_root_context::<WindowControls>()
        .is_some_and(|controls| controls.titlebar_overlaid())
}

// ADR 0054: in full screen the chrome shows exactly while the compositor bar is down.
fn shown_as(reveal: Reveal) -> Reveal {
    if fullscreen_overlay() {
        Reveal::Always
    } else {
        reveal
    }
}

fn reveal_state() -> (Reveal, f32) {
    chrome().map_or((Reveal::Always, 1.0), |chrome| {
        (shown_as(*chrome.reveal.read()), *chrome.progress.read())
    })
}

pub(crate) fn lights_opacity() -> f32 {
    match reveal_state() {
        (Reveal::Always, _) => 1.0,
        (Reveal::Popup, _) => 0.0,
        (_, progress) => progress,
    }
}

pub(crate) fn revealable_bar(bar: Element, height: f32) -> Element {
    match reveal_state() {
        (Reveal::Push, progress) => rect()
            .width(Size::fill())
            .height(Size::px(height * progress))
            .overflow(Overflow::Clip)
            .child(bar)
            .into_element(),
        (Reveal::Drawer, progress) => rect()
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

#[derive(PartialEq)]
pub struct WindowChrome {
    style: Theme,
    child: Element,
    overrides: Overrides,
}

impl WindowChrome {
    pub fn new(child: impl IntoElement) -> Self {
        Self {
            style: Theme::tengen(),
            child: child.into_element(),
            overrides: Overrides::default(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(WindowChromeStyle) -> WindowChromeStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }
}

forward_overrides!(WindowChrome);

impl Component for WindowChrome {
    fn render(&self) -> impl IntoElement {
        let style: WindowChromeStyle = resolve(&self.style);
        let chrome = use_hook(|| {
            let chrome = Chrome {
                reveal: State::create(style.reveal),
                shadows: State::create(style.shadows.clone()),
                progress: State::create(0.0),
            };
            provide_context(chrome);
            chrome
        });
        let mut reveal_state = chrome.reveal;
        reveal_state.set_if_modified(style.reveal);
        let mut shadows_state = chrome.shadows;
        shadows_state.set_if_modified(style.shadows.clone());
        let mut revealed = use_state(|| false);
        let progress =
            crate::animation::use_animated(f32::from(u8::from(*revealed.read())), style.animation);
        let mut progress_state = chrome.progress;
        progress_state.set_if_modified(progress);

        let reveal = shown_as(style.reveal);
        let overlaid = fullscreen_overlay();
        let hidden = reveal != Reveal::Always && reveal != Reveal::Corner;
        let root = rect()
            .expanded()
            .direction(Direction::Horizontal)
            .content(Content::flex())
            .on_global_pointer_move(move |event: Event<PointerEventData>| {
                let point = event.global_location();
                let open = *revealed.peek();
                revealed.set_if_modified(wants_reveal(
                    reveal,
                    open,
                    (point.x as f32, point.y as f32),
                ));
            })
            .on_pointer_leave(move |_| revealed.set_if_modified(false))
            .child(
                rect()
                    .width(Size::flex(1.0))
                    .height(Size::fill())
                    .child(self.child.clone()),
            )
            .maybe_child((hidden && reveal != Reveal::Popup).then(|| {
                rect()
                    .position(Position::new_absolute().top(0.0).left(0.0).right(0.0))
                    .width(Size::fill())
                    .layer(Layer::Overlay)
                    .child(resting_edge(&style))
            }))
            .maybe_child(
                (reveal == Reveal::Popup && progress > 0.0 && !overlaid)
                    .then(|| pill(&style, progress)),
            );
        self.overrides.apply(root)
    }
}

fn window_state() -> (bool, bool) {
    try_consume_root_context::<WindowControls>()
        .map(|controls| (controls.status.read().activated, controls.fixed))
        .unwrap_or((true, false))
}

fn window_paints() -> Paints {
    let state = ControlState {
        focused: window_state().0,
        ..ControlState::default()
    };
    Paints::current(state)
}

fn line(paints: &Paints, (paint, thickness): &(ShapeStyle, f32)) -> Rect {
    rect()
        .width(Size::fill())
        .height(Size::px(*thickness))
        .background(paints.fill(&StateValue::new(paint.clone())))
}

fn resting_edge(style: &WindowChromeStyle) -> Rect {
    let paints = window_paints();
    match style.resting_edge {
        RestingEdge::None => line(&paints, &(ShapeStyle::Clear, 0.0)),
        RestingEdge::Line => line(&paints, &style.line_edge),
        RestingEdge::Accent => line(&paints, &style.accent_edge),
    }
}

fn pill(style: &WindowChromeStyle, slide: f32) -> Rect {
    let (activated, fixed) = window_state();
    let paints = window_paints();
    let hidden_by = TITLEBAR_REVEAL.reveal_corner * (1.0 - slide);
    let position = Position::new_absolute().top(-hidden_by).left(0.0);
    let (padding_x, padding_y) = style.pill_padding;
    let lights: TrafficLightsStyle = resolve(&Theme::tengen());
    let (lights_width, lights_height) = (lights.length(), lights.diameter);
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
        .maybe(slide < 1.0, |el| shadowed(el, &style.shadows))
        .layer(Layer::Overlay)
        .child(TrafficLights::for_window(activated, fixed))
}

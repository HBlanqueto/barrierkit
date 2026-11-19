use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use ctappearance::{ScrollBarClick, ScrollBars};
use ctdesign::Rgba;
use freya_components::scrollviews::{
    ScrollConfig, ScrollController as Inner, ScrollView as Scroller,
};
use freya_core::prelude::*;
use torin::prelude::{Area, Direction, Position, Size};

use crate::adornments::Divider;
use crate::platform::Timers;
use crate::styles::{
    ControlState, LayoutDirection, Paints, ScrollViewStyle, ShapeStyle, StateValue, Theme, resolve,
};
use crate::theme::color;

// Freya paints by tree depth; the thumb must sit above the scrolled content, like Freya's own bar.
const THUMB_LAYER: i16 = 999;

pub(crate) static SCROLLED_WITH_A_MOUSE: AtomicBool = AtomicBool::new(false);

#[derive(Clone, PartialEq)]
pub struct ScrollView {
    children: Vec<Element>,
    width: Size,
    height: Size,
    direction: Direction,
    scroll_bar: bool,
    top_line: bool,
    edge_effect: bool,
    controller: Option<ScrollController>,
    style: Theme,
    key: DiffKey,
}

#[derive(Clone, Copy, PartialEq)]
pub struct ScrollController {
    inner: Inner,
    viewport: State<(f32, f32)>,
    content: State<(f32, f32)>,
}

pub fn use_scroll_controller() -> ScrollController {
    let inner = freya_components::scrollviews::use_scroll_controller(ScrollConfig::default);
    let viewport = use_state(|| (0.0, 0.0));
    let content = use_state(|| (0.0, 0.0));
    ScrollController {
        inner,
        viewport,
        content,
    }
}

impl From<ScrollController> for (i32, i32) {
    fn from(controller: ScrollController) -> Self {
        controller.inner.into()
    }
}

fn clamped(offset: i32, delta: f64, viewport: f32, content: f32) -> i32 {
    let limit = (content - viewport).max(0.0);
    (offset as f32 + delta as f32).clamp(-limit, 0.0) as i32
}

pub fn revealed_offset(offset: i32, top: f32, bottom: f32, viewport: f32) -> i32 {
    let shown_top = -offset as f32;
    if top < shown_top {
        -(top.floor() as i32)
    } else if bottom > shown_top + viewport {
        -((bottom - viewport).ceil() as i32)
    } else {
        offset
    }
}

impl ScrollController {
    pub fn scroll_to_x(&mut self, to: i32) -> bool {
        self.inner.scroll_to_x(to)
    }

    pub fn reveal_y(mut self, top: f32, bottom: f32) {
        let (_, offset): (i32, i32) = self.into();
        let next = revealed_offset(offset, top, bottom, self.viewport.peek().1);
        if next != offset {
            self.inner.scroll_to_y(next);
        }
    }

    // Freya sends the wheel only to the hovered node and its ancestors; a layer over the view uses this.
    pub fn scroll_by_wheel(mut self, event: &WheelEventData) {
        let (x, y): (i32, i32) = self.into();
        let (viewport, content) = (*self.viewport.peek(), *self.content.peek());
        self.inner
            .scroll_to_x(clamped(x, event.delta_x, viewport.0, content.0));
        self.inner
            .scroll_to_y(clamped(y, event.delta_y, viewport.1, content.1));
    }
}

impl Default for ScrollView {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollView {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            width: Size::fill(),
            height: Size::fill(),
            direction: Direction::Vertical,
            scroll_bar: true,
            top_line: false,
            edge_effect: false,
            controller: None,
            style: Theme::tengen(),
            key: DiffKey::None,
        }
    }

    pub fn style(mut self, restyle: impl Fn(ScrollViewStyle) -> ScrollViewStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn edge_effect(mut self, edge_effect: bool) -> Self {
        self.edge_effect = edge_effect;
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn scroll_bar(mut self, scroll_bar: bool) -> Self {
        self.scroll_bar = scroll_bar;
        self
    }

    pub fn top_line(mut self, top_line: bool) -> Self {
        self.top_line = top_line;
        self
    }

    pub fn controller(mut self, controller: ScrollController) -> Self {
        self.controller = Some(controller);
        self
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Size) -> Self {
        self.height = height;
        self
    }

    pub fn expanded(self) -> Self {
        self.width(Size::fill()).height(Size::fill())
    }
}

impl ChildrenExt for ScrollView {
    fn children_mut(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

impl KeyExt for ScrollView {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

fn thumb(viewport: f32, content: f32, scrolled: f32, min_length: f32) -> Option<(f32, f32)> {
    if viewport <= 0.0 || content <= viewport {
        return None;
    }
    let length = (viewport * viewport / content)
        .max(min_length)
        .min(viewport);
    let travel = (-scrolled / (content - viewport)).clamp(0.0, 1.0);
    Some((travel * (viewport - length), length))
}

fn track_click(
    click: ScrollBarClick,
    at: f32,
    thumb: (f32, f32),
    track: f32,
    offset: i32,
    viewport: f32,
    content: f32,
) -> i32 {
    let (start, length) = thumb;
    match click {
        ScrollBarClick::NextPage if at < start => {
            clamped(offset, viewport.into(), viewport, content)
        }
        ScrollBarClick::NextPage if at > start + length => {
            clamped(offset, (-viewport).into(), viewport, content)
        }
        ScrollBarClick::NextPage => offset,
        ScrollBarClick::ClickedSpot => {
            let travel = ((at - length / 2.0) / (track - length).max(1.0)).clamp(0.0, 1.0);
            -(travel * (content - viewport).max(0.0)).round() as i32
        }
    }
}

fn bar_rect(
    style: &ScrollViewStyle,
    horizontal: bool,
    offset: f32,
    length: f32,
    breadth: f32,
) -> Rect {
    let (position, width, height) = if horizontal {
        (
            Position::new_absolute().left(offset).bottom(style.inset),
            length,
            breadth,
        )
    } else {
        (
            Position::new_absolute().top(offset).right(style.inset),
            breadth,
            length,
        )
    };
    rect()
        .position(position)
        .layer(THUMB_LAYER)
        .width(Size::px(width))
        .height(Size::px(height))
        .corner_radius(
            style
                .thumb_shape
                .corner_radius(width, height, LayoutDirection::LeftToRight),
        )
}

#[derive(Clone, Copy)]
enum FadedEdge {
    Top,
    Left,
    Right,
}

fn edge_fade(style: &ScrollViewStyle, edge: FadedEdge) -> Rect {
    let paints = Paints::current(ControlState::default(), style.effect);
    let ground = paints.color(style.edge.resolve(ControlState::default()));
    let (position, angle) = match edge {
        FadedEdge::Top => (Position::new_absolute().top(0.0).left(0.0), 0.0),
        FadedEdge::Left => (Position::new_absolute().top(0.0).left(0.0), -90.0),
        FadedEdge::Right => (Position::new_absolute().top(0.0).right(0.0), 90.0),
    };
    let fade = rect()
        .position(position)
        .layer(THUMB_LAYER)
        .interactive(Interactive::No)
        .background(
            LinearGradient::new()
                .angle(angle)
                .stop((color(ground), 0.0))
                .stop((color(Rgba { a: 0.0, ..ground }), 100.0)),
        );
    match edge {
        FadedEdge::Top => fade
            .width(Size::percent(100.0))
            .height(Size::px(style.edge_height)),
        FadedEdge::Left | FadedEdge::Right => fade
            .width(Size::px(style.edge_height))
            .height(Size::percent(100.0)),
    }
}

impl Component for ScrollView {
    fn render(&self) -> impl IntoElement {
        let style: ScrollViewStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default(), style.effect);
        let ink =
            |paint: &StateValue<ShapeStyle>| paints.color(paint.resolve(ControlState::default()));
        let resolved = crate::theme::resolved();
        let always = match resolved.scroll_bars {
            ScrollBars::Always => true,
            ScrollBars::WhenScrolling => false,
            ScrollBars::Automatic => SCROLLED_WITH_A_MOUSE.load(Ordering::Relaxed),
        };
        let own = use_scroll_controller();
        let controller = self.controller.unwrap_or(own);
        let horizontal = self.direction == Direction::Horizontal;
        let (mut viewport, mut content) = (controller.viewport, controller.content);
        let (scrolled_x, scrolled_y): (i32, i32) = controller.into();
        let scrolled = if horizontal { scrolled_x } else { scrolled_y };
        let along = move |(width, height): (f32, f32)| if horizontal { width } else { height };
        let size = |area: Area| (area.width(), area.height());
        let mut shown_at = use_state(|| scrolled);
        let mut scrolls = use_state(|| 0u32);
        let mut scrolling = use_state(|| false);
        if *shown_at.peek() != scrolled {
            shown_at.set(scrolled);
            let scroll = *scrolls.peek() + 1;
            scrolls.set(scroll);
            scrolling.set(true);
            if let Some(timers) = try_consume_root_context::<Timers>() {
                timers.after(Duration::from_millis(style.hide_delay_ms), move || {
                    if scrolls.try_peek().is_some_and(|last| *last == scroll) {
                        let mut scrolling = scrolling;
                        scrolling.set(false);
                    }
                });
            }
        }
        let fade =
            crate::animation::use_animated(f32::from(u8::from(*scrolling.read())), style.animation);
        let visibility = if always { 1.0 } else { fade };
        let (breadth, opacity, edge) = if always {
            (style.track_width, style.always_thumb_opacity, style.inset)
        } else {
            (style.thumb_width, style.thumb_opacity, 0.0)
        };
        let track_length = along(*viewport.read()) - 2.0 * edge;
        let bar = thumb(
            track_length,
            along(*content.read()) - 2.0 * edge,
            scrolled as f32,
            style.min_thumb_length,
        )
        .filter(|_| self.scroll_bar && visibility > 0.0);
        rect()
            .width(self.width.clone())
            .height(self.height.clone())
            .on_sized(move |event: Event<SizedEventData>| {
                viewport.set_if_modified(size(event.area))
            })
            .child(
                Scroller::new_controlled(controller.inner)
                    .direction(self.direction)
                    .show_scrollbar(false)
                    .child(
                        rect()
                            .direction(self.direction)
                            .maybe(!horizontal, |el| el.width(Size::fill()))
                            .on_sized(move |event: Event<SizedEventData>| {
                                content.set_if_modified(size(event.area))
                            })
                            .children(self.children.clone()),
                    ),
            )
            .maybe_child((self.edge_effect && scrolled < 0).then(|| {
                if horizontal {
                    edge_fade(&style, FadedEdge::Left)
                } else {
                    edge_fade(&style, FadedEdge::Top)
                }
            }))
            .maybe_child(
                (self.edge_effect
                    && horizontal
                    && along(*content.read()) - along(*viewport.read()) + scrolled as f32 >= 1.0)
                    .then(|| edge_fade(&style, FadedEdge::Right)),
            )
            .maybe_child(self.top_line.then(|| {
                Divider::new()
                    .position(Position::new_absolute().top(0.0).left(0.0))
                    .layer(THUMB_LAYER)
                    .background(color(ink(&style.top_line)))
            }))
            .maybe_child(bar.filter(|_| always).map(|thumb| {
                let click = resolved.scroll_bar_click;
                let (shown, whole) = (along(*viewport.peek()), along(*content.peek()));
                let mut inner = controller.inner;
                bar_rect(&style, horizontal, edge, track_length, breadth)
                    .background(color(Rgba {
                        a: style.track_opacity,
                        ..ink(&style.track)
                    }))
                    .on_mouse_down(move |event: Event<MouseEventData>| {
                        event.stop_propagation();
                        let at = if horizontal {
                            event.element_location.x
                        } else {
                            event.element_location.y
                        } as f32;
                        let next =
                            track_click(click, at, thumb, track_length, scrolled, shown, whole);
                        if horizontal {
                            inner.scroll_to_x(next);
                        } else {
                            inner.scroll_to_y(next);
                        }
                    })
            }))
            .maybe_child(bar.map(|(offset, length)| {
                bar_rect(&style, horizontal, edge + offset, length, breadth).background(color(
                    Rgba {
                        a: opacity * visibility,
                        ..ink(&style.thumb)
                    },
                ))
            }))
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}

#[cfg(test)]
mod tests {
    use ctdesign::SCROLL_BAR;

    use super::*;

    #[test]
    fn revealing_scrolls_only_as_far_as_the_range_needs() {
        assert_eq!(revealed_offset(0, 10.0, 30.0, 100.0), 0);
        assert_eq!(revealed_offset(0, 150.0, 170.0, 100.0), -70);
        assert_eq!(revealed_offset(-200, 50.0, 70.0, 100.0), -50);
    }

    #[test]
    fn a_forwarded_wheel_stops_at_both_ends() {
        assert_eq!(clamped(0, 30.0, 200.0, 800.0), 0);
        assert_eq!(clamped(-590, -30.0, 200.0, 800.0), -600);
        assert_eq!(clamped(0, -30.0, 200.0, 100.0), 0);
        assert_eq!(clamped(-100, -30.0, 200.0, 800.0), -130);
    }

    #[test]
    fn a_click_beside_the_thumb_pages_toward_it_and_a_click_on_it_does_nothing() {
        let page = |at| {
            track_click(
                ScrollBarClick::NextPage,
                at,
                (40.0, 20.0),
                100.0,
                -200,
                100.0,
                500.0,
            )
        };
        assert_eq!(page(10.0), -100);
        assert_eq!(page(90.0), -300);
        assert_eq!(page(50.0), -200);
        assert_eq!(
            track_click(
                ScrollBarClick::NextPage,
                95.0,
                (70.0, 20.0),
                100.0,
                -350,
                100.0,
                500.0
            ),
            -400
        );
    }

    #[test]
    fn a_click_on_the_spot_centres_the_thumb_there() {
        let spot = |at| {
            track_click(
                ScrollBarClick::ClickedSpot,
                at,
                (0.0, 20.0),
                100.0,
                0,
                100.0,
                500.0,
            )
        };
        assert_eq!(spot(10.0), 0);
        assert_eq!(spot(50.0), -200);
        assert_eq!(spot(95.0), -400);
    }

    #[test]
    fn content_that_fits_has_no_thumb() {
        assert_eq!(thumb(200.0, 200.0, 0.0, SCROLL_BAR.min_length), None);
        assert_eq!(thumb(0.0, 500.0, 0.0, SCROLL_BAR.min_length), None);
    }

    #[test]
    fn the_thumb_spans_the_track_from_top_to_bottom() {
        let (top, length) = thumb(200.0, 800.0, 0.0, SCROLL_BAR.min_length).unwrap();
        assert_eq!((top, length), (0.0, 50.0));
        let (bottom, _) = thumb(200.0, 800.0, -600.0, SCROLL_BAR.min_length).unwrap();
        assert_eq!(bottom, 150.0);
    }
}

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use ctappearance::{ScrollBarClick, ScrollBars};
use ctdesign::{Rgba, SCROLL_PHYSICS};
use freya_components::scrollviews::{
    ScrollConfig, ScrollController as Inner, ScrollView as Scroller,
};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Area, Direction, Gaps, Position, Size};

use crate::adornments::Divider;
use crate::elements::{Overrides, forward_overrides};
use crate::gesture::{DragGesture, DragValue};
use crate::i18n::text;
use crate::modifiers::ViewModifiers;
use crate::platform::Timers;
use crate::progress::Spinner;
use crate::scroll_physics::{Glide, VelocityTracker, rubber_band, rubber_band_input};
use crate::styles::{
    ControlState, LayoutDirection, Paints, ScrollViewStyle, ShapeStyle, StateValue, Theme, resolve,
};
use crate::theme::color;

mod position;

pub use position::{IdentityModifiers, ScrollGeometry, ScrollPhase, ScrollPosition};
use position::{Targets, use_scroll_events, use_scroll_position};

// Freya paints by tree depth; the thumb must sit above the scrolled content, like Freya's own bar.
pub(crate) const THUMB_LAYER: i16 = 999;

pub(crate) static SCROLLED_WITH_A_MOUSE: AtomicBool = AtomicBool::new(false);

#[derive(Clone, PartialEq)]
pub struct ScrollView {
    children: Vec<Element>,
    direction: Direction,
    scroll_bar: bool,
    autohides: bool,
    top_line: bool,
    edge_effect: bool,
    refresh: Option<Refresh>,
    controller: Option<ScrollController>,
    position: Option<State<ScrollPosition>>,
    on_geometry_change: Option<Callback<ScrollGeometry, ()>>,
    on_phase_change: Option<Callback<(ScrollPhase, ScrollPhase), ()>>,
    style: Theme,
    key: DiffKey,
    overrides: Overrides,
}

#[derive(Clone, Copy, PartialEq)]
pub struct ScrollController {
    inner: Inner,
    viewport: State<(f32, f32)>,
    content: State<(f32, f32)>,
    phase: State<ScrollPhase>,
    over: State<f32>,
    motion: State<Option<TaskHandle>>,
    animation: State<Option<TaskHandle>>,
    targets: State<Targets>,
}

pub fn use_scroll_controller() -> ScrollController {
    let inner = freya_components::scrollviews::use_scroll_controller(ScrollConfig::default);
    let viewport = use_state(|| (0.0, 0.0));
    let content = use_state(|| (0.0, 0.0));
    ScrollController {
        inner,
        viewport,
        content,
        phase: use_state(ScrollPhase::default),
        over: use_state(|| 0.0),
        motion: use_state(|| None),
        animation: use_state(|| None),
        targets: use_state(Targets::default),
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
    pub(crate) fn raw(self) -> Inner {
        self.inner
    }

    pub(crate) fn set_viewport(mut self, size: (f32, f32)) {
        self.viewport.set_if_modified(size);
    }

    pub(crate) fn set_content(mut self, size: (f32, f32)) {
        self.content.set_if_modified(size);
    }

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

// A touchpad scroll past an edge stretches the content and a release glides (ADR 0104). The rest offset stays
// in Freya's controller; the stretch is a shift of the whole view, so nothing else reads it. While the view
// is refreshing the start edge is `refresh_hold` above the content, so the offset may be negative.
#[derive(Clone)]
pub(crate) struct Elastic {
    over: State<f32>,
    glide: State<Option<TaskHandle>>,
    reduced_motion: State<bool>,
    pose: Rc<Cell<(f32, f32)>>,
    tracker: Rc<RefCell<VelocityTracker>>,
    began: Rc<Instant>,
    refresh: Rc<RefCell<Option<Refresh>>>,
    dragging: Rc<Cell<bool>>,
    fired: Rc<Cell<bool>>,
    touching: Rc<Cell<bool>>,
    panned: Rc<Cell<f32>>,
}

type Refresh = (bool, Callback<(), ()>);

pub(crate) fn use_elastic(controller: ScrollController, refresh: Option<Refresh>) -> Elastic {
    let elastic = Elastic {
        over: controller.over,
        glide: controller.motion,
        reduced_motion: crate::animation::use_reduced_motion(),
        pose: use_hook(|| Rc::new(Cell::new((0.0, 0.0)))),
        tracker: use_hook(|| Rc::new(RefCell::new(VelocityTracker::default()))),
        began: use_hook(|| Rc::new(Instant::now())),
        refresh: use_hook(|| Rc::new(RefCell::new(None))),
        dragging: use_hook(|| Rc::new(Cell::new(false))),
        fired: use_hook(|| Rc::new(Cell::new(false))),
        touching: use_hook(|| Rc::new(Cell::new(false))),
        panned: use_hook(|| Rc::new(Cell::new(0.0))),
    };
    let refreshing = refresh.as_ref().is_some_and(|(refreshing, _)| *refreshing);
    *elastic.refresh.borrow_mut() = refresh;
    let held = elastic.clone();
    // A task runs outside the effect's reactive context, so the offset it reads does not rerun the effect.
    use_side_effect_with_deps(&refreshing, move |_| {
        let held = held.clone();
        spawn(async move { held.retarget(controller) });
    });
    elastic
}

impl Elastic {
    // The shift of the content along the axis, negative past the far edge.
    pub(crate) fn shift(&self) -> f32 {
        -*self.over.read()
    }

    // How far the content is pulled down from its start, for the indicator of a refresh.
    pub(crate) fn pulled(&self) -> Option<f32> {
        self.refresh
            .borrow()
            .as_ref()
            .map(|_| (-*self.over.read()).max(0.0))
    }

    pub(crate) fn refreshing(&self) -> bool {
        self.refresh
            .borrow()
            .as_ref()
            .is_some_and(|(refreshing, _)| *refreshing)
    }

    pub(crate) fn wheel(
        &self,
        controller: ScrollController,
        horizontal: bool,
    ) -> impl Fn(Event<WheelEventData>) + 'static {
        let this = self.clone();
        move |event| this.handle(controller, horizontal, &event)
    }

    fn handle(
        &self,
        controller: ScrollController,
        horizontal: bool,
        event: &Event<WheelEventData>,
    ) {
        if event.source != WheelSource::Pixel || *self.reduced_motion.peek() {
            return;
        }
        let along = |(width, height): (f32, f32)| if horizontal { width } else { height };
        let viewport = along(*controller.viewport.peek());
        let limit = (along(*controller.content.peek()) - viewport).max(0.0);
        if limit == 0.0 && *self.over.peek() == 0.0 && self.refresh.borrow().is_none() {
            return;
        }
        event.stop_propagation();
        let delta = -(if horizontal {
            event.delta_x
        } else {
            event.delta_y
        }) as f32;
        let span = (viewport, limit);
        match event.phase {
            TouchPhase::Started => {
                self.grab(controller, horizontal, viewport);
                self.drag(controller, horizontal, span, delta);
            }
            TouchPhase::Moved => self.drag(controller, horizontal, span, delta),
            TouchPhase::Ended => self.release(controller, horizontal, limit, true),
            TouchPhase::Cancelled => self.release(controller, horizontal, limit, false),
        }
    }

    // The pan of a finger, as a gesture of the lowest priority on the content (ADR 0105). The finger's
    // velocity at the release is the glide's, whichever platform: only a wheel has system momentum.
    pub(crate) fn pan(&self, controller: ScrollController, horizontal: bool) -> DragGesture {
        let (changed, ended) = (self.clone(), self.clone());
        let (changed_controller, last) = (controller, self.panned.clone());
        let started = last.clone();
        let along = move |(width, height): (f32, f32)| if horizontal { width } else { height };
        let span = move |controller: ScrollController| {
            let viewport = along(*controller.viewport.peek());
            (
                viewport,
                (along(*controller.content.peek()) - viewport).max(0.0),
            )
        };
        DragGesture::new()
            .touches_only()
            .on_changed(move |value: DragValue| {
                let (viewport, limit) = span(changed_controller);
                let travel = if horizontal {
                    value.translation.0
                } else {
                    value.translation.1
                };
                if !changed.touching.get() {
                    changed.touching.set(true);
                    changed.grab(changed_controller, horizontal, viewport);
                    started.set(0.0);
                }
                let delta = -(travel - last.get());
                last.set(travel);
                if *changed.reduced_motion.peek() {
                    changed_controller.set_phase(ScrollPhase::Interacting);
                    let (rest, _) = changed.pose.get();
                    let rest = (rest + delta).clamp(0.0, limit);
                    changed.pose.set((rest, 0.0));
                    scroll_to(changed_controller, horizontal, rest);
                } else {
                    changed.drag(changed_controller, horizontal, (viewport, limit), delta);
                }
            })
            .on_ended(move |value: DragValue| {
                let (_, limit) = span(controller);
                let speed = -(if horizontal {
                    value.velocity.0
                } else {
                    value.velocity.1
                });
                ended.touching.set(false);
                if *ended.reduced_motion.peek() {
                    controller.set_phase(ScrollPhase::Idle);
                    return;
                }
                ended.release_at(controller, horizontal, limit, speed);
            })
    }

    fn now(&self) -> f32 {
        self.began.elapsed().as_secs_f32() * 1000.0
    }

    // The lowest offset: the start edge, or the held position of a refresh.
    fn low(&self) -> f32 {
        if self.refreshing() {
            -SCROLL_PHYSICS.refresh_hold
        } else {
            0.0
        }
    }

    // The pull down to a refresh is looser than any other stretch.
    fn band(&self, raw: f32) -> f32 {
        let pulling = self
            .refresh
            .borrow()
            .as_ref()
            .is_some_and(|(refreshing, _)| !refreshing);
        if pulling && raw < 0.0 {
            SCROLL_PHYSICS.refresh_band
        } else if self.touching.get() {
            SCROLL_PHYSICS.touch_band
        } else {
            SCROLL_PHYSICS.band
        }
    }

    fn stop(&self) {
        let task = self.glide.clone().write().take();
        if let Some(task) = task {
            task.cancel();
        }
    }

    fn grab(&self, controller: ScrollController, horizontal: bool, viewport: f32) {
        self.stop();
        controller.stop_animation();
        controller.set_phase(ScrollPhase::Tracking);
        self.tracker.borrow_mut().clear();
        self.dragging.set(true);
        self.fired.set(false);
        let inner: (i32, i32) = controller.into();
        let inner = -(if horizontal { inner.0 } else { inner.1 }) as f32;
        let (mut rest, _) = self.pose.get();
        if rest.max(0.0).round() != inner.round() {
            rest = inner;
        }
        let stretch = *self.over.peek() - rest.min(0.0);
        self.pose.set((
            rest,
            rubber_band_input(stretch, viewport, self.band(stretch)),
        ));
    }

    fn drag(
        &self,
        controller: ScrollController,
        horizontal: bool,
        (viewport, limit): (f32, f32),
        delta: f32,
    ) {
        controller.set_phase(ScrollPhase::Interacting);
        self.tracker.borrow_mut().push(self.now(), delta);
        let (rest, raw) = self.pose.get();
        let position = rest + raw + delta;
        let rest = position.clamp(self.low(), limit);
        let raw = position - rest;
        self.pose.set((rest, raw));
        scroll_to(controller, horizontal, rest.max(0.0));
        let shown = rest.min(0.0) + rubber_band(raw, viewport, self.band(raw));
        let mut over = self.over;
        over.set(shown);
        let refresh = self.refresh.borrow().clone();
        if let Some((false, action)) = refresh
            && shown < -SCROLL_PHYSICS.refresh_trigger
            && !self.fired.replace(true)
        {
            action.call(());
        }
    }

    fn release(&self, controller: ScrollController, horizontal: bool, limit: f32, flung: bool) {
        let speed = if flung && !crate::platform::SYSTEM_MOMENTUM {
            self.tracker.borrow().velocity(self.now())
        } else {
            0.0
        };
        self.release_at(controller, horizontal, limit, speed);
    }

    fn release_at(&self, controller: ScrollController, horizontal: bool, limit: f32, speed: f32) {
        self.dragging.set(false);
        self.tracker.borrow_mut().clear();
        let (rest, _) = self.pose.get();
        let (low, shown) = (self.low(), *self.over.peek());
        let glide = Glide::new(rest - low, speed, limit - low, shown - rest.min(0.0));
        self.run(glide, low, controller, horizontal);
    }

    // The refreshing state changed: the start edge moved, and the content returns to the edge it now has.
    fn retarget(&self, controller: ScrollController) {
        let (viewport, content) = (*controller.viewport.peek(), *controller.content.peek());
        let (low, shown) = (self.low(), *self.over.peek());
        if self.dragging.get() {
            self.pose.set((
                low,
                rubber_band_input(shown - low, viewport.1, SCROLL_PHYSICS.band),
            ));
            return;
        }
        self.stop();
        let inner: (i32, i32) = controller.into();
        if inner.1 < 0 {
            self.pose.set((-inner.1 as f32, 0.0));
            return;
        }
        let limit = (content.1 - viewport.1).max(0.0);
        self.run(
            Glide::new(0.0, 0.0, limit - low, shown - low),
            low,
            controller,
            false,
        );
    }

    fn run(&self, glide: Glide, low: f32, controller: ScrollController, horizontal: bool) {
        let (over, pose) = (self.over, self.pose.clone());
        let finish = move |rest: f32, stretch: f32| {
            pose.set((rest, 0.0));
            scroll_to(controller, horizontal, rest.max(0.0));
            let mut over = over;
            over.set(rest.min(0.0) + stretch);
        };
        if glide.at(0.0).done {
            finish(glide.at(0.0).rest + low, 0.0);
            controller.set_phase(ScrollPhase::Idle);
            return;
        }
        controller.set_phase(ScrollPhase::Decelerating);
        let (ticker, platform, clock) = (
            RenderingTicker::get(),
            Platform::get().current_window(),
            AnimationClock::get(),
        );
        let mut task = self.glide;
        let handle = spawn(async move {
            let (mut at, mut previous) = (0.0, Instant::now());
            loop {
                ticker.tick().await;
                at += clock
                    .correct_elapsed_duration(previous.elapsed())
                    .as_secs_f32()
                    * 1000.0;
                previous = Instant::now();
                let position = glide.at(at);
                finish(position.rest + low, position.over);
                if position.done {
                    finish(position.rest + low, 0.0);
                    break;
                }
                platform.send(UserEvent::RequestRedraw);
            }
            controller.set_phase(ScrollPhase::Idle);
            task.write().take();
        });
        task.write().replace(handle);
    }
}

fn scroll_to(controller: ScrollController, horizontal: bool, rest: f32) {
    let mut inner = controller.inner;
    let to = -rest.round() as i32;
    if horizontal {
        inner.scroll_to_x(to);
    } else {
        inner.scroll_to_y(to);
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
            direction: Direction::Vertical,
            scroll_bar: true,
            autohides: true,
            top_line: false,
            edge_effect: false,
            refresh: None,
            controller: None,
            position: None,
            on_geometry_change: None,
            on_phase_change: None,
            style: Theme::tengen(),
            key: DiffKey::None,
            overrides: Overrides::default(),
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

    // Pulling the content down past its start calls `action`; it stays pulled while `refreshing` is true.
    pub fn refreshable(mut self, refreshing: bool, action: impl Into<Callback<(), ()>>) -> Self {
        self.refresh = Some((refreshing, action.into()));
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

    // NSTextView's scroll view keeps its legacy track while the text fits; SwiftUI's ScrollView hides it.
    pub(crate) fn autohides(mut self, autohides: bool) -> Self {
        self.autohides = autohides;
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

    pub fn scroll_position(mut self, position: State<ScrollPosition>) -> Self {
        self.position = Some(position);
        self
    }

    pub fn on_scroll_geometry_change(
        mut self,
        action: impl Into<Callback<ScrollGeometry, ()>>,
    ) -> Self {
        self.on_geometry_change = Some(action.into());
        self
    }

    pub fn on_scroll_phase_change(
        mut self,
        action: impl Into<Callback<(ScrollPhase, ScrollPhase), ()>>,
    ) -> Self {
        self.on_phase_change = Some(action.into());
        self
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

pub(crate) struct ScrollBarParts {
    pub(crate) reserved: f32,
    pub(crate) track: Option<Rect>,
    pub(crate) thumb: Option<Rect>,
}

// The bar of a scroll view: the legacy one that is always shown beside the content and the overlay one that
// fades in while scrolling. A view that draws its own content (a virtual table) takes it from here too.
pub(crate) fn use_scroll_bar(
    controller: ScrollController,
    horizontal: bool,
    enabled: bool,
    autohides: bool,
    style: &ScrollViewStyle,
) -> ScrollBarParts {
    let paints = Paints::current(ControlState::default(), style.effect);
    let ink = |paint: &StateValue<ShapeStyle>| paints.color(paint.resolve(ControlState::default()));
    let resolved = crate::theme::resolved();
    let always = match resolved.scroll_bars {
        ScrollBars::Always => true,
        ScrollBars::WhenScrolling => false,
        ScrollBars::Automatic => SCROLLED_WITH_A_MOUSE.load(Ordering::Relaxed),
    };
    let (viewport, content) = (controller.viewport, controller.content);
    let (scrolled_x, scrolled_y): (i32, i32) = controller.into();
    let scrolled = if horizontal { scrolled_x } else { scrolled_y };
    let along = move |(width, height): (f32, f32)| if horizontal { width } else { height };
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
        (style.track_width, style.thumb_opacity, style.inset)
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
    .filter(|_| enabled && visibility > 0.0);
    let tracked = always && enabled && (bar.is_some() || !autohides);
    let reserved = if tracked {
        style.track_width + 2.0 * style.inset
    } else {
        0.0
    };
    let track = tracked.then(|| {
        let click = resolved.scroll_bar_click;
        let (shown, whole) = (along(*viewport.peek()), along(*content.peek()));
        let mut inner = controller.inner;
        bar_rect(style, horizontal, edge, track_length, breadth)
            .background(color(Rgba {
                a: style.track_opacity,
                ..ink(&style.track)
            }))
            .on_mouse_down(move |event: Event<MouseEventData>| {
                event.stop_propagation();
                let Some(thumb) = bar else {
                    return;
                };
                let at = if horizontal {
                    event.element_location.x
                } else {
                    event.element_location.y
                } as f32;
                let next = track_click(click, at, thumb, track_length, scrolled, shown, whole);
                if horizontal {
                    inner.scroll_to_x(next);
                } else {
                    inner.scroll_to_y(next);
                }
            })
    });
    let thumb = bar.map(|(offset, length)| {
        bar_rect(style, horizontal, edge + offset, length, breadth).background(color(Rgba {
            a: opacity * visibility,
            ..ink(&style.thumb)
        }))
    });
    ScrollBarParts {
        reserved,
        track,
        thumb,
    }
}

forward_overrides!(ScrollView);

impl Component for ScrollView {
    fn render(&self) -> impl IntoElement {
        let style: ScrollViewStyle = resolve(&self.style);
        let own = use_scroll_controller();
        let controller = self.controller.unwrap_or(own);
        provide_context(controller);
        let horizontal = self.direction == Direction::Horizontal;
        let (mut viewport, mut content) = (controller.viewport, controller.content);
        let (scrolled_x, scrolled_y): (i32, i32) = controller.into();
        let scrolled = if horizontal { scrolled_x } else { scrolled_y };
        let along = move |(width, height): (f32, f32)| if horizontal { width } else { height };
        let size = |area: Area| (area.width(), area.height());
        let bars = use_scroll_bar(
            controller,
            horizontal,
            self.scroll_bar,
            self.autohides,
            &style,
        );
        let elastic = use_elastic(controller, self.refresh.clone());
        use_scroll_position(controller, self.position);
        use_scroll_events(
            controller,
            self.on_geometry_change.clone(),
            self.on_phase_change.clone(),
        );
        let frames = try_consume_root_context::<crate::titlebar::ScrollFrames>();
        use_drop(move || {
            if let Some(frames) = frames {
                frames.forget(viewport);
            }
        });
        let reserved = bars.reserved;
        let root = rect()
            .expanded()
            .on_sized(move |event: Event<SizedEventData>| {
                viewport.set_if_modified(size(event.area));
                if let Some(frames) = frames {
                    frames.report(viewport, event.area);
                }
            })
            .child(
                rect()
                    .expanded()
                    .overflow(Overflow::Clip)
                    .maybe(horizontal, |el| el.offset_x(elastic.shift()))
                    .maybe(!horizontal, |el| el.offset_y(elastic.shift()))
                    .child(
                        Scroller::new_controlled(controller.inner)
                            .direction(self.direction)
                            .margin(if horizontal {
                                Gaps::new(0.0, 0.0, reserved, 0.0)
                            } else {
                                Gaps::new(0.0, reserved, 0.0, 0.0)
                            })
                            .show_scrollbar(false)
                            .drag_scrolling(false)
                            .child(
                                rect()
                                    .direction(self.direction)
                                    .maybe(!horizontal, |el| el.width(Size::fill()))
                                    .on_wheel(elastic.wheel(controller, horizontal))
                                    .gesture(elastic.pan(controller, horizontal))
                                    .on_sized(move |event: Event<SizedEventData>| {
                                        content.set_if_modified(size(event.area));
                                        controller.set_origin(event.area);
                                    })
                                    .children(self.children.clone()),
                            ),
                    ),
            )
            .maybe_child(
                elastic
                    .pulled()
                    .filter(|pulled| *pulled > 0.0)
                    .map(|pulled| {
                        rect()
                            .position(Position::new_absolute().top(0.0).left(0.0))
                            .width(Size::fill())
                            .height(Size::px(pulled))
                            .overflow(Overflow::Clip)
                            .main_align(Alignment::Center)
                            .cross_align(Alignment::Center)
                            .child(Spinner::new(text!("refreshing")))
                    }),
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
                let paints = Paints::current(ControlState::default(), style.effect);
                Divider::new()
                    .position(Position::new_absolute().top(0.0).left(0.0))
                    .layer(THUMB_LAYER)
                    .background(color(
                        paints.color(style.top_line.resolve(ControlState::default())),
                    ))
            }))
            .maybe_child(bars.track)
            .maybe_child(bars.thumb);
        self.overrides.apply(root)
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}

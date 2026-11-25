use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use freya_core::prelude::*;
use torin::prelude::{Alignment, Area};

use crate::animation::{effective, transaction_at};
use crate::scroll::{ScrollController, clamped, revealed_offset};
use crate::styles::{Animation, Edge};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollPhase {
    #[default]
    Idle,
    Tracking,
    Interacting,
    Decelerating,
    Animating,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollGeometry {
    pub content_offset: (f32, f32),
    pub content_size: (f32, f32),
    pub container_size: (f32, f32),
}

#[derive(Clone, Debug, PartialEq)]
enum Target {
    Point((f32, f32)),
    Edge(Edge),
    View(String, Option<Alignment>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollPosition {
    target: Target,
    positioned_by_user: bool,
}

impl Default for ScrollPosition {
    fn default() -> Self {
        Self::at(0.0, 0.0)
    }
}

impl ScrollPosition {
    pub fn at(x: f32, y: f32) -> Self {
        Self::of(Target::Point((x, y)))
    }

    pub fn edge(edge: Edge) -> Self {
        Self::of(Target::Edge(edge))
    }

    pub fn id(id: impl Into<String>, anchor: Option<Alignment>) -> Self {
        Self::of(Target::View(id.into(), anchor))
    }

    pub fn is_positioned_by_user(&self) -> bool {
        self.positioned_by_user
    }

    pub fn point(&self) -> Option<(f32, f32)> {
        match self.target {
            Target::Point(point) => Some(point),
            _ => None,
        }
    }

    fn of(target: Target) -> Self {
        Self {
            target,
            positioned_by_user: false,
        }
    }

    fn by_user(offset: (i32, i32)) -> Self {
        Self {
            target: Target::Point((-offset.0 as f32, -offset.1 as f32)),
            positioned_by_user: true,
        }
    }
}

#[derive(Clone, Default, PartialEq)]
pub(crate) struct Targets {
    origin: (f32, f32),
    areas: HashMap<String, Area>,
}

fn anchored_offset(
    offset: i32,
    start: f32,
    extent: f32,
    viewport: f32,
    anchor: Option<f32>,
) -> i32 {
    match anchor {
        Some(anchor) => -(start + (extent - viewport) * anchor).round() as i32,
        None => revealed_offset(offset, start, start + extent, viewport),
    }
}

fn anchor_fraction(anchor: &Option<Alignment>) -> Option<f32> {
    match anchor {
        Some(Alignment::Start) => Some(0.0),
        Some(Alignment::Center) => Some(0.5),
        Some(Alignment::End) => Some(1.0),
        _ => None,
    }
}

impl ScrollController {
    pub fn content_offset(self) -> (f32, f32) {
        let (x, y): (i32, i32) = self.into();
        (-x as f32, -y as f32)
    }

    pub fn geometry(self) -> ScrollGeometry {
        ScrollGeometry {
            content_offset: self.content_offset(),
            content_size: *self.content.read(),
            container_size: *self.viewport.read(),
        }
    }

    pub fn phase(self) -> ScrollPhase {
        *self.phase.read()
    }

    pub fn scroll_to(self, position: ScrollPosition, animation: Animation) {
        if let Some(to) = self.target_of(&position) {
            self.move_to(to, animation);
        }
    }

    pub(crate) fn set_phase(mut self, phase: ScrollPhase) {
        self.phase.set_if_modified(phase);
    }

    pub(crate) fn set_origin(mut self, content: Area) {
        self.targets.write().origin = (content.min_x(), content.min_y());
    }

    fn set_target(mut self, id: &str, area: Area) {
        self.targets.write().areas.insert(id.to_owned(), area);
    }

    fn remove_target(mut self, id: &str) {
        self.targets.write().areas.remove(id);
    }

    fn target_of(self, position: &ScrollPosition) -> Option<(i32, i32)> {
        let (viewport, content) = (*self.viewport.peek(), *self.content.peek());
        let current: (i32, i32) = self.into();
        let clamp = |(x, y): (i32, i32)| {
            (
                clamped(x, 0.0, viewport.0, content.0),
                clamped(y, 0.0, viewport.1, content.1),
            )
        };
        match &position.target {
            Target::Point((x, y)) => Some(clamp((-x.round() as i32, -y.round() as i32))),
            Target::Edge(Edge::Top) => Some(clamp((current.0, 0))),
            Target::Edge(Edge::Bottom) => Some(clamp((current.0, i32::MIN))),
            Target::Edge(Edge::Leading) => Some(clamp((0, current.1))),
            Target::Edge(Edge::Trailing) => Some(clamp((i32::MIN, current.1))),
            Target::View(id, anchor) => {
                let targets = self.targets.peek();
                let area = targets.areas.get(id)?;
                let (x, y) = (
                    area.min_x() - targets.origin.0,
                    area.min_y() - targets.origin.1,
                );
                let anchor = anchor_fraction(anchor);
                Some(clamp((
                    anchored_offset(current.0, x, area.width(), viewport.0, anchor),
                    anchored_offset(current.1, y, area.height(), viewport.1, anchor),
                )))
            }
        }
    }

    fn jump(mut self, (x, y): (i32, i32)) {
        self.inner.scroll_to_x(x);
        self.inner.scroll_to_y(y);
    }

    pub(crate) fn stop_animation(mut self) {
        if let Some(task) = self.animation.take() {
            task.cancel();
        }
    }

    // The glide and its stretch end where they are, because the content is going elsewhere.
    fn stop_motion(mut self) {
        if let Some(task) = self.motion.take() {
            task.cancel();
        }
        self.stop_animation();
        self.over.set(0.0);
        self.set_phase(ScrollPhase::Idle);
    }

    fn move_to(self, to: (i32, i32), animation: Animation) {
        self.stop_motion();
        let animation = effective(animation, crate::theme::resolved().reduced_motion);
        let from: (i32, i32) = self.into();
        if animation.duration_ms() == 0 || from == to {
            return self.jump(to);
        }
        self.set_phase(ScrollPhase::Animating);
        let (ticker, platform, clock) = (
            RenderingTicker::get(),
            Platform::get().current_window(),
            AnimationClock::get(),
        );
        let mut task = self.animation;
        let handle = spawn(async move {
            let (mut elapsed, mut previous) = (0.0, Instant::now());
            loop {
                ticker.tick().await;
                elapsed += clock
                    .correct_elapsed_duration(previous.elapsed())
                    .as_secs_f32()
                    * 1000.0;
                previous = Instant::now();
                let fraction = (elapsed / animation.duration_ms() as f32).min(1.0);
                let along = |from: i32, to: i32| {
                    if fraction >= 1.0 {
                        to
                    } else {
                        from + ((to - from) as f32 * animation.progress(fraction)).round() as i32
                    }
                };
                self.jump((along(from.0, to.0), along(from.1, to.1)));
                if fraction >= 1.0 {
                    break;
                }
                platform.send(UserEvent::RequestRedraw);
            }
            self.set_phase(ScrollPhase::Idle);
            task.write().take();
        });
        task.write().replace(handle);
    }
}

#[derive(Clone, PartialEq)]
struct ScrollTarget {
    id: String,
    child: Element,
}

impl Component for ScrollTarget {
    fn render_key(&self) -> DiffKey {
        DiffKey::from(&self.id)
    }

    fn render(&self) -> impl IntoElement {
        let controller = try_consume_context::<ScrollController>();
        let id = self.id.clone();
        use_drop({
            let id = id.clone();
            move || {
                if let Some(controller) = controller {
                    controller.remove_target(&id);
                }
            }
        });
        rect()
            .on_sized(move |event: Event<SizedEventData>| {
                if let Some(controller) = controller {
                    controller.set_target(&id, event.area);
                }
            })
            .child(self.child.clone())
    }
}

pub trait IdentityModifiers: IntoElement + Sized {
    fn id(self, id: impl Into<String>) -> impl IntoElement {
        ScrollTarget {
            id: id.into(),
            child: self.into_element(),
        }
    }
}

impl<T: IntoElement> IdentityModifiers for T {}

pub(crate) fn use_scroll_position(
    controller: ScrollController,
    position: Option<State<ScrollPosition>>,
) {
    let applied = use_hook(|| Rc::new(RefCell::new(None::<(ScrollPosition, (i32, i32))>)));
    let settled = use_hook(|| Rc::new(Cell::new(None::<(i32, i32)>)));
    let requested = position.map(|position| position.read().clone());
    let (applying, resolved) = (applied.clone(), settled.clone());
    use_side_effect_with_deps(
        &(requested, *controller.content.read()),
        move |(requested, _)| {
            let Some(requested) = requested else { return };
            if requested.is_positioned_by_user() {
                *applying.borrow_mut() = None;
                return;
            }
            let Some(to) = controller.target_of(requested) else {
                return;
            };
            let request = Some((requested.clone(), to));
            if *applying.borrow() == request {
                return;
            }
            *applying.borrow_mut() = request;
            resolved.set(Some(to));
            controller.move_to(
                to,
                transaction_at(Instant::now()).unwrap_or(Animation::None),
            );
        },
    );
    let offset: (i32, i32) = controller.into();
    use_side_effect_with_deps(&(offset, controller.phase()), move |&(offset, phase)| {
        if let Some(mut position) = position
            && phase != ScrollPhase::Animating
            && settled.take() != Some(offset)
        {
            position.set_if_modified(ScrollPosition::by_user(offset));
        }
    });
}

pub(crate) fn use_scroll_events(
    controller: ScrollController,
    on_geometry_change: Option<Callback<ScrollGeometry, ()>>,
    on_phase_change: Option<Callback<(ScrollPhase, ScrollPhase), ()>>,
) {
    let actions = use_hook(|| Rc::new(RefCell::new((None, None))));
    *actions.borrow_mut() = (on_geometry_change, on_phase_change);
    let before = use_hook(|| Rc::new(Cell::new(ScrollPhase::Idle)));
    let geometry_actions = actions.clone();
    use_side_effect_with_deps(&controller.geometry(), move |geometry| {
        if let Some(action) = &geometry_actions.borrow().0 {
            action.call(*geometry);
        }
    });
    use_side_effect_with_deps(&controller.phase(), move |phase| {
        let old = before.replace(*phase);
        if let Some(action) = &actions.borrow().1
            && old != *phase
        {
            action.call((old, *phase));
        }
    });
}

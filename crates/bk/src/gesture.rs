use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use freya_core::prelude::*;

use crate::scroll_physics::VelocityTracker;

// SwiftUI's documented defaults; none is measured on this Mac (UIKit roadmap, U6).
const TAP_DISTANCE: f32 = 10.0;
const TAP_INTERVAL_MS: f32 = 500.0;
const LONG_PRESS_MS: f32 = 500.0;
const LONG_PRESS_DISTANCE: f32 = 10.0;
const DRAG_DISTANCE: f32 = 10.0;
const MAGNIFY_DELTA: f32 = 0.01;
const ROTATE_DELTA: f32 = 1.0_f32.to_radians();

pub type Point = (f32, f32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Down,
    Move,
    Up,
    Cancel,
}

/// One pointer or finger at an instant; times are milliseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Input {
    pub id: u64,
    pub phase: Phase,
    pub at: Point,
    pub time: f32,
}

/// The states of `UIGestureRecognizer.State`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Possible,
    Began,
    Changed,
    Ended,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Step<V> {
    Idle,
    Began(V),
    Changed(V),
    Ended(V),
    Failed,
    Cancelled,
}

pub trait Recognizer {
    type Value;

    fn feed(&mut self, input: Input) -> Step<Self::Value>;

    // The time at which `elapse` has something to say, if any.
    fn deadline(&self) -> Option<f32> {
        None
    }

    fn elapse(&mut self, _now: f32) -> Step<Self::Value> {
        Step::Idle
    }

    fn state(&self) -> State;

    // Another gesture won the pointer.
    fn fail(&mut self);
}

fn distance(a: Point, b: Point) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TapValue {
    pub location: Point,
}

#[derive(Clone, Debug)]
pub struct TapRecognizer {
    count: u32,
    taps: u32,
    down: Option<(u64, Point)>,
    last: Option<(Point, f32)>,
    state: State,
}

impl TapRecognizer {
    pub fn new(count: u32) -> Self {
        Self {
            count: count.max(1),
            taps: 0,
            down: None,
            last: None,
            state: State::Possible,
        }
    }

    fn reset(&mut self) {
        self.taps = 0;
        self.down = None;
        self.last = None;
    }
}

impl Recognizer for TapRecognizer {
    type Value = TapValue;

    fn feed(&mut self, input: Input) -> Step<TapValue> {
        match input.phase {
            Phase::Down => {
                self.state = State::Possible;
                if self.down.is_some() {
                    self.reset();
                    self.state = State::Failed;
                    return Step::Failed;
                }
                if let Some((at, time)) = self.last
                    && (input.time - time > TAP_INTERVAL_MS
                        || distance(at, input.at) > TAP_DISTANCE)
                {
                    self.taps = 0;
                }
                self.down = Some((input.id, input.at));
                Step::Idle
            }
            Phase::Move => match self.down {
                Some((id, at)) if id == input.id && distance(at, input.at) > TAP_DISTANCE => {
                    self.reset();
                    self.state = State::Failed;
                    Step::Failed
                }
                _ => Step::Idle,
            },
            Phase::Up => match self.down {
                Some((id, _)) if id == input.id => {
                    self.down = None;
                    self.taps += 1;
                    self.last = Some((input.at, input.time));
                    if self.taps == self.count {
                        self.reset();
                        self.state = State::Ended;
                        Step::Ended(TapValue { location: input.at })
                    } else {
                        Step::Idle
                    }
                }
                _ => Step::Idle,
            },
            Phase::Cancel => {
                self.reset();
                self.state = State::Failed;
                Step::Failed
            }
        }
    }

    fn deadline(&self) -> Option<f32> {
        match (self.last, self.down) {
            (Some((_, time)), None) if self.taps > 0 => Some(time + TAP_INTERVAL_MS),
            _ => None,
        }
    }

    fn elapse(&mut self, now: f32) -> Step<TapValue> {
        match self.deadline() {
            Some(deadline) if now > deadline => {
                self.reset();
                self.state = State::Failed;
                Step::Failed
            }
            _ => Step::Idle,
        }
    }

    fn state(&self) -> State {
        self.state
    }

    fn fail(&mut self) {
        self.reset();
        self.state = State::Failed;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LongPressValue {
    pub location: Point,
}

#[derive(Clone, Debug)]
pub struct LongPressRecognizer {
    duration: f32,
    distance: f32,
    down: Option<(u64, Point, f32)>,
    state: State,
}

impl LongPressRecognizer {
    pub fn new(duration_ms: f32, maximum_distance: f32) -> Self {
        Self {
            duration: duration_ms,
            distance: maximum_distance,
            down: None,
            state: State::Possible,
        }
    }
}

impl Recognizer for LongPressRecognizer {
    type Value = LongPressValue;

    fn feed(&mut self, input: Input) -> Step<LongPressValue> {
        match (input.phase, self.down) {
            (Phase::Down, None) => {
                self.state = State::Possible;
                self.down = Some((input.id, input.at, input.time));
                Step::Idle
            }
            (Phase::Move, Some((id, at, _)))
                if id == input.id && distance(at, input.at) > self.distance =>
            {
                self.fail();
                Step::Failed
            }
            (Phase::Up, Some((id, ..))) if id == input.id => {
                self.fail();
                Step::Failed
            }
            (Phase::Cancel, _) | (Phase::Down, Some(_)) => {
                self.fail();
                Step::Failed
            }
            _ => Step::Idle,
        }
    }

    fn deadline(&self) -> Option<f32> {
        self.down.map(|(_, _, time)| time + self.duration)
    }

    fn elapse(&mut self, now: f32) -> Step<LongPressValue> {
        match self.down {
            Some((_, at, time)) if now >= time + self.duration => {
                self.down = None;
                self.state = State::Ended;
                Step::Ended(LongPressValue { location: at })
            }
            _ => Step::Idle,
        }
    }

    fn state(&self) -> State {
        self.state
    }

    fn fail(&mut self) {
        self.down = None;
        self.state = State::Failed;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragValue {
    pub start_location: Point,
    pub location: Point,
    pub translation: Point,
    // Points per millisecond.
    pub velocity: Point,
}

#[derive(Default)]
pub struct DragRecognizer {
    minimum_distance: f32,
    down: Option<(u64, Point)>,
    began: bool,
    x: VelocityTracker,
    y: VelocityTracker,
    last: Point,
    state: Option<State>,
}

impl DragRecognizer {
    pub fn new(minimum_distance: f32) -> Self {
        Self {
            minimum_distance,
            ..Self::default()
        }
    }

    fn value(&self, start: Point, at: Point, time: f32) -> DragValue {
        DragValue {
            start_location: start,
            location: at,
            translation: (at.0 - start.0, at.1 - start.1),
            velocity: (self.x.velocity(time), self.y.velocity(time)),
        }
    }

    fn clear(&mut self) {
        self.down = None;
        self.began = false;
        self.x.clear();
        self.y.clear();
    }
}

impl Recognizer for DragRecognizer {
    type Value = DragValue;

    fn feed(&mut self, input: Input) -> Step<DragValue> {
        match (input.phase, self.down) {
            (Phase::Down, None) => {
                self.state = Some(State::Possible);
                self.down = Some((input.id, input.at));
                self.last = input.at;
                self.x.clear();
                self.y.clear();
                self.x.push(input.time, 0.0);
                self.y.push(input.time, 0.0);
                Step::Idle
            }
            (Phase::Move, Some((id, start))) if id == input.id => {
                self.x.push(input.time, input.at.0 - self.last.0);
                self.y.push(input.time, input.at.1 - self.last.1);
                self.last = input.at;
                let value = self.value(start, input.at, input.time);
                if self.began {
                    self.state = Some(State::Changed);
                    Step::Changed(value)
                } else if distance(start, input.at) >= self.minimum_distance {
                    self.began = true;
                    self.state = Some(State::Began);
                    Step::Began(value)
                } else {
                    Step::Idle
                }
            }
            (Phase::Up, Some((id, start))) if id == input.id => {
                let value = self.value(start, input.at, input.time);
                let began = self.began;
                self.clear();
                if began {
                    self.state = Some(State::Ended);
                    Step::Ended(value)
                } else {
                    self.state = Some(State::Failed);
                    Step::Failed
                }
            }
            (Phase::Cancel, Some(_)) => {
                let began = self.began;
                self.clear();
                if began {
                    self.state = Some(State::Cancelled);
                    Step::Cancelled
                } else {
                    self.state = Some(State::Failed);
                    Step::Failed
                }
            }
            _ => Step::Idle,
        }
    }

    fn state(&self) -> State {
        self.state.unwrap_or(State::Possible)
    }

    fn fail(&mut self) {
        self.clear();
        self.state = Some(State::Failed);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MagnifyValue {
    // The ratio of the distance between the fingers to the first one.
    pub magnification: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RotateValue {
    // Radians, positive turning from +x toward +y, from the first angle.
    pub rotation: f32,
}

// Two fingers, the first two that go down; `measure` reads the quantity of the pair.
struct Pair {
    fingers: [Option<(u64, Point)>; 2],
    first: f32,
    began: bool,
    state: State,
}

impl Pair {
    fn new() -> Self {
        Self {
            fingers: [None, None],
            first: 0.0,
            began: false,
            state: State::Possible,
        }
    }

    fn feed(
        &mut self,
        input: Input,
        measure: impl Fn(Point, Point) -> f32,
        change: impl Fn(f32, f32) -> f32,
        threshold: f32,
    ) -> Option<(State, f32)> {
        match input.phase {
            Phase::Down => {
                if let Some(slot) = self.fingers.iter_mut().find(|slot| slot.is_none()) {
                    *slot = Some((input.id, input.at));
                }
                if let [Some((_, a)), Some((_, b))] = self.fingers {
                    self.first = measure(a, b);
                    self.began = false;
                    self.state = State::Possible;
                }
                None
            }
            Phase::Move => {
                for slot in self.fingers.iter_mut().flatten() {
                    if slot.0 == input.id {
                        slot.1 = input.at;
                    }
                }
                let [Some((_, a)), Some((_, b))] = self.fingers else {
                    return None;
                };
                let amount = change(self.first, measure(a, b));
                if self.began {
                    self.state = State::Changed;
                    Some((State::Changed, amount))
                } else if (amount - neutral(&change)).abs() >= threshold {
                    self.began = true;
                    self.state = State::Began;
                    Some((State::Began, amount))
                } else {
                    None
                }
            }
            Phase::Up | Phase::Cancel => {
                let paired = self.fingers.iter().flatten().count() == 2;
                let ours = self.fingers.iter().flatten().any(|(id, _)| *id == input.id);
                if !ours {
                    return None;
                }
                let began = self.began;
                let amount = match self.fingers {
                    [Some((_, a)), Some((_, b))] if paired => change(self.first, measure(a, b)),
                    _ => 0.0,
                };
                *self = Self::new();
                if began && input.phase == Phase::Up {
                    self.state = State::Ended;
                    Some((State::Ended, amount))
                } else if began {
                    self.state = State::Cancelled;
                    Some((State::Cancelled, amount))
                } else {
                    None
                }
            }
        }
    }
}

// What `change` returns when nothing changed: 1 for a ratio, 0 for a difference.
fn neutral(change: &impl Fn(f32, f32) -> f32) -> f32 {
    change(1.0, 1.0)
}

fn pair_step<V>(result: Option<(State, f32)>, value: impl Fn(f32) -> V) -> Step<V> {
    match result {
        Some((State::Began, amount)) => Step::Began(value(amount)),
        Some((State::Changed, amount)) => Step::Changed(value(amount)),
        Some((State::Ended, amount)) => Step::Ended(value(amount)),
        Some((State::Cancelled, _)) => Step::Cancelled,
        _ => Step::Idle,
    }
}

pub struct MagnifyRecognizer {
    pair: Pair,
    minimum_delta: f32,
}

impl MagnifyRecognizer {
    pub fn new(minimum_delta: f32) -> Self {
        Self {
            pair: Pair::new(),
            minimum_delta,
        }
    }
}

impl Recognizer for MagnifyRecognizer {
    type Value = MagnifyValue;

    fn feed(&mut self, input: Input) -> Step<MagnifyValue> {
        let result = self.pair.feed(
            input,
            distance,
            |first, now| now / first.max(f32::EPSILON),
            self.minimum_delta,
        );
        pair_step(result, |magnification| MagnifyValue { magnification })
    }

    fn state(&self) -> State {
        self.pair.state
    }

    fn fail(&mut self) {
        self.pair = Pair::new();
        self.pair.state = State::Failed;
    }
}

pub struct RotateRecognizer {
    pair: Pair,
    minimum_delta: f32,
}

impl RotateRecognizer {
    pub fn new(minimum_delta: f32) -> Self {
        Self {
            pair: Pair::new(),
            minimum_delta,
        }
    }
}

fn angle(a: Point, b: Point) -> f32 {
    (b.1 - a.1).atan2(b.0 - a.0)
}

fn turned(first: f32, now: f32) -> f32 {
    let delta = (now - first) % std::f32::consts::TAU;
    if delta > std::f32::consts::PI {
        delta - std::f32::consts::TAU
    } else if delta < -std::f32::consts::PI {
        delta + std::f32::consts::TAU
    } else {
        delta
    }
}

impl Recognizer for RotateRecognizer {
    type Value = RotateValue;

    fn feed(&mut self, input: Input) -> Step<RotateValue> {
        let result = self.pair.feed(input, angle, turned, self.minimum_delta);
        pair_step(result, |rotation| RotateValue { rotation })
    }

    fn state(&self) -> State {
        self.pair.state
    }

    fn fail(&mut self) {
        self.pair = Pair::new();
        self.pair.state = State::Failed;
    }
}

/// How a gesture takes part in the arena, from the modifier that attached it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Priority {
    Normal,
    High,
    Simultaneous,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contender {
    // The nesting of the view: a descendant is deeper.
    pub depth: usize,
    pub priority: Priority,
    pub state: State,
    // It is `Possible` and can still begin at a later time (a double tap between its taps).
    pub pending: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Continue,
    Deliver,
    Hold,
    Fail,
}

fn recognized(state: State) -> bool {
    matches!(state, State::Began | State::Changed | State::Ended)
}

// SwiftUI's rule: the deepest gesture wins, a high-priority one beats its descendants, a simultaneous one
// neither wins nor loses. The winner waits while a gesture that outranks it can still begin.
pub fn verdicts(contenders: &[Contender]) -> Vec<Verdict> {
    let outranks = |a: &(usize, &Contender), b: &(usize, &Contender)| {
        let class = |c: &Contender| u8::from(c.priority == Priority::High);
        (class(a.1), a.1.depth, std::cmp::Reverse(a.0))
            > (class(b.1), b.1.depth, std::cmp::Reverse(b.0))
    };
    let competing: Vec<_> = contenders
        .iter()
        .enumerate()
        .filter(|(_, c)| c.priority != Priority::Simultaneous)
        .collect();
    let winner = competing
        .iter()
        .filter(|(_, c)| recognized(c.state))
        .copied()
        .reduce(|best, next| if outranks(&next, &best) { next } else { best });
    contenders
        .iter()
        .enumerate()
        .map(|(index, contender)| {
            if contender.priority == Priority::Simultaneous {
                return if recognized(contender.state) {
                    Verdict::Deliver
                } else {
                    Verdict::Continue
                };
            }
            let Some(winner) = winner else {
                return Verdict::Continue;
            };
            if winner.0 != index {
                let waiting = contender.pending && outranks(&(index, contender), &winner);
                return if waiting {
                    Verdict::Continue
                } else {
                    Verdict::Fail
                };
            }
            let held = competing
                .iter()
                .any(|other| other.1.pending && outranks(other, &winner));
            if held {
                Verdict::Hold
            } else {
                Verdict::Deliver
            }
        })
        .collect()
}

// What a gesture reports. `updating` is `@GestureState`: it changes while the gesture does and goes back
// when it ends, fails or is cancelled.
pub struct Callbacks<V> {
    changed: Vec<Callback<V, ()>>,
    ended: Option<Callback<V, ()>>,
    reset: Vec<Callback<(), ()>>,
}

impl<V> Default for Callbacks<V> {
    fn default() -> Self {
        Self {
            changed: Vec::new(),
            ended: None,
            reset: Vec::new(),
        }
    }
}

impl<V> Clone for Callbacks<V> {
    fn clone(&self) -> Self {
        Self {
            changed: self.changed.clone(),
            ended: self.ended.clone(),
            reset: self.reset.clone(),
        }
    }
}

/// A `@GestureState`: a state that goes back to its first value when the gesture that updates it is over.
#[derive(Clone, Copy, PartialEq)]
pub struct GestureState<T: 'static> {
    state: State2<T>,
    initial: T,
}

type State2<T> = freya_core::prelude::State<T>;

pub fn use_gesture_state<T: Clone + PartialEq + 'static>(initial: T) -> GestureState<T> {
    let state = use_state({
        let initial = initial.clone();
        move || initial
    });
    GestureState { state, initial }
}

impl<T: Clone + PartialEq + 'static> GestureState<T> {
    pub fn get(&self) -> T {
        self.state.read().clone()
    }
}

macro_rules! gesture {
    ($name:ident, $recognizer:ty, $value:ty, $new:expr, $began:literal) => {
        impl $name {
            /// Called as the gesture changes: when it begins and each time it moves.
            pub fn on_changed(mut self, callback: impl Into<Callback<$value, ()>>) -> Self {
                self.callbacks.changed.push(callback.into());
                self
            }

            pub fn on_ended(mut self, callback: impl Into<Callback<$value, ()>>) -> Self {
                self.callbacks.ended = Some(callback.into());
                self
            }

            /// Like SwiftUI's `updating`: `update` changes `state` while the gesture does, and it goes back
            /// to its first value when the gesture ends.
            pub fn updating<T: Clone + PartialEq + 'static>(
                mut self,
                state: GestureState<T>,
                update: impl Fn(&$value, &mut T) + 'static,
            ) -> Self {
                let mut target = state.state;
                self.callbacks
                    .changed
                    .push(Callback::new(move |value: $value| {
                        let mut next = target.peek().clone();
                        update(&value, &mut next);
                        target.set(next);
                    }));
                let (mut target, initial) = (state.state, state.initial);
                self.callbacks.reset.push(Callback::new(move |()| {
                    target.set(initial.clone());
                }));
                self
            }
        }

        impl Gesture for $name {
            type Recognizer = $recognizer;
            type Value = $value;

            fn recognizer(&self) -> $recognizer {
                $new(self)
            }

            fn callbacks(&self) -> Callbacks<$value> {
                self.callbacks.clone()
            }
        }
    };
}

pub trait Gesture {
    type Recognizer: Recognizer<Value = Self::Value> + 'static;
    type Value: 'static;

    fn recognizer(&self) -> Self::Recognizer;

    fn callbacks(&self) -> Callbacks<Self::Value>;
}

/// `TapGesture`: `count` taps in a row.
pub struct TapGesture {
    count: u32,
    callbacks: Callbacks<TapValue>,
}

impl TapGesture {
    pub fn new() -> Self {
        Self {
            count: 1,
            callbacks: Callbacks::default(),
        }
    }

    pub fn count(mut self, count: u32) -> Self {
        self.count = count;
        self
    }
}

impl Default for TapGesture {
    fn default() -> Self {
        Self::new()
    }
}

gesture!(
    TapGesture,
    TapRecognizer,
    TapValue,
    |g: &TapGesture| TapRecognizer::new(g.count),
    false
);

/// `LongPressGesture`: the pointer stays down in place for `minimum_duration`.
pub struct LongPressGesture {
    minimum_duration: f32,
    maximum_distance: f32,
    callbacks: Callbacks<LongPressValue>,
}

impl LongPressGesture {
    pub fn new() -> Self {
        Self {
            minimum_duration: LONG_PRESS_MS,
            maximum_distance: LONG_PRESS_DISTANCE,
            callbacks: Callbacks::default(),
        }
    }

    pub fn minimum_duration(mut self, milliseconds: f32) -> Self {
        self.minimum_duration = milliseconds;
        self
    }

    pub fn maximum_distance(mut self, distance: f32) -> Self {
        self.maximum_distance = distance;
        self
    }
}

impl Default for LongPressGesture {
    fn default() -> Self {
        Self::new()
    }
}

gesture!(
    LongPressGesture,
    LongPressRecognizer,
    LongPressValue,
    |g: &LongPressGesture| LongPressRecognizer::new(g.minimum_duration, g.maximum_distance),
    false
);

/// `DragGesture`: the pointer moves `minimum_distance` with the button or the finger down.
pub struct DragGesture {
    minimum_distance: f32,
    callbacks: Callbacks<DragValue>,
}

impl DragGesture {
    pub fn new() -> Self {
        Self {
            minimum_distance: DRAG_DISTANCE,
            callbacks: Callbacks::default(),
        }
    }

    pub fn minimum_distance(mut self, distance: f32) -> Self {
        self.minimum_distance = distance;
        self
    }
}

impl Default for DragGesture {
    fn default() -> Self {
        Self::new()
    }
}

gesture!(
    DragGesture,
    DragRecognizer,
    DragValue,
    |g: &DragGesture| DragRecognizer::new(g.minimum_distance),
    true
);

/// `MagnifyGesture`: two fingers moving apart or together.
pub struct MagnifyGesture {
    minimum_delta: f32,
    callbacks: Callbacks<MagnifyValue>,
}

impl MagnifyGesture {
    pub fn new() -> Self {
        Self {
            minimum_delta: MAGNIFY_DELTA,
            callbacks: Callbacks::default(),
        }
    }
}

impl Default for MagnifyGesture {
    fn default() -> Self {
        Self::new()
    }
}

gesture!(
    MagnifyGesture,
    MagnifyRecognizer,
    MagnifyValue,
    |g: &MagnifyGesture| MagnifyRecognizer::new(g.minimum_delta),
    true
);

/// `RotateGesture`: two fingers turning around each other.
pub struct RotateGesture {
    minimum_delta: f32,
    callbacks: Callbacks<RotateValue>,
}

impl RotateGesture {
    pub fn new() -> Self {
        Self {
            minimum_delta: ROTATE_DELTA,
            callbacks: Callbacks::default(),
        }
    }
}

impl Default for RotateGesture {
    fn default() -> Self {
        Self::new()
    }
}

gesture!(
    RotateGesture,
    RotateRecognizer,
    RotateValue,
    |g: &RotateGesture| RotateRecognizer::new(g.minimum_delta),
    true
);

// The pointers of a window: who has claimed each one. A gesture that begins claims its pointer, so the
// gestures of the views above it, which see the event later, fail; a high-priority gesture that can still
// begin keeps the others from delivering until it fails.
#[derive(Default)]
struct Pointer {
    claimed: Option<u64>,
    high: u32,
    finished: bool,
    // The gestures that saw the pointer go down, the deepest first, and whether each can still begin.
    contenders: Vec<(u64, Priority, bool)>,
    held: Vec<(u64, Box<dyn FnOnce()>)>,
}

impl Pointer {
    fn outranked(&self, id: u64) -> bool {
        self.contenders
            .iter()
            .take_while(|(other, ..)| *other != id)
            .any(|(_, priority, possible)| *possible && *priority == Priority::Normal)
    }

    fn rank(&self, id: u64) -> usize {
        self.contenders
            .iter()
            .position(|(other, ..)| *other == id)
            .unwrap_or(usize::MAX)
    }

    fn settled(&mut self, id: u64) {
        if let Some(entry) = self.contenders.iter_mut().find(|(other, ..)| *other == id) {
            entry.2 = false;
        }
    }
}

thread_local! {
    static POINTERS: RefCell<HashMap<u64, Pointer>> = RefCell::new(HashMap::new());
    static NEXT_ENGINE: Cell<u64> = const { Cell::new(1) };
    static CLOCK: Instant = Instant::now();
}

fn now_ms() -> f32 {
    CLOCK.with(|clock| clock.elapsed().as_secs_f32() * 1000.0)
}

fn with_pointer<R>(id: u64, run: impl FnOnce(&mut Pointer) -> R) -> R {
    POINTERS.with(|pointers| run(pointers.borrow_mut().entry(id).or_default()))
}

struct Engine<R: Recognizer> {
    id: u64,
    priority: Priority,
    recognizer: R,
    // Window coordinates minus these are the view's.
    origin: Point,
    armed: bool,
}

fn pointer_input(event: &Event<PointerEventData>, phase: Phase, origin: Point) -> (Input, u64) {
    let (id, global) = match event.data() {
        PointerEventData::Mouse(mouse) => (0, mouse.global_location),
        PointerEventData::Touch(touch) => (touch.finger_id + 1, touch.global_location),
    };
    let at = ((global.x as f32) - origin.0, (global.y as f32) - origin.1);
    (
        Input {
            id,
            phase,
            at,
            time: now_ms(),
        },
        id,
    )
}

impl<R: Recognizer + 'static> Engine<R>
where
    R::Value: Clone + 'static,
{
    // Feeds one input and delivers what the pointer's other gestures allow.
    fn handle(
        engine: &Rc<RefCell<Self>>,
        callbacks: &Callbacks<R::Value>,
        input: Input,
        pointer: u64,
    ) {
        if input.phase == Phase::Down {
            with_pointer(pointer, |state| {
                if state.finished {
                    *state = Pointer::default();
                }
                let priority = engine.borrow().priority;
                if priority == Priority::High {
                    state.high += 1;
                }
                state.contenders.push((engine.borrow().id, priority, true));
            });
        }
        let step = engine.borrow_mut().recognizer.feed(input);
        Self::settle(engine, callbacks, step, pointer);
        Self::arm(engine, callbacks, pointer);
        if matches!(input.phase, Phase::Up | Phase::Cancel) {
            with_pointer(pointer, |state| state.finished = true);
        }
    }

    // Waits for the deadline of a long press or a double tap.
    fn arm(engine: &Rc<RefCell<Self>>, callbacks: &Callbacks<R::Value>, pointer: u64) {
        {
            let mut engine = engine.borrow_mut();
            if engine.armed || engine.recognizer.deadline().is_none() {
                return;
            }
            engine.armed = true;
        }
        let (engine, callbacks) = (engine.clone(), callbacks.clone());
        let ticker = RenderingTicker::get();
        spawn(async move {
            loop {
                ticker.tick().await;
                let (step, more) = {
                    let mut engine = engine.borrow_mut();
                    let step = engine.recognizer.elapse(now_ms());
                    (step, engine.recognizer.deadline().is_some())
                };
                Self::settle(&engine, &callbacks, step, pointer);
                if !more {
                    engine.borrow_mut().armed = false;
                    break;
                }
            }
        });
    }

    fn settle(
        engine: &Rc<RefCell<Self>>,
        callbacks: &Callbacks<R::Value>,
        step: Step<R::Value>,
        pointer: u64,
    ) {
        let (id, priority) = {
            let engine = engine.borrow();
            (engine.id, engine.priority)
        };
        let deliver =
            |step: Step<R::Value>, callbacks: &Callbacks<R::Value>| -> Box<dyn FnOnce()> {
                let callbacks = callbacks.clone();
                Box::new(move || match step {
                    Step::Began(value) | Step::Changed(value) => {
                        for changed in &callbacks.changed {
                            changed.call(value.clone());
                        }
                    }
                    Step::Ended(value) => {
                        for changed in &callbacks.changed {
                            changed.call(value.clone());
                        }
                        if let Some(ended) = &callbacks.ended {
                            ended.call(value);
                        }
                        for reset in &callbacks.reset {
                            reset.call(());
                        }
                    }
                    Step::Cancelled | Step::Failed => {
                        for reset in &callbacks.reset {
                            reset.call(());
                        }
                    }
                    Step::Idle => {}
                })
            };
        match step {
            Step::Idle => {}
            Step::Failed | Step::Cancelled => {
                deliver(step, callbacks)();
                let mut released = with_pointer(pointer, |state| {
                    state.settled(id);
                    if priority == Priority::High {
                        state.high = state.high.saturating_sub(1);
                    }
                    if state.high == 0 && state.claimed.is_none() {
                        let mut held = std::mem::take(&mut state.held);
                        held.sort_by_key(|(holder, _)| state.rank(*holder));
                        held
                    } else {
                        Vec::new()
                    }
                });
                if !released.is_empty() {
                    let (holder, delivery) = released.remove(0);
                    let blocked = with_pointer(pointer, |state| state.outranked(holder));
                    if blocked {
                        with_pointer(pointer, |state| {
                            state.held.push((holder, delivery));
                            state.held.extend(released);
                        });
                    } else {
                        with_pointer(pointer, |state| state.claimed = Some(holder));
                        delivery();
                    }
                }
            }
            recognized => {
                if priority == Priority::Simultaneous {
                    deliver(recognized, callbacks)();
                    return;
                }
                enum Fate {
                    Deliver,
                    Hold,
                    Fail,
                }
                let fate = with_pointer(pointer, |state| match state.claimed {
                    Some(owner) if owner != id => {
                        state.settled(id);
                        Fate::Fail
                    }
                    _ if (priority != Priority::High && state.high > 0)
                        || (priority != Priority::High && state.outranked(id)) =>
                    {
                        Fate::Hold
                    }
                    _ => {
                        state.settled(id);
                        state.claimed = Some(id);
                        if priority == Priority::High {
                            state.high = state.high.saturating_sub(1);
                        }
                        state.held.clear();
                        Fate::Deliver
                    }
                });
                match fate {
                    Fate::Deliver => deliver(recognized, callbacks)(),
                    Fate::Hold => with_pointer(pointer, |state| {
                        state.held.push((id, deliver(recognized, callbacks)))
                    }),
                    Fate::Fail => {
                        engine.borrow_mut().recognizer.fail();
                        deliver(Step::Failed, callbacks)();
                    }
                }
            }
        }
    }
}

// The handlers of one gesture on one view, made during a render.
#[doc(hidden)]
pub trait Handlers {
    fn down(&self, event: &Event<PointerEventData>);
    fn moved(&self, event: &Event<PointerEventData>);
    fn up(&self, event: &Event<PointerEventData>);
}

struct Wired<G: Gesture> {
    engine: Rc<RefCell<Engine<G::Recognizer>>>,
    callbacks: Callbacks<G::Value>,
}

impl<G: Gesture> Wired<G>
where
    G::Value: Clone,
{
    fn feed(&self, event: &Event<PointerEventData>, phase: Phase) {
        let origin = self.engine.borrow().origin;
        let (input, pointer) = pointer_input(event, phase, origin);
        Engine::handle(&self.engine, &self.callbacks, input, pointer);
    }
}

impl<G: Gesture> Handlers for Wired<G>
where
    G::Value: Clone,
{
    fn down(&self, event: &Event<PointerEventData>) {
        let local = event.element_location();
        let (global, _) = pointer_input(event, Phase::Down, (0.0, 0.0));
        self.engine.borrow_mut().origin =
            (global.at.0 - local.x as f32, global.at.1 - local.y as f32);
        self.feed(event, Phase::Down);
    }

    fn moved(&self, event: &Event<PointerEventData>) {
        self.feed(event, Phase::Move);
    }

    fn up(&self, event: &Event<PointerEventData>) {
        self.feed(event, Phase::Up);
    }
}

/// What a gesture modifier takes: one gesture, or gestures put together.
pub trait Attachable {
    #[doc(hidden)]
    fn handlers(self, priority: Priority) -> Vec<Box<dyn Handlers>>;

    /// Both recognize at once.
    fn simultaneously<O: Attachable>(self, other: O) -> Combined
    where
        Self: Sized,
    {
        Combined {
            items: vec![
                (self.handlers(Priority::Simultaneous), true),
                (other.handlers(Priority::Simultaneous), true),
            ],
        }
    }

    /// `self` is tried first: while it can still begin, `other` waits, and when it does, `other` fails.
    fn exclusively<O: Attachable>(self, other: O) -> Combined
    where
        Self: Sized,
    {
        Combined {
            items: vec![
                (self.handlers(Priority::Normal), false),
                (other.handlers(Priority::Normal), false),
            ],
        }
    }
}

pub struct Combined {
    items: Vec<(Vec<Box<dyn Handlers>>, bool)>,
}

impl Attachable for Combined {
    fn handlers(self, _: Priority) -> Vec<Box<dyn Handlers>> {
        self.items
            .into_iter()
            .flat_map(|(handlers, _)| handlers)
            .collect()
    }
}

impl<G: Gesture + 'static> Attachable for G
where
    G::Value: Clone,
{
    fn handlers(self, priority: Priority) -> Vec<Box<dyn Handlers>> {
        let engine = use_hook(|| {
            let id = NEXT_ENGINE.with(|next| next.replace(next.get() + 1));
            Rc::new(RefCell::new(Engine {
                id,
                priority,
                recognizer: self.recognizer(),
                origin: (0.0, 0.0),
                armed: false,
            }))
        });
        vec![Box::new(Wired::<G> {
            engine,
            callbacks: self.callbacks(),
        })]
    }
}

/// Attaches gestures to a view: what `.gesture`, `.simultaneous_gesture` and `.high_priority_gesture` do.
pub(crate) fn attach<E: EventHandlersExt>(
    element: E,
    gesture: impl Attachable,
    priority: Priority,
) -> E {
    let handlers: Rc<Vec<Box<dyn Handlers>>> = Rc::new(gesture.handlers(priority));
    let (down, moved, up) = (handlers.clone(), handlers.clone(), handlers);
    element
        .on_pointer_down(move |event: Event<PointerEventData>| {
            down.iter().for_each(|handler| handler.down(&event));
        })
        .on_global_pointer_move(move |event: Event<PointerEventData>| {
            moved.iter().for_each(|handler| handler.moved(&event));
        })
        .on_global_pointer_up(move |event: Event<PointerEventData>| {
            up.iter().for_each(|handler| handler.up(&event));
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(id: u64, phase: Phase, x: f32, y: f32, time: f32) -> Input {
        Input {
            id,
            phase,
            at: (x, y),
            time,
        }
    }

    fn contender(depth: usize, priority: Priority, state: State, pending: bool) -> Contender {
        Contender {
            depth,
            priority,
            state,
            pending,
        }
    }

    #[test]
    fn a_tap_ends_when_the_pointer_goes_up_near_where_it_went_down() {
        let mut tap = TapRecognizer::new(1);
        assert_eq!(tap.feed(input(1, Phase::Down, 5.0, 5.0, 0.0)), Step::Idle);
        assert_eq!(
            tap.feed(input(1, Phase::Up, 8.0, 5.0, 90.0)),
            Step::Ended(TapValue {
                location: (8.0, 5.0)
            })
        );
        assert_eq!(tap.state(), State::Ended);
    }

    #[test]
    fn a_press_that_moves_past_the_slop_is_no_tap() {
        let mut tap = TapRecognizer::new(1);
        tap.feed(input(1, Phase::Down, 0.0, 0.0, 0.0));
        assert_eq!(
            tap.feed(input(1, Phase::Move, 11.0, 0.0, 30.0)),
            Step::Failed
        );
        assert_eq!(tap.feed(input(1, Phase::Up, 11.0, 0.0, 60.0)), Step::Idle);
    }

    #[test]
    fn a_double_tap_needs_two_taps_inside_the_interval() {
        let mut tap = TapRecognizer::new(2);
        tap.feed(input(1, Phase::Down, 0.0, 0.0, 0.0));
        assert_eq!(tap.feed(input(1, Phase::Up, 0.0, 0.0, 50.0)), Step::Idle);
        assert_eq!(tap.deadline(), Some(550.0));
        tap.feed(input(1, Phase::Down, 2.0, 0.0, 300.0));
        assert!(matches!(
            tap.feed(input(1, Phase::Up, 2.0, 0.0, 340.0)),
            Step::Ended(_)
        ));
    }

    #[test]
    fn a_double_tap_fails_when_the_interval_passes_and_when_the_second_tap_is_far() {
        let mut tap = TapRecognizer::new(2);
        tap.feed(input(1, Phase::Down, 0.0, 0.0, 0.0));
        tap.feed(input(1, Phase::Up, 0.0, 0.0, 50.0));
        assert_eq!(tap.elapse(500.0), Step::Idle);
        assert_eq!(tap.elapse(551.0), Step::Failed);
        tap.feed(input(1, Phase::Down, 0.0, 0.0, 600.0));
        tap.feed(input(1, Phase::Up, 0.0, 0.0, 640.0));
        tap.feed(input(1, Phase::Down, 40.0, 0.0, 700.0));
        assert_eq!(tap.feed(input(1, Phase::Up, 40.0, 0.0, 740.0)), Step::Idle);
    }

    #[test]
    fn a_long_press_ends_when_the_duration_has_passed_in_place() {
        let mut press = LongPressRecognizer::new(LONG_PRESS_MS, LONG_PRESS_DISTANCE);
        press.feed(input(1, Phase::Down, 3.0, 4.0, 100.0));
        assert_eq!(press.deadline(), Some(600.0));
        assert_eq!(press.elapse(599.0), Step::Idle);
        assert_eq!(
            press.elapse(600.0),
            Step::Ended(LongPressValue {
                location: (3.0, 4.0)
            })
        );
        assert_eq!(press.deadline(), None);
    }

    #[test]
    fn a_long_press_fails_when_released_early_or_moved_away() {
        let mut press = LongPressRecognizer::new(LONG_PRESS_MS, LONG_PRESS_DISTANCE);
        press.feed(input(1, Phase::Down, 0.0, 0.0, 0.0));
        assert_eq!(
            press.feed(input(1, Phase::Up, 0.0, 0.0, 200.0)),
            Step::Failed
        );
        press.feed(input(1, Phase::Down, 0.0, 0.0, 300.0));
        assert_eq!(
            press.feed(input(1, Phase::Move, 0.0, 12.0, 350.0)),
            Step::Failed
        );
        assert_eq!(press.elapse(2000.0), Step::Idle);
    }

    #[test]
    fn a_drag_begins_at_the_minimum_distance_with_the_translation_so_far() {
        let mut drag = DragRecognizer::new(DRAG_DISTANCE);
        drag.feed(input(1, Phase::Down, 10.0, 10.0, 0.0));
        assert_eq!(
            drag.feed(input(1, Phase::Move, 15.0, 10.0, 10.0)),
            Step::Idle
        );
        let Step::Began(value) = drag.feed(input(1, Phase::Move, 20.0, 10.0, 20.0)) else {
            panic!("the drag began");
        };
        assert_eq!(value.translation, (10.0, 0.0));
        assert_eq!(value.start_location, (10.0, 10.0));
        assert_eq!(drag.state(), State::Began);
    }

    #[test]
    fn a_drag_reports_its_velocity_in_points_per_millisecond() {
        let mut drag = DragRecognizer::new(DRAG_DISTANCE);
        drag.feed(input(1, Phase::Down, 0.0, 0.0, 0.0));
        let mut last = Step::Idle;
        for step in 1..=10 {
            last = drag.feed(input(
                1,
                Phase::Move,
                0.0,
                20.0 * step as f32,
                10.0 * step as f32,
            ));
        }
        let Step::Changed(value) = last else {
            panic!("the drag changed");
        };
        assert!((value.velocity.1 - 2.0).abs() < 0.01 && value.velocity.0 == 0.0);
        let Step::Ended(end) = drag.feed(input(1, Phase::Up, 0.0, 200.0, 100.0)) else {
            panic!("the drag ended");
        };
        assert_eq!(end.translation, (0.0, 200.0));
    }

    #[test]
    fn a_release_before_the_distance_fails_and_a_cancel_after_it_cancels() {
        let mut drag = DragRecognizer::new(DRAG_DISTANCE);
        drag.feed(input(1, Phase::Down, 0.0, 0.0, 0.0));
        assert_eq!(drag.feed(input(1, Phase::Up, 3.0, 0.0, 50.0)), Step::Failed);
        drag.feed(input(1, Phase::Down, 0.0, 0.0, 100.0));
        drag.feed(input(1, Phase::Move, 30.0, 0.0, 120.0));
        assert_eq!(
            drag.feed(input(1, Phase::Cancel, 30.0, 0.0, 130.0)),
            Step::Cancelled
        );
    }

    #[test]
    fn a_drag_ignores_a_second_finger() {
        let mut drag = DragRecognizer::new(DRAG_DISTANCE);
        drag.feed(input(1, Phase::Down, 0.0, 0.0, 0.0));
        drag.feed(input(2, Phase::Down, 100.0, 100.0, 5.0));
        assert_eq!(
            drag.feed(input(2, Phase::Move, 200.0, 200.0, 10.0)),
            Step::Idle
        );
    }

    #[test]
    fn two_fingers_moving_apart_magnify_by_the_ratio_of_their_distances() {
        let mut magnify = MagnifyRecognizer::new(MAGNIFY_DELTA);
        magnify.feed(input(1, Phase::Down, 0.0, 0.0, 0.0));
        magnify.feed(input(2, Phase::Down, 100.0, 0.0, 0.0));
        assert_eq!(
            magnify.feed(input(2, Phase::Move, 100.5, 0.0, 10.0)),
            Step::Idle
        );
        let Step::Began(value) = magnify.feed(input(2, Phase::Move, 150.0, 0.0, 20.0)) else {
            panic!("the magnify began");
        };
        assert!((value.magnification - 1.5).abs() < 1e-4);
        let Step::Ended(end) = magnify.feed(input(1, Phase::Up, 0.0, 0.0, 30.0)) else {
            panic!("the magnify ended");
        };
        assert!((end.magnification - 1.5).abs() < 1e-4);
    }

    #[test]
    fn one_finger_never_magnifies_or_rotates() {
        let mut magnify = MagnifyRecognizer::new(MAGNIFY_DELTA);
        let mut rotate = RotateRecognizer::new(ROTATE_DELTA);
        for recognizer_input in [
            input(1, Phase::Down, 0.0, 0.0, 0.0),
            input(1, Phase::Move, 90.0, 50.0, 10.0),
        ] {
            assert_eq!(magnify.feed(recognizer_input), Step::Idle);
            assert_eq!(rotate.feed(recognizer_input), Step::Idle);
        }
    }

    #[test]
    fn two_fingers_turning_rotate_by_the_angle_of_the_line_between_them() {
        let mut rotate = RotateRecognizer::new(ROTATE_DELTA);
        rotate.feed(input(1, Phase::Down, 0.0, 0.0, 0.0));
        rotate.feed(input(2, Phase::Down, 100.0, 0.0, 0.0));
        let Step::Began(value) = rotate.feed(input(2, Phase::Move, 0.0, 100.0, 20.0)) else {
            panic!("the rotation began");
        };
        assert!((value.rotation - std::f32::consts::FRAC_PI_2).abs() < 1e-4);
    }

    #[test]
    fn a_rotation_takes_the_short_way_across_the_half_turn() {
        assert!((turned(3.0, -3.0) - (std::f32::consts::TAU - 6.0)).abs() < 1e-4);
        assert!((turned(-3.0, 3.0) + (std::f32::consts::TAU - 6.0)).abs() < 1e-4);
    }

    #[test]
    fn nothing_is_decided_until_a_gesture_is_recognized() {
        let possible = contender(0, Priority::Normal, State::Possible, false);
        assert_eq!(verdicts(&[possible, possible]), vec![Verdict::Continue; 2]);
    }

    #[test]
    fn the_deepest_gesture_wins_over_its_ancestors() {
        let parent = contender(0, Priority::Normal, State::Began, false);
        let child = contender(1, Priority::Normal, State::Began, false);
        assert_eq!(
            verdicts(&[parent, child]),
            vec![Verdict::Fail, Verdict::Deliver]
        );
    }

    #[test]
    fn a_high_priority_ancestor_beats_its_descendants() {
        let parent = contender(0, Priority::High, State::Began, false);
        let child = contender(1, Priority::Normal, State::Began, false);
        assert_eq!(
            verdicts(&[parent, child]),
            vec![Verdict::Deliver, Verdict::Fail]
        );
    }

    #[test]
    fn a_simultaneous_gesture_neither_wins_nor_loses() {
        let drag = contender(1, Priority::Normal, State::Began, false);
        let along = contender(0, Priority::Simultaneous, State::Began, false);
        let waiting = contender(0, Priority::Simultaneous, State::Possible, false);
        assert_eq!(
            verdicts(&[drag, along, waiting]),
            vec![Verdict::Deliver, Verdict::Deliver, Verdict::Continue]
        );
    }

    #[test]
    fn a_gesture_that_began_fails_a_tap_that_is_still_possible() {
        let tap = contender(1, Priority::Normal, State::Possible, false);
        let drag = contender(0, Priority::Normal, State::Began, false);
        assert_eq!(
            verdicts(&[tap, drag]),
            vec![Verdict::Fail, Verdict::Deliver]
        );
    }

    #[test]
    fn a_winner_waits_while_a_gesture_that_outranks_it_can_still_begin() {
        let double_tap = contender(1, Priority::Normal, State::Possible, true);
        let tap = contender(0, Priority::Normal, State::Ended, false);
        assert_eq!(
            verdicts(&[double_tap, tap]),
            vec![Verdict::Continue, Verdict::Hold]
        );
        let failed = contender(1, Priority::Normal, State::Failed, false);
        assert_eq!(
            verdicts(&[failed, tap]),
            vec![Verdict::Fail, Verdict::Deliver]
        );
    }

    #[test]
    fn the_earlier_of_two_equal_gestures_wins() {
        let first = contender(0, Priority::Normal, State::Ended, false);
        assert_eq!(
            verdicts(&[first, first]),
            vec![Verdict::Deliver, Verdict::Fail]
        );
    }
}

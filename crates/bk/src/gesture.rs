use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::time::{Duration, Instant};

use ctdesign::GESTURE;
use freya_core::prelude::*;

use crate::platform::Timers;
use crate::scroll_physics::VelocityTracker;

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

    // The gesture comes from a trackpad pinch or rotation as well as from two fingers.
    const TRACKPAD: bool = false;

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
                if let Some((_, time)) = self.last
                    && input.time - time > GESTURE.tap_interval_ms
                {
                    self.taps = 0;
                }
                self.down = Some((input.id, input.at));
                Step::Idle
            }
            Phase::Move => match self.down {
                Some((id, at))
                    if id == input.id && distance(at, input.at) > GESTURE.tap_distance =>
                {
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
            (Some((_, time)), None) if self.taps > 0 => Some(time + GESTURE.tap_interval_ms),
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
    touches_only: bool,
    down: Option<(u64, Point)>,
    began: bool,
    x: VelocityTracker,
    y: VelocityTracker,
    last: Point,
    state: Option<State>,
}

impl DragRecognizer {
    /// A mouse does not begin it: the pan of a scroll view takes fingers only.
    pub fn touches_only(mut self, touches_only: bool) -> Self {
        self.touches_only = touches_only;
        self
    }

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
            (Phase::Down, None) if self.touches_only && input.id == 0 => Step::Idle,
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
    const TRACKPAD: bool = true;

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
    const TRACKPAD: bool = true;

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

/// What a `SequenceGesture` has: the first alone, then the first and the second once it can begin.
#[derive(Clone, Debug, PartialEq)]
pub enum SequenceValue<A, B> {
    First(A),
    Second(A, Option<B>),
}

// The second begins when the first has ended: from the press that is still down, or from the next press when
// the first ended on a release (a tap).
pub struct SequenceRecognizer<A: Recognizer, B: Recognizer> {
    first: A,
    second: B,
    done: Option<A::Value>,
    awaiting_press: bool,
    last: Option<Input>,
    state: State,
}

impl<A: Recognizer, B: Recognizer> SequenceRecognizer<A, B> {
    pub fn new(first: A, second: B) -> Self {
        Self {
            first,
            second,
            done: None,
            awaiting_press: false,
            last: None,
            state: State::Possible,
        }
    }
}

impl<A: Recognizer, B: Recognizer> SequenceRecognizer<A, B>
where
    A::Value: Clone,
    B::Value: Clone,
{
    fn first_step(
        &mut self,
        step: Step<A::Value>,
        released: bool,
        now: f32,
    ) -> Step<SequenceValue<A::Value, B::Value>> {
        match step {
            Step::Idle => Step::Idle,
            Step::Failed => {
                self.state = State::Failed;
                Step::Failed
            }
            Step::Cancelled => {
                self.state = State::Cancelled;
                Step::Cancelled
            }
            Step::Began(value) | Step::Changed(value) => {
                self.state = State::Changed;
                Step::Changed(SequenceValue::First(value))
            }
            Step::Ended(value) => {
                self.done = Some(value.clone());
                self.state = State::Changed;
                self.awaiting_press = released;
                if let (false, Some(last)) = (released, self.last) {
                    self.second.feed(Input {
                        phase: Phase::Down,
                        time: now,
                        ..last
                    });
                }
                Step::Changed(SequenceValue::Second(value, None))
            }
        }
    }

    fn second_step(
        &mut self,
        step: Step<B::Value>,
        released: bool,
    ) -> Step<SequenceValue<A::Value, B::Value>> {
        let Some(first) = self.done.clone() else {
            return Step::Idle;
        };
        let step = match step {
            Step::Began(value) | Step::Changed(value) => {
                self.state = State::Changed;
                Step::Changed(SequenceValue::Second(first, Some(value)))
            }
            Step::Ended(value) => {
                self.state = State::Ended;
                Step::Ended(SequenceValue::Second(first, Some(value)))
            }
            Step::Cancelled => {
                self.state = State::Cancelled;
                Step::Cancelled
            }
            Step::Failed | Step::Idle if released => {
                self.state = State::Ended;
                Step::Ended(SequenceValue::Second(first, None))
            }
            Step::Failed => {
                self.state = State::Failed;
                Step::Failed
            }
            Step::Idle => Step::Idle,
        };
        if matches!(step, Step::Ended(_) | Step::Cancelled | Step::Failed) {
            self.reset();
        }
        step
    }

    fn reset(&mut self) {
        self.first.fail();
        self.second.fail();
        self.done = None;
        self.awaiting_press = false;
    }
}

impl<A: Recognizer, B: Recognizer> Recognizer for SequenceRecognizer<A, B>
where
    A::Value: Clone,
    B::Value: Clone,
{
    type Value = SequenceValue<A::Value, B::Value>;
    const TRACKPAD: bool = A::TRACKPAD || B::TRACKPAD;

    fn feed(&mut self, input: Input) -> Step<Self::Value> {
        self.last = Some(input);
        let released = matches!(input.phase, Phase::Up);
        if self.done.is_none() {
            let step = self.first.feed(input);
            return self.first_step(step, released, input.time);
        }
        if self.awaiting_press {
            if input.phase != Phase::Down {
                return Step::Idle;
            }
            self.awaiting_press = false;
        }
        let step = self.second.feed(input);
        self.second_step(step, released)
    }

    fn deadline(&self) -> Option<f32> {
        if self.done.is_none() {
            self.first.deadline()
        } else {
            self.second.deadline()
        }
    }

    fn elapse(&mut self, now: f32) -> Step<Self::Value> {
        if self.done.is_none() {
            let step = self.first.elapse(now);
            self.first_step(step, false, now)
        } else {
            let step = self.second.elapse(now);
            self.second_step(step, false)
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

/// How a gesture takes part in the arena, from the modifier that attached it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Priority {
    Normal,
    High,
    Simultaneous,
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

    /// `sequenced(before:)`: `next` can begin once `self` has ended.
    fn sequenced_before<B: Gesture>(self, next: B) -> SequenceGesture<Self, B>
    where
        Self: Sized,
    {
        SequenceGesture {
            first: self,
            second: next,
            callbacks: Callbacks::default(),
        }
    }
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
            minimum_duration: GESTURE.long_press_ms,
            maximum_distance: GESTURE.long_press_distance,
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
    touches_only: bool,
    callbacks: Callbacks<DragValue>,
}

impl DragGesture {
    pub fn new() -> Self {
        Self {
            minimum_distance: GESTURE.drag_distance,
            touches_only: false,
            callbacks: Callbacks::default(),
        }
    }

    /// Only a finger drags; a mouse does not (a scroll view's pan).
    pub fn touches_only(mut self) -> Self {
        self.touches_only = true;
        self
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
    |g: &DragGesture| DragRecognizer::new(g.minimum_distance).touches_only(g.touches_only),
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
            minimum_delta: GESTURE.magnify_delta,
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
            minimum_delta: GESTURE.rotate_degrees.to_radians(),
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

/// `SequenceGesture`: the first gesture, then the second from where the first ended.
pub struct SequenceGesture<A: Gesture, B: Gesture> {
    first: A,
    second: B,
    callbacks: Callbacks<SequenceValue<A::Value, B::Value>>,
}

impl<A: Gesture, B: Gesture> SequenceGesture<A, B> {
    pub fn on_changed(
        mut self,
        callback: impl Into<Callback<SequenceValue<A::Value, B::Value>, ()>>,
    ) -> Self {
        self.callbacks.changed.push(callback.into());
        self
    }

    pub fn on_ended(
        mut self,
        callback: impl Into<Callback<SequenceValue<A::Value, B::Value>, ()>>,
    ) -> Self {
        self.callbacks.ended = Some(callback.into());
        self
    }
}

impl<A: Gesture, B: Gesture> Gesture for SequenceGesture<A, B>
where
    A::Value: Clone,
    B::Value: Clone,
{
    type Recognizer = SequenceRecognizer<A::Recognizer, B::Recognizer>;
    type Value = SequenceValue<A::Value, B::Value>;

    fn recognizer(&self) -> Self::Recognizer {
        SequenceRecognizer::new(self.first.recognizer(), self.second.recognizer())
    }

    fn callbacks(&self) -> Callbacks<Self::Value> {
        self.callbacks.clone()
    }
}

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
    callbacks: Callbacks<R::Value>,
    hovered: bool,
    // The pointer in the view's coordinates and the scale and angle of the trackpad gesture so far.
    cursor: Point,
    pinch: Option<(f32, f32)>,
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
    fn handle(engine: &Rc<RefCell<Self>>, input: Input, pointer: u64) {
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
        Self::settle(engine, step, pointer);
        Self::arm(engine, pointer);
        if matches!(input.phase, Phase::Up | Phase::Cancel) {
            with_pointer(pointer, |state| state.finished = true);
        }
    }

    // Waits for the deadline of a long press or a double tap.
    fn arm(engine: &Rc<RefCell<Self>>, pointer: u64) {
        {
            let mut engine = engine.borrow_mut();
            if engine.armed || engine.recognizer.deadline().is_none() {
                return;
            }
            engine.armed = true;
        }
        let engine = engine.clone();
        let ticker = RenderingTicker::get();
        let waker = try_consume_root_context::<Timers>()
            .map(|timers| (timers, Platform::get().current_window()));
        spawn(async move {
            let mut woken_for = None;
            loop {
                let deadline = engine.borrow().recognizer.deadline();
                // The ticker fires only after a drawn frame, so a still press asks for one at its deadline.
                if let (Some((timers, window)), Some(deadline)) = (&waker, deadline)
                    && (woken_for != Some(deadline) || now_ms() >= deadline)
                {
                    woken_for = Some(deadline);
                    let window = window.clone();
                    let delay = Duration::from_secs_f32((deadline - now_ms()).max(0.0) / 1000.0);
                    timers.after(delay, move || window.send(UserEvent::RequestRedraw));
                }
                ticker.tick().await;
                let (step, more) = {
                    let mut engine = engine.borrow_mut();
                    let step = engine.recognizer.elapse(now_ms());
                    (step, engine.recognizer.deadline().is_some())
                };
                Self::settle(&engine, step, pointer);
                if !more {
                    engine.borrow_mut().armed = false;
                    break;
                }
            }
        });
    }

    fn settle(engine: &Rc<RefCell<Self>>, step: Step<R::Value>, pointer: u64) {
        let (id, priority) = {
            let engine = engine.borrow();
            (engine.id, engine.priority)
        };
        let deliver = |step: Step<R::Value>| -> Box<dyn FnOnce()> {
            let engine = engine.clone();
            Box::new(move || {
                let callbacks = engine.borrow().callbacks.clone();
                match step {
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
                }
            })
        };
        match step {
            Step::Idle => {}
            Step::Failed | Step::Cancelled => {
                deliver(step)();
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
                    deliver(recognized)();
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
                    Fate::Deliver => deliver(recognized)(),
                    Fate::Hold => {
                        with_pointer(pointer, |state| state.held.push((id, deliver(recognized))))
                    }
                    Fate::Fail => {
                        engine.borrow_mut().recognizer.fail();
                        deliver(Step::Failed)();
                    }
                }
            }
        }
    }
}

/// A pinch or a rotation on a trackpad, as the runner reads it: `Magnify` carries the change in magnification
/// (`NSEvent.magnification`) and `Rotate` the change in degrees, counter-clockwise (`NSEvent.rotation`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrackpadKind {
    Magnify(f32),
    Rotate(f32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackpadPhase {
    Began,
    Changed,
    Ended,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trackpad {
    pub phase: TrackpadPhase,
    pub kind: TrackpadKind,
}

type Target = (Weak<dyn Any>, Box<dyn Fn(Trackpad)>);

thread_local! {
    static TRACKPAD_TARGETS: RefCell<Vec<Target>> = RefCell::new(Vec::new());
}

// Two fingers that a trackpad gesture stands for: far from the pointer's own id, `TRACKPAD_SPAN` apart.
const TRACKPAD_FINGER: u64 = 1 << 48;
const TRACKPAD_SPAN: f32 = 100.0;

/// Gives a trackpad pinch or rotation to the `MagnifyGesture` and `RotateGesture` of the views under the
/// pointer, as the two fingers it stands for. The macOS runner calls it; the Wayland one has no source yet.
pub fn trackpad(event: Trackpad) {
    TRACKPAD_TARGETS.with(|targets| {
        let mut list = std::mem::take(&mut *targets.borrow_mut());
        list.retain(|(engine, _)| engine.strong_count() > 0);
        for (_, send) in list.iter().rev() {
            send(event);
        }
        list.append(&mut targets.borrow_mut());
        *targets.borrow_mut() = list;
    });
}

fn add_trackpad_target(target: Target) {
    TRACKPAD_TARGETS.with(|targets| {
        let mut targets = targets.borrow_mut();
        targets.retain(|(engine, _)| engine.strong_count() > 0);
        targets.push(target);
    });
}

impl<R: Recognizer + 'static> Engine<R>
where
    R::Value: Clone + 'static,
{
    fn trackpad(engine: &Rc<RefCell<Self>>, event: Trackpad) {
        if !engine.borrow().hovered {
            return;
        }
        let center = engine.borrow().cursor;
        let fingers = |scale: f32, turned: f32| {
            let half = TRACKPAD_SPAN * scale / 2.0;
            let (sin, cos) = turned.sin_cos();
            [
                (center.0 - half * cos, center.1 - half * sin),
                (center.0 + half * cos, center.1 + half * sin),
            ]
        };
        let send = |phase: Phase, at: [Point; 2]| {
            for (index, at) in at.into_iter().enumerate() {
                let id = TRACKPAD_FINGER + index as u64;
                let input = Input {
                    id,
                    phase,
                    at,
                    time: now_ms(),
                };
                Self::handle(engine, input, id);
            }
        };
        let began = engine.borrow().pinch.is_none();
        if began && event.phase != TrackpadPhase::Ended {
            engine.borrow_mut().pinch = Some((1.0, 0.0));
            send(Phase::Down, fingers(1.0, 0.0));
        }
        let Some((scale, turned)) = engine.borrow().pinch else {
            return;
        };
        let (scale, turned) = match event.kind {
            TrackpadKind::Magnify(delta) => ((scale + delta).max(0.05), turned),
            TrackpadKind::Rotate(degrees) => (scale, turned - degrees.to_radians()),
        };
        engine.borrow_mut().pinch = Some((scale, turned));
        if event.phase == TrackpadPhase::Ended {
            send(Phase::Up, fingers(scale, turned));
            engine.borrow_mut().pinch = None;
        } else {
            send(Phase::Move, fingers(scale, turned));
        }
    }
}

// The handlers of one gesture on one view, made during a render.
#[doc(hidden)]
pub trait Handlers {
    fn down(&self, event: &Event<PointerEventData>);
    fn moved(&self, event: &Event<PointerEventData>);
    fn up(&self, event: &Event<PointerEventData>);
    fn hover(&self, event: &Event<PointerEventData>, inside: bool);
}

struct Wired<G: Gesture> {
    engine: Rc<RefCell<Engine<G::Recognizer>>>,
}

impl<G: Gesture> Wired<G>
where
    G::Value: Clone,
{
    fn feed(&self, event: &Event<PointerEventData>, phase: Phase) {
        let origin = self.engine.borrow().origin;
        let (input, pointer) = pointer_input(event, phase, origin);
        Engine::handle(&self.engine, input, pointer);
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
        {
            let mut engine = self.engine.borrow_mut();
            let (global, _) = pointer_input(event, Phase::Move, engine.origin);
            engine.cursor = global.at;
        }
        self.feed(event, Phase::Move);
    }

    fn hover(&self, event: &Event<PointerEventData>, inside: bool) {
        let mut engine = self.engine.borrow_mut();
        engine.hovered = inside;
        if inside {
            let local = event.element_location();
            let (global, _) = pointer_input(event, Phase::Move, (0.0, 0.0));
            engine.origin = (global.at.0 - local.x as f32, global.at.1 - local.y as f32);
            engine.cursor = (local.x as f32, local.y as f32);
        }
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
        Combined(
            [
                self.handlers(Priority::Simultaneous),
                other.handlers(Priority::Simultaneous),
            ]
            .into_iter()
            .flatten()
            .collect(),
        )
    }

    /// `self` is tried first: while it can still begin, `other` waits, and when it does, `other` fails.
    fn exclusively<O: Attachable>(self, other: O) -> Combined
    where
        Self: Sized,
    {
        Combined(
            [
                self.handlers(Priority::Normal),
                other.handlers(Priority::Normal),
            ]
            .into_iter()
            .flatten()
            .collect(),
        )
    }
}

pub struct Combined(Vec<Box<dyn Handlers>>);

impl Attachable for Combined {
    fn handlers(self, _: Priority) -> Vec<Box<dyn Handlers>> {
        self.0
    }
}

impl<G: Gesture + 'static> Attachable for G
where
    G::Value: Clone,
{
    fn handlers(self, priority: Priority) -> Vec<Box<dyn Handlers>> {
        let engine = use_hook(|| {
            let id = NEXT_ENGINE.with(|next| next.replace(next.get() + 1));
            let engine = Rc::new(RefCell::new(Engine {
                id,
                priority,
                recognizer: self.recognizer(),
                origin: (0.0, 0.0),
                armed: false,
                callbacks: self.callbacks(),
                hovered: false,
                cursor: (0.0, 0.0),
                pinch: None,
            }));
            if <G::Recognizer as Recognizer>::TRACKPAD {
                let weak = Rc::downgrade(&engine);
                add_trackpad_target((
                    weak.clone(),
                    Box::new(move |event| {
                        if let Some(engine) = weak.upgrade() {
                            Engine::trackpad(&engine, event);
                        }
                    }),
                ));
            }
            engine
        });
        {
            let mut wired = engine.borrow_mut();
            wired.callbacks = self.callbacks();
            wired.priority = priority;
        }
        vec![Box::new(Wired::<G> { engine })]
    }
}

/// Attaches gestures to a view: what `.gesture`, `.simultaneous_gesture` and `.high_priority_gesture` do.
pub(crate) fn attach<E: EventHandlersExt>(
    element: E,
    gesture: impl Attachable,
    priority: Priority,
) -> E {
    let handlers: Rc<Vec<Box<dyn Handlers>>> = Rc::new(gesture.handlers(priority));
    let (down, moved, up) = (handlers.clone(), handlers.clone(), handlers.clone());
    let (enter, leave) = (handlers.clone(), handlers);
    element
        .on_pointer_enter(move |event: Event<PointerEventData>| {
            enter.iter().for_each(|handler| handler.hover(&event, true));
        })
        .on_pointer_leave(move |event: Event<PointerEventData>| {
            leave
                .iter()
                .for_each(|handler| handler.hover(&event, false));
        })
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

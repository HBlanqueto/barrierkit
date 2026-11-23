use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use freya_animation::prelude::{AnimDirection, AnimNum, AnimatedValue, use_animation_transition};
use freya_core::prelude::*;

use crate::styles::Animation;

pub fn use_alive() -> Rc<Cell<bool>> {
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    use_drop({
        let alive = alive.clone();
        move || alive.set(false)
    });
    alive
}

// Animations build their closure once, so the preference is a reactive state read when they run.
pub fn use_reduced_motion() -> State<bool> {
    use_reactive(&crate::theme::resolved().reduced_motion)
}

pub fn motion_ms(reduced_motion: State<bool>, ms: u64) -> u64 {
    if *reduced_motion.peek() { 0 } else { ms }
}

// A transaction applies to the changes that follow it for this long: a render pass, not a measurement.
const TRANSACTION_WINDOW: Duration = Duration::from_millis(100);

thread_local! {
    static NESTING: Cell<u32> = const { Cell::new(0) };
    static TRANSACTION: Cell<Option<(Instant, Animation)>> = const { Cell::new(None) };
}

// Freya's State has one write path the kit cannot tag, so the outermost call of a turn decides (ADR 0102).
pub fn with_animation<R>(animation: Animation, body: impl FnOnce() -> R) -> R {
    let depth = NESTING.get();
    NESTING.set(depth + 1);
    let result = body();
    NESTING.set(depth);
    if depth == 0 {
        TRANSACTION.set(Some((Instant::now(), animation)));
    }
    result
}

pub(crate) fn transaction_at(now: Instant) -> Option<Animation> {
    TRANSACTION
        .get()
        .filter(|(at, _)| now.saturating_duration_since(*at) <= TRANSACTION_WINDOW)
        .map(|(_, animation)| animation)
}

pub(crate) fn effective(animation: Animation, reduced_motion: bool) -> Animation {
    if reduced_motion {
        Animation::None
    } else {
        animation
    }
}

// The path of one change: from where it started, with the velocity it had, to where it goes.
#[derive(Clone, Copy)]
struct Segment {
    to: f32,
    animation: Animation,
    path: Path,
}

#[derive(Clone, Copy)]
enum Path {
    Spring(Spring),
    Timed { from: f32, duration_ms: u64 },
}

impl Segment {
    fn new(from: f32, to: f32, velocity: f32, animation: Animation) -> Self {
        let path = match animation {
            Animation::Spring {
                duration_ms,
                bounce,
            } => Path::Spring(Spring::moving(
                from,
                to,
                Duration::from_millis(duration_ms),
                bounce,
                velocity,
            )),
            _ => Path::Timed {
                from,
                duration_ms: animation.duration_ms(),
            },
        };
        Self {
            to,
            animation,
            path,
        }
    }

    fn duration_ms(self) -> u64 {
        match self.path {
            Path::Spring(spring) => spring.settling_duration().as_millis() as u64,
            Path::Timed { duration_ms, .. } => duration_ms,
        }
    }

    fn value(self, fraction: f32) -> f32 {
        let fraction = fraction.clamp(0.0, 1.0);
        if fraction == 1.0 {
            return self.to;
        }
        match self.path {
            Path::Spring(spring) => spring.value(Duration::from_secs_f32(
                fraction * self.duration_ms() as f32 / 1000.0,
            )),
            Path::Timed { from, .. } => from + (self.to - from) * self.animation.progress(fraction),
        }
    }

    fn velocity(self, fraction: f32) -> f32 {
        let seconds = self.duration_ms() as f32 / 1000.0;
        match self.path {
            Path::Spring(spring) => {
                spring.velocity(Duration::from_secs_f32(fraction.clamp(0.0, 1.0) * seconds))
            }
            Path::Timed { .. } => {
                let step = 1.0 / 600.0;
                let (before, after) = ((fraction - step).max(0.0), (fraction + step).min(1.0));
                (self.value(after) - self.value(before))
                    / ((after - before) * seconds).max(f32::EPSILON)
            }
        }
    }

    fn retarget(self, fraction: f32, to: f32, animation: Animation) -> Self {
        Self::new(self.value(fraction), to, self.velocity(fraction), animation)
    }
}

pub fn use_animated(target: f32, animation: Animation) -> f32 {
    let segment = use_hook(|| Rc::new(Cell::new(Segment::new(target, target, 0.0, animation))));
    let mut serial = use_state(|| 0u32);
    let elapsed = use_animation_transition(serial, {
        let segment = segment.clone();
        move |_: u32, _| AnimNum::new(0.0, 1.0).time(segment.get().duration_ms())
    });
    // Freya restarts the clock one render after the serial changes; that render still reads the old one, finished.
    let held = use_hook(|| Rc::new(Cell::new(None::<f32>)));
    let fraction = elapsed.get().value();
    if let Some(held) = held.take()
        && fraction >= 1.0
    {
        return held;
    }
    let current = segment.get();
    let shown = current.value(fraction);
    if current.to != target {
        let animation = effective(
            transaction_at(Instant::now()).unwrap_or(animation),
            crate::theme::resolved().reduced_motion,
        );
        segment.set(current.retarget(fraction, target, animation));
        held.set((animation.duration_ms() > 0).then_some(shown));
        *serial.write() += 1;
    }
    if segment.get().animation.duration_ms() == 0 {
        target
    } else {
        shown
    }
}

pub fn use_switch_progress(on: State<bool>, animation: Animation) -> f32 {
    let animation = if *use_reduced_motion().read() {
        Animation::None
    } else {
        animation
    };
    let duration_ms = use_reactive(&animation.duration_ms());
    // A switch turned back midway goes on from the progress shown, not from the other end.
    let resume = use_hook(|| {
        let at = f32::from(u8::from(*on.peek()));
        Rc::new(Cell::new((at, at)))
    });
    let starts = resume.clone();
    let linear = use_animation_transition(on, move |_: bool, to: bool| {
        let (toward_on, toward_off) = starts.get();
        let (from, to) = if to {
            (toward_on, 1.0)
        } else {
            (toward_off, 0.0)
        };
        AnimNum::new(from, to).time((*duration_ms.peek() as f32 * (to - from).abs()) as u64)
    });
    let linear = linear.get().value();
    let progress = if *on.read() {
        animation.progress(linear)
    } else {
        1.0 - animation.progress(1.0 - linear)
    };
    resume.set((
        elapsed_at(animation, progress),
        1.0 - elapsed_at(animation, 1.0 - progress),
    ));
    progress
}

fn elapsed_at(animation: Animation, progress: f32) -> f32 {
    let (mut low, mut high) = (0.0f32, 1.0f32);
    for _ in 0..24 {
        let middle = (low + high) / 2.0;
        if animation.progress(middle) < progress {
            low = middle;
        } else {
            high = middle;
        }
    }
    (low + high) / 2.0
}

pub(crate) use ctdesign::{ease_in_out, ease_out};

// SwiftUI's `Spring(duration:bounce:)`: `duration` is the perceptual period 2π/ω0 and `bounce` is 1 - ζ.
// The value is within SETTLED of the travel from the settling duration on and exactly `to` after it.
const SETTLED: f32 = 0.001;
const SETTLING_LIMIT: f32 = 10.0;
const SETTLING_STEP: f32 = 1.0 / 240.0;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Spring {
    from: f32,
    to: f32,
    duration: Duration,
    bounce: f32,
    velocity: f32,
    settling: Duration,
    current: f32,
}

impl Spring {
    pub fn new(from: f32, to: f32, duration: Duration, bounce: f32) -> Self {
        Self::moving(from, to, duration, bounce, 0.0)
    }

    fn moving(from: f32, to: f32, duration: Duration, bounce: f32, velocity: f32) -> Self {
        Self {
            from,
            to,
            duration,
            bounce: bounce.clamp(0.0, 0.95),
            velocity,
            settling: Duration::ZERO,
            current: from,
        }
        .settled()
    }

    // Units per second.
    pub fn with_velocity(self, velocity: f32) -> Self {
        Self { velocity, ..self }.settled()
    }

    pub fn settling_duration(&self) -> Duration {
        self.settling
    }

    fn omega(&self) -> f32 {
        2.0 * std::f32::consts::PI / self.duration.as_secs_f32().max(f32::EPSILON)
    }

    // The displacement from `to` and its rate after `t` seconds.
    fn offset(&self, t: f32) -> (f32, f32) {
        let (y0, v0, omega, zeta) = (
            self.from - self.to,
            self.velocity,
            self.omega(),
            1.0 - self.bounce,
        );
        if self.bounce == 0.0 {
            let c = v0 + omega * y0;
            let e = (-omega * t).exp();
            (e * (y0 + c * t), e * (v0 - omega * c * t))
        } else {
            let decay = zeta * omega;
            let damped = omega * (1.0 - zeta * zeta).sqrt();
            let e = (-decay * t).exp();
            let (sin, cos) = (damped * t).sin_cos();
            (
                e * (y0 * cos + (v0 + decay * y0) / damped * sin),
                e * (v0 * cos - (decay * v0 + omega * omega * y0) / damped * sin),
            )
        }
    }

    fn settled(mut self) -> Self {
        let scale = (self.from - self.to)
            .abs()
            .max(self.velocity.abs() / self.omega());
        let (mut last, mut t) = (0.0, 0.0);
        while t < SETTLING_LIMIT && scale > 0.0 {
            let (y, v) = self.offset(t);
            if y.abs() > SETTLED * scale || v.abs() > SETTLED * scale * self.omega() {
                last = t + SETTLING_STEP;
            }
            t += SETTLING_STEP;
        }
        self.settling = Duration::from_millis((last * 1000.0).ceil() as u64);
        self
    }

    pub fn value(&self, elapsed: Duration) -> f32 {
        if elapsed >= self.settling {
            return self.to;
        }
        self.to + self.offset(elapsed.as_secs_f32()).0
    }

    pub fn velocity(&self, elapsed: Duration) -> f32 {
        if elapsed >= self.settling {
            return 0.0;
        }
        self.offset(elapsed.as_secs_f32()).1
    }

    pub fn current(&self) -> f32 {
        self.current
    }
}

impl AnimatedValue for Spring {
    fn prepare(&mut self, direction: AnimDirection) {
        self.current = match direction {
            AnimDirection::Forward => self.from,
            AnimDirection::Reverse => self.to,
        };
    }

    fn is_finished(&self, index: u128, _: AnimDirection) -> bool {
        index >= self.settling.as_millis()
    }

    fn advance(&mut self, index: u128, direction: AnimDirection) {
        let elapsed = Duration::from_millis(index.min(self.settling.as_millis()) as u64);
        self.current = match direction {
            AnimDirection::Forward => self.value(elapsed),
            AnimDirection::Reverse => self.into_reversed().value(elapsed),
        };
    }

    fn finish(&mut self, direction: AnimDirection) {
        self.advance(self.settling.as_millis(), direction);
    }

    fn into_reversed(self) -> Self {
        let reversed = Self {
            from: self.to,
            to: self.from,
            velocity: 0.0,
            ..self
        };
        // Without velocity the offset only changes sign, so the settling is the same.
        if self.velocity == 0.0 {
            reversed
        } else {
            reversed.settled()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMOOTH: Duration = Duration::from_millis(344);

    fn smooth() -> Spring {
        Spring::new(0.0, 1.0, SMOOTH, 0.0)
    }

    #[test]
    fn a_spring_without_bounce_is_critically_damped_at_two_pi_over_its_duration() {
        let omega = 2.0 * std::f32::consts::PI / 0.344;
        for t in [0.02, 0.09, 0.2, 0.35] {
            let expected = 1.0 - (-omega * t).exp() * (1.0 + omega * t);
            assert!((smooth().value(Duration::from_secs_f32(t)) - expected).abs() < 1e-4);
        }
    }

    #[test]
    fn endpoints_are_exact_and_the_settling_duration_is_finite() {
        let spring = Spring::new(0.0, 100.0, SMOOTH, 0.4);
        assert_eq!(spring.value(Duration::ZERO), 0.0);
        assert_eq!(spring.value(spring.settling_duration()), 100.0);
        assert!(spring.settling_duration() < Duration::from_secs(5));
    }

    #[test]
    fn bounce_overshoots_and_zero_bounce_does_not() {
        let overshoot = |bounce| {
            let spring = Spring::new(0.0, 1.0, SMOOTH, bounce);
            (0..200)
                .map(|i| spring.value(Duration::from_millis(i * 5)))
                .fold(0.0f32, f32::max)
        };
        assert!(overshoot(0.3) > 1.01);
        assert!(overshoot(0.0) <= 1.0);
    }

    #[test]
    fn a_spring_started_from_the_value_and_velocity_of_another_goes_on_with_it() {
        let first = Spring::new(0.0, 1.0, SMOOTH, 0.2);
        let at = Duration::from_millis(90);
        let second =
            Spring::new(first.value(at), 1.0, SMOOTH, 0.2).with_velocity(first.velocity(at));
        for ms in [0, 30, 100, 250] {
            let ahead = first.value(at + Duration::from_millis(ms));
            assert!((second.value(Duration::from_millis(ms)) - ahead).abs() < 1e-4);
        }
    }

    #[test]
    fn an_interrupted_spring_keeps_its_velocity_and_its_value() {
        let first = smooth();
        let at = Duration::from_millis(100);
        let back = Spring::new(first.value(at), 0.0, SMOOTH, 0.0).with_velocity(first.velocity(at));
        assert_eq!(back.value(Duration::ZERO), first.value(at));
        assert_eq!(back.velocity(Duration::ZERO), first.velocity(at));
        assert!(first.velocity(at) > 1.0);
    }

    #[test]
    fn advancing_past_the_settling_lands_on_the_target_in_both_directions() {
        let mut spring = Spring::new(0.0, 1.0, Duration::from_millis(100), 0.6);
        spring.advance(5000, AnimDirection::Forward);
        assert_eq!(spring.current(), 1.0);
        spring.advance(5000, AnimDirection::Reverse);
        assert_eq!(spring.current(), 0.0);
    }

    #[test]
    fn a_spring_with_nowhere_to_go_does_not_move() {
        let spring = Spring::new(3.0, 3.0, SMOOTH, 0.0);
        assert_eq!(spring.settling_duration(), Duration::ZERO);
        assert_eq!(spring.value(Duration::from_millis(50)), 3.0);
    }

    #[test]
    fn reduced_motion_gives_no_duration() {
        let spring = Animation::Spring {
            duration_ms: 344,
            bounce: 0.0,
        };
        assert_eq!(effective(spring, true).duration_ms(), 0);
        assert!(effective(spring, false).duration_ms() > 0);
    }

    #[test]
    fn a_segment_retargeted_midway_starts_at_the_value_and_velocity_it_showed() {
        let animation = Animation::Spring {
            duration_ms: 344,
            bounce: 0.0,
        };
        let first = Segment::new(0.0, 1.0, 0.0, animation);
        let midway = first.retarget(0.1, 0.0, animation);
        assert!((midway.value(0.0) - first.value(0.1)).abs() < 1e-5);
        assert!((midway.velocity(0.0) - first.velocity(0.1)).abs() < 1e-3);
        assert_eq!(midway.value(1.0), 0.0);
    }

    #[test]
    fn a_timed_segment_has_the_velocity_of_its_curve_and_a_spring_after_it_carries_it() {
        let timed = Animation::Ease { duration_ms: 250 };
        let first = Segment::new(0.0, 1.0, 0.0, timed);
        let rate = first.velocity(0.2);
        let expected = 3.0 * (1.0f32 - 0.2).powi(2) / 0.25;
        assert!((rate - expected).abs() < 0.05, "{rate} {expected}");
        let spring = Animation::Spring {
            duration_ms: 344,
            bounce: 0.0,
        };
        assert!((first.retarget(0.2, 0.0, spring).velocity(0.0) - rate).abs() < 0.05);
    }

    #[test]
    fn the_outermost_with_animation_of_a_turn_decides() {
        let slow = Animation::Ease { duration_ms: 1000 };
        let fast = Animation::Ease { duration_ms: 400 };
        with_animation(slow, || with_animation(fast, || ()));
        assert_eq!(transaction_at(Instant::now()).unwrap().duration_ms(), 1000);
        with_animation(Animation::None, || with_animation(fast, || ()));
        assert_eq!(transaction_at(Instant::now()).unwrap().duration_ms(), 0);
    }

    #[test]
    fn a_transaction_expires_after_its_window() {
        with_animation(Animation::Ease { duration_ms: 100 }, || ());
        let later = Instant::now() + TRANSACTION_WINDOW + Duration::from_millis(1);
        assert!(transaction_at(later).is_none());
    }
}

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

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

pub(crate) fn use_animated(target: f32, animation: Animation) -> f32 {
    let duration_ms = use_reactive(&animation.duration_ms());
    let mut from = use_state(|| target);
    let mut to = use_state(|| target);
    let elapsed = use_animation_transition(to, move |_: f32, _| {
        AnimNum::new(0.0, 1.0).time(*duration_ms.peek())
    });
    let (start, end) = (*from.peek(), *to.peek());
    let shown = start + (end - start) * animation.progress(elapsed.get().value());
    if end != target {
        from.set(shown);
        to.set(target);
    }
    if animation.duration_ms() == 0 {
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

pub(crate) fn ease_out(progress: f32) -> f32 {
    1.0 - (1.0 - progress).powi(3)
}

pub(crate) fn ease_in_out(progress: f32) -> f32 {
    if progress < 0.5 {
        4.0 * progress.powi(3)
    } else {
        1.0 - (2.0 - 2.0 * progress).powi(3) / 2.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Spring {
    from: f32,
    to: f32,
    duration: Duration,
    bounce: f32,
    current: f32,
}

impl Spring {
    pub fn new(from: f32, to: f32, duration: Duration, bounce: f32) -> Self {
        Self {
            from,
            to,
            duration,
            bounce: bounce.clamp(0.0, 1.0),
            current: from,
        }
    }

    pub fn value_at(&self, progress: f32) -> f32 {
        let t = progress.clamp(0.0, 1.0);
        if t >= 1.0 {
            return self.to;
        }
        let base = if self.bounce == 0.0 {
            1.0 - (1.0 - t).powi(3)
        } else {
            let omega = std::f32::consts::PI * (1.0 + self.bounce);
            1.0 - (-ctdesign::SPRING_DAMPING * t).exp() * (omega * t).cos()
        };
        self.from + (self.to - self.from) * base
    }

    pub fn value(&self, elapsed: Duration) -> f32 {
        if elapsed >= self.duration {
            return self.to;
        }
        let secs = self.duration.as_secs_f32().max(f32::EPSILON);
        self.value_at(elapsed.as_secs_f32() / secs)
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
        index >= self.duration.as_millis()
    }

    fn advance(&mut self, index: u128, direction: AnimDirection) {
        let elapsed = Duration::from_millis(index.min(self.duration.as_millis()) as u64);
        self.current = match direction {
            AnimDirection::Forward => self.value(elapsed),
            AnimDirection::Reverse => self.into_reversed().value(elapsed),
        };
    }

    fn finish(&mut self, direction: AnimDirection) {
        self.advance(self.duration.as_millis(), direction);
    }

    fn into_reversed(self) -> Self {
        Self {
            from: self.to,
            to: self.from,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints_are_exact() {
        let spring = Spring::new(0.0, 100.0, Duration::from_millis(250), 0.4);
        assert_eq!(spring.value_at(0.0), 0.0);
        assert_eq!(spring.value_at(1.0), 100.0);
        assert_eq!(spring.value(Duration::from_millis(250)), 100.0);
    }

    #[test]
    fn bounce_overshoots_and_zero_bounce_does_not() {
        let stiff = Spring::new(0.0, 1.0, Duration::from_millis(200), 0.0);
        let bouncy = Spring::new(0.0, 1.0, Duration::from_millis(200), 0.8);
        assert!(stiff.value_at(0.5) <= 1.0);
        assert!(bouncy.value_at(0.5) > 1.0 || bouncy.value_at(0.35) > 1.0);
    }

    #[test]
    fn advancing_past_the_duration_lands_on_the_target_in_both_directions() {
        let mut spring = Spring::new(0.0, 1.0, Duration::from_millis(100), 0.6);
        spring.advance(500, AnimDirection::Forward);
        assert_eq!(spring.current(), 1.0);
        spring.advance(500, AnimDirection::Reverse);
        assert_eq!(spring.current(), 0.0);
    }

    #[test]
    fn progress_is_clamped() {
        let spring = Spring::new(10.0, 20.0, Duration::from_millis(100), 0.2);
        assert_eq!(spring.value_at(-5.0), 10.0);
        assert_eq!(spring.value_at(9.0), 20.0);
    }
}

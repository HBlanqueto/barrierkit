use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use freya_animation::prelude::{
    AnimDirection, AnimNum, AnimatedValue, Ease, Function, use_animation_transition,
};
use freya_core::prelude::*;

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

pub fn use_switch_progress(on: State<bool>, ms: u64) -> f32 {
    let reduced_motion = use_reduced_motion();
    let animation = use_animation_transition(on, move |from: bool, to: bool| {
        AnimNum::new(f32::from(u8::from(from)), f32::from(u8::from(to)))
            .time(motion_ms(reduced_motion, ms))
            .ease(Ease::Out)
            .function(Function::Cubic)
    });
    if *reduced_motion.read() {
        f32::from(u8::from(*on.read()))
    } else {
        animation.get().value()
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
            1.0 - (-vow_design::SPRING_DAMPING * t).exp() * (omega * t).cos()
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

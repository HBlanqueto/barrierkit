//! Spring-based animation helpers built on `freya_animation`.

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring {
    from: f32,
    to: f32,
    duration: Duration,
    bounce: f32,
}

impl Spring {
    pub fn new(from: f32, to: f32, duration: Duration, bounce: f32) -> Self {
        Self {
            from,
            to,
            duration,
            bounce: bounce.clamp(0.0, 1.0),
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
            let damping = 5.0;
            let omega = std::f32::consts::PI * (1.0 + self.bounce);
            1.0 - (-damping * t).exp() * (omega * t).cos()
        };
        self.from + (self.to - self.from) * base
    }

    pub fn value(&self, elapsed: Duration) -> f32 {
        let secs = self.duration.as_secs_f32().max(f32::EPSILON);
        self.value_at(elapsed.as_secs_f32() / secs)
    }

    pub fn duration(&self) -> Duration {
        self.duration
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
    fn progress_is_clamped() {
        let spring = Spring::new(10.0, 20.0, Duration::from_millis(100), 0.2);
        assert_eq!(spring.value_at(-5.0), 10.0);
        assert_eq!(spring.value_at(9.0), 20.0);
    }
}

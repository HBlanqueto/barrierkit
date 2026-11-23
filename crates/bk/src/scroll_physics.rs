use std::collections::VecDeque;

// Measured on a Mac Catalyst `UIScrollView` fed with touchpad scroll events (UIKit animations, section 6, ADR 0104).
// Velocities are points per millisecond, times are milliseconds, distances are points.
const FLING_DISTANCE: f32 = 157.5;
const FLING_EXPONENT: f32 = 4.0 / 3.0;
const FLING_POWER: i32 = 4;
const BAND: f32 = 0.071;
const RETURN_OMEGA: f32 = 12.0 / 1000.0;
const BAND_SPEED: f32 = 0.3;
const SETTLED_OVER: f32 = 0.25;
const SETTLED_SPEED: f32 = 0.002;
const VELOCITY_WINDOW_MS: f32 = 100.0;
pub(crate) const FLING_MIN_SPEED: f32 = 0.05;

// The release velocity of a drag: the movement in the last 100 ms over the time it took.
#[derive(Default)]
pub(crate) struct VelocityTracker {
    samples: VecDeque<(f32, f32)>,
}

impl VelocityTracker {
    pub(crate) fn push(&mut self, at: f32, delta: f32) {
        self.samples.push_back((at, delta));
        while self
            .samples
            .front()
            .is_some_and(|(first, _)| at - first > VELOCITY_WINDOW_MS)
        {
            self.samples.pop_front();
        }
    }

    pub(crate) fn velocity(&self, now: f32) -> f32 {
        let (Some(&(first, _)), Some(&(last, _))) = (self.samples.front(), self.samples.back())
        else {
            return 0.0;
        };
        if now - last > VELOCITY_WINDOW_MS || last <= first {
            return 0.0;
        }
        self.samples
            .iter()
            .skip(1)
            .map(|(_, delta)| delta)
            .sum::<f32>()
            / (last - first)
    }

    pub(crate) fn clear(&mut self) {
        self.samples.clear();
    }
}

// The distance of the stretch past an edge for an input of `travel`.
pub(crate) fn rubber_band(travel: f32, dimension: f32) -> f32 {
    let dimension = dimension.max(1.0);
    travel.signum() * dimension * (1.0 - 1.0 / (1.0 + BAND * travel.abs() / dimension))
}

// The input that stretches the content by `over`.
pub(crate) fn rubber_band_input(over: f32, dimension: f32) -> f32 {
    let dimension = dimension.max(1.0);
    let over = over.abs().min(dimension * 0.999).copysign(over);
    dimension * over / (BAND * (dimension - over.abs()))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Fling {
    speed: f32,
    distance: f32,
    duration: f32,
}

impl Fling {
    pub(crate) fn new(speed: f32) -> Self {
        let distance = FLING_DISTANCE * speed.abs().powf(FLING_EXPONENT);
        Self {
            speed,
            distance: distance.copysign(speed),
            duration: if speed == 0.0 {
                0.0
            } else {
                FLING_POWER as f32 * distance / speed.abs()
            },
        }
    }

    pub(crate) fn duration(self) -> f32 {
        self.duration
    }

    pub(crate) fn travelled(self, at: f32) -> f32 {
        let left = 1.0 - (at / self.duration.max(f32::EPSILON)).clamp(0.0, 1.0);
        self.distance * (1.0 - left.powi(FLING_POWER))
    }

    pub(crate) fn speed_at(self, at: f32) -> f32 {
        let left = 1.0 - (at / self.duration.max(f32::EPSILON)).clamp(0.0, 1.0);
        self.speed * left.powi(FLING_POWER - 1)
    }

    // The time the fling has covered `distance` of its travel, if it ever does.
    fn reaches(self, distance: f32) -> Option<f32> {
        if distance.abs() > self.distance.abs() || distance.signum() != self.distance.signum() {
            return None;
        }
        let (mut low, mut high) = (0.0, self.duration);
        for _ in 0..32 {
            let middle = (low + high) / 2.0;
            if self.travelled(middle).abs() < distance.abs() {
                low = middle;
            } else {
                high = middle;
            }
        }
        Some(high)
    }
}

// A stretch that returns on a critically damped spring.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Bounce {
    over: f32,
    speed: f32,
}

impl Bounce {
    fn over_at(self, at: f32) -> f32 {
        let omega = RETURN_OMEGA;
        (self.over + (self.speed + omega * self.over) * at) * (-omega * at).exp()
    }

    fn speed_at(self, at: f32) -> f32 {
        let omega = RETURN_OMEGA;
        let linear = self.speed + omega * self.over;
        (self.speed - omega * linear * at) * (-omega * at).exp()
    }

    fn settles_at(self) -> f32 {
        let mut at = 0.0;
        while at < 4000.0
            && (self.over_at(at).abs() > SETTLED_OVER || self.speed_at(at).abs() > SETTLED_SPEED)
        {
            at += 4.0;
        }
        at
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Position {
    // The offset inside the range and the stretch past an edge (positive past the far one).
    pub(crate) rest: f32,
    pub(crate) over: f32,
    pub(crate) done: bool,
}

// What happens after a release: a fling inside `0..=limit`, and a bounce when it reaches an edge or the release
// was past one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Glide {
    from: f32,
    limit: f32,
    fling: Fling,
    edge: Option<(f32, f32, Bounce)>,
    bounce: Option<Bounce>,
}

impl Glide {
    pub(crate) fn new(from: f32, speed: f32, limit: f32, over: f32) -> Self {
        let fling = if over == 0.0 && speed.abs() >= FLING_MIN_SPEED {
            Fling::new(speed)
        } else {
            Fling::new(0.0)
        };
        let reach = if speed > 0.0 { limit - from } else { -from };
        let edge = fling.reaches(reach).map(|at| {
            let arrival = fling.speed_at(at);
            (
                at,
                reach,
                Bounce {
                    over: 0.0,
                    speed: BAND_SPEED * arrival,
                },
            )
        });
        Self {
            from,
            limit,
            fling,
            edge,
            bounce: (over != 0.0).then_some(Bounce { over, speed: 0.0 }),
        }
    }

    pub(crate) fn at(self, at: f32) -> Position {
        if let Some(bounce) = self.bounce {
            return Position {
                rest: self.from,
                over: bounce.over_at(at),
                done: at >= bounce.settles_at(),
            };
        }
        match self.edge {
            Some((arrival, reach, bounce)) if at >= arrival => {
                let since = at - arrival;
                Position {
                    rest: self.from + reach,
                    over: bounce.over_at(since),
                    done: since >= bounce.settles_at(),
                }
            }
            _ => Position {
                rest: (self.from + self.fling.travelled(at)).clamp(0.0, self.limit),
                over: 0.0,
                done: at >= self.fling.duration(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(actual: f32, expected: f32, tolerance: f32) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "{actual} is not within {tolerance} of {expected}"
        );
    }

    #[test]
    fn a_flick_stops_where_a_catalyst_scroll_view_stops() {
        for (speed, stop) in [
            (0.695, 97.0),
            (2.0, 396.6),
            (4.638, 1218.2),
            (13.11, 4868.5),
        ] {
            near(Fling::new(speed).travelled(f32::MAX), stop, stop * 0.01);
        }
        near(Fling::new(-2.0).travelled(f32::MAX), -396.6, 4.0);
    }

    #[test]
    fn a_fling_starts_at_the_release_speed_and_stops_at_its_duration() {
        let fling = Fling::new(2.0);
        near(fling.speed_at(0.0), 2.0, 1e-4);
        near(fling.speed_at(fling.duration()), 0.0, 1e-4);
        near(fling.duration(), 794.0, 4.0);
        near(
            fling.travelled(fling.duration()),
            fling.travelled(f32::MAX),
            1e-3,
        );
    }

    #[test]
    fn a_fling_follows_the_curve_recorded_at_a_tenth_to_four_fifths_of_its_time() {
        let fling = Fling::new(2.0);
        for (fraction, recorded) in [(0.1, 0.35), (0.2, 0.60), (0.5, 0.935), (0.8, 0.998)] {
            near(
                fling.travelled(fraction * fling.duration()) / fling.travelled(f32::MAX),
                recorded,
                0.012,
            );
        }
    }

    #[test]
    fn a_pull_past_an_edge_stretches_as_measured() {
        near(rubber_band(240.0, 500.0), 16.0, 1.0);
        near(rubber_band(600.0, 500.0), 39.0, 1.0);
        near(rubber_band(1800.0, 500.0), 103.0, 2.5);
        near(rubber_band(-600.0, 500.0), -39.0, 1.0);
    }

    #[test]
    fn the_input_of_a_stretch_gives_the_stretch_back() {
        for travel in [-900.0, -30.0, 0.0, 5.0, 240.0, 1800.0] {
            near(
                rubber_band_input(rubber_band(travel, 500.0), 500.0),
                travel,
                travel.abs() * 1e-3 + 1e-3,
            );
        }
    }

    #[test]
    fn the_stretch_never_reaches_the_size_of_the_view() {
        assert!(rubber_band(1.0e6, 500.0) < 500.0);
        assert_eq!(rubber_band(0.0, 500.0), 0.0);
    }

    #[test]
    fn a_release_in_the_band_returns_without_crossing_the_edge() {
        let glide = Glide::new(0.0, 0.0, 400.0, -40.0);
        let mut last = -40.0;
        for step in 0..200 {
            let position = glide.at(step as f32 * 4.0);
            assert!(position.over <= 0.0 && position.over >= last);
            assert_eq!(position.rest, 0.0);
            last = position.over;
        }
        near(
            glide.at(300.0).over,
            -40.0 * (1.0 + 3.6) * (-3.6f32).exp(),
            0.1,
        );
        assert!(glide.at(900.0).done && glide.at(900.0).over.abs() < SETTLED_OVER);
    }

    #[test]
    fn a_fling_into_the_edge_overshoots_as_measured_and_comes_back() {
        // 2.85 pt/ms from 112 pt before the end: UIKit overshot 23 pt, 75 ms after it hit.
        let glide = Glide::new(288.0, 2.85, 400.0, 0.0);
        let peak = (0..300)
            .map(|step| glide.at(step as f32 * 2.0).over)
            .fold(0.0f32, f32::max);
        near(peak, 23.0, 4.0);
        let end = glide.at(2000.0);
        assert!(end.done);
        assert_eq!((end.rest, end.over.abs() < SETTLED_OVER), (400.0, true));
    }

    #[test]
    fn a_fling_that_stops_inside_the_range_stays_inside_it() {
        let glide = Glide::new(100.0, 1.0, 1000.0, 0.0);
        for step in 0..400 {
            let position = glide.at(step as f32 * 4.0);
            assert!((0.0..=1000.0).contains(&position.rest) && position.over == 0.0);
        }
        near(
            glide.at(10000.0).rest,
            100.0 + Fling::new(1.0).travelled(f32::MAX),
            0.01,
        );
        assert!(glide.at(10000.0).done);
    }

    #[test]
    fn a_fling_toward_the_start_bounces_at_zero() {
        let glide = Glide::new(30.0, -2.0, 400.0, 0.0);
        let low = (0..300)
            .map(|step| glide.at(step as f32 * 2.0).over)
            .fold(0.0f32, f32::min);
        assert!(low < -10.0);
        assert_eq!(glide.at(3000.0).rest, 0.0);
    }

    #[test]
    fn a_slower_release_than_the_smallest_fling_does_not_glide() {
        let glide = Glide::new(100.0, 0.01, 1000.0, 0.0);
        assert!(glide.at(0.0).done);
        assert_eq!(glide.at(0.0).rest, 100.0);
    }

    #[test]
    fn the_release_velocity_is_the_speed_of_the_last_tenth_of_a_second() {
        let mut tracker = VelocityTracker::default();
        for step in 0..30 {
            tracker.push(step as f32 * 10.0, 20.0);
        }
        near(tracker.velocity(290.0), 2.0, 0.01);
        near(tracker.velocity(350.0), 2.0, 0.01);
    }

    #[test]
    fn a_pause_before_the_release_means_no_velocity() {
        let mut tracker = VelocityTracker::default();
        for step in 0..10 {
            tracker.push(step as f32 * 10.0, 20.0);
        }
        assert_eq!(tracker.velocity(250.0), 0.0);
        assert_eq!(VelocityTracker::default().velocity(10.0), 0.0);
    }

    #[test]
    fn only_the_last_tenth_of_a_second_counts() {
        let mut tracker = VelocityTracker::default();
        for step in 0..10 {
            tracker.push(step as f32 * 10.0, 100.0);
        }
        for step in 10..21 {
            tracker.push(step as f32 * 10.0, 10.0);
        }
        near(tracker.velocity(200.0), 1.0, 0.01);
    }
}

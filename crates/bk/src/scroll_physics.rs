use std::collections::VecDeque;

use ctdesign::{SCROLL_FLING_POWER, SCROLL_PHYSICS, SCROLL_SETTLED_OVER, SCROLL_SETTLED_SPEED};

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
            .is_some_and(|(first, _)| at - first > SCROLL_PHYSICS.velocity_window_ms)
        {
            self.samples.pop_front();
        }
    }

    pub(crate) fn velocity(&self, now: f32) -> f32 {
        let (Some(&(first, _)), Some(&(last, _))) = (self.samples.front(), self.samples.back())
        else {
            return 0.0;
        };
        if now - last > SCROLL_PHYSICS.velocity_window_ms || last <= first {
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
pub(crate) fn rubber_band(travel: f32, dimension: f32, band: f32) -> f32 {
    let dimension = dimension.max(1.0);
    travel.signum() * dimension * (1.0 - 1.0 / (1.0 + band * travel.abs() / dimension))
}

// The input that stretches the content by `over`.
pub(crate) fn rubber_band_input(over: f32, dimension: f32, band: f32) -> f32 {
    let dimension = dimension.max(1.0);
    let over = over.abs().min(dimension * 0.999).copysign(over);
    dimension * over / (band * (dimension - over.abs()))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Fling {
    speed: f32,
    distance: f32,
    duration: f32,
}

impl Fling {
    pub(crate) fn new(speed: f32) -> Self {
        let distance =
            SCROLL_PHYSICS.fling_distance * speed.abs().powf(SCROLL_PHYSICS.fling_exponent);
        Self {
            speed,
            distance: distance.copysign(speed),
            duration: if speed == 0.0 {
                0.0
            } else {
                SCROLL_FLING_POWER as f32 * distance / speed.abs()
            },
        }
    }

    pub(crate) fn duration(self) -> f32 {
        self.duration
    }

    pub(crate) fn travelled(self, at: f32) -> f32 {
        let left = 1.0 - (at / self.duration.max(f32::EPSILON)).clamp(0.0, 1.0);
        self.distance * (1.0 - left.powi(SCROLL_FLING_POWER))
    }

    pub(crate) fn speed_at(self, at: f32) -> f32 {
        let left = 1.0 - (at / self.duration.max(f32::EPSILON)).clamp(0.0, 1.0);
        self.speed * left.powi(SCROLL_FLING_POWER - 1)
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
        let omega = SCROLL_PHYSICS.return_omega / 1000.0;
        (self.over + (self.speed + omega * self.over) * at) * (-omega * at).exp()
    }

    fn speed_at(self, at: f32) -> f32 {
        let omega = SCROLL_PHYSICS.return_omega / 1000.0;
        let linear = self.speed + omega * self.over;
        (self.speed - omega * linear * at) * (-omega * at).exp()
    }

    fn settles_at(self) -> f32 {
        let mut at = 0.0;
        while at < 4000.0
            && (self.over_at(at).abs() > SCROLL_SETTLED_OVER
                || self.speed_at(at).abs() > SCROLL_SETTLED_SPEED)
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
        let fling = if over == 0.0 && speed.abs() >= SCROLL_PHYSICS.min_speed {
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
                    speed: SCROLL_PHYSICS.band_speed * arrival,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Linear {
    pub domain: (f64, f64),
    pub range: (f64, f64),
}

impl Linear {
    pub fn map(&self, value: f64) -> f64 {
        let (d0, d1) = self.domain;
        let (r0, r1) = self.range;
        if d1 == d0 {
            return (r0 + r1) / 2.0;
        }
        r0 + (value - d0) / (d1 - d0) * (r1 - r0)
    }
}

pub fn nice_ticks(min: f64, max: f64, desired: usize) -> Vec<f64> {
    let (min, max) = if max > min {
        (min, max)
    } else {
        (min - 1.0, min + 1.0)
    };
    let raw = (max - min) / desired.max(1) as f64;
    let magnitude = 10f64.powf(raw.log10().floor());
    let step = [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .map(|factor| factor * magnitude)
        .find(|step| *step >= raw)
        .unwrap_or(10.0 * magnitude);
    let first = (min / step).floor() as i64;
    let last = (max / step).ceil() as i64;
    (first..=last).map(|i| i as f64 * step).collect()
}

pub fn stack(series: &[Vec<f64>]) -> Vec<Vec<(f64, f64)>> {
    let len = series.iter().map(Vec::len).max().unwrap_or(0);
    let mut base = vec![0.0; len];
    series
        .iter()
        .map(|values| {
            values
                .iter()
                .enumerate()
                .map(|(i, value)| {
                    let lower = base[i];
                    base[i] += value;
                    (lower, base[i])
                })
                .collect()
        })
        .collect()
}

pub fn sector_angles(values: &[f64]) -> Vec<(f32, f32)> {
    let total: f64 = values.iter().copied().filter(|v| *v > 0.0).sum();
    let mut start = -90.0f32;
    values
        .iter()
        .map(|value| {
            let sweep = if total > 0.0 {
                (value.max(0.0) / total * 360.0) as f32
            } else {
                0.0
            };
            let sector = (start, sweep);
            start += sweep;
            sector
        })
        .collect()
}

pub fn monotone_controls(points: &[(f64, f64)]) -> Vec<[(f64, f64); 2]> {
    let n = points.len();
    if n < 2 {
        return Vec::new();
    }
    let slopes: Vec<f64> = points
        .windows(2)
        .map(|w| (w[1].1 - w[0].1) / (w[1].0 - w[0].0))
        .collect();
    let mut tangents = vec![0.0; n];
    tangents[0] = slopes[0];
    tangents[n - 1] = slopes[n - 2];
    for i in 1..n - 1 {
        tangents[i] = if slopes[i - 1] * slopes[i] <= 0.0 {
            0.0
        } else {
            (slopes[i - 1] + slopes[i]) / 2.0
        };
    }
    for i in 0..n - 1 {
        if slopes[i] == 0.0 {
            tangents[i] = 0.0;
            tangents[i + 1] = 0.0;
            continue;
        }
        let (a, b) = (tangents[i] / slopes[i], tangents[i + 1] / slopes[i]);
        let h = a.hypot(b);
        if h > 3.0 {
            tangents[i] = 3.0 / h * a * slopes[i];
            tangents[i + 1] = 3.0 / h * b * slopes[i];
        }
    }
    (0..n - 1)
        .map(|i| {
            let (x0, y0) = points[i];
            let (x1, y1) = points[i + 1];
            let dx = (x1 - x0) / 3.0;
            [
                (x0 + dx, y0 + tangents[i] * dx),
                (x1 - dx, y1 - tangents[i + 1] * dx),
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nice_ticks_cover_the_data_with_round_steps() {
        assert_eq!(
            nice_ticks(0.0, 97.0, 5),
            vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0]
        );
        assert_eq!(nice_ticks(0.3, 1.7, 4), vec![0.0, 0.5, 1.0, 1.5, 2.0]);
    }

    #[test]
    fn a_flat_domain_still_gets_ticks() {
        let ticks = nice_ticks(5.0, 5.0, 4);
        assert!(ticks.first().unwrap() < &5.0 && ticks.last().unwrap() > &5.0);
    }

    #[test]
    fn stacked_series_start_where_the_previous_ends() {
        let bands = stack(&[vec![1.0, 2.0], vec![3.0, 4.0]]);
        assert_eq!(bands[1], vec![(1.0, 4.0), (2.0, 6.0)]);
    }

    #[test]
    fn sectors_split_the_full_turn_by_share() {
        let sectors = sector_angles(&[1.0, 3.0]);
        assert_eq!(sectors, vec![(-90.0, 90.0), (0.0, 270.0)]);
    }

    #[test]
    fn monotone_curve_never_overshoots_the_points() {
        let points = [(0.0, 0.0), (1.0, 10.0), (2.0, 10.0), (3.0, 0.0)];
        for [(_, a), (_, b)] in monotone_controls(&points) {
            assert!((0.0..=10.0).contains(&a) && (0.0..=10.0).contains(&b));
        }
    }

    #[test]
    fn linear_maps_the_domain_ends_to_the_range_ends() {
        let scale = Linear {
            domain: (0.0, 100.0),
            range: (200.0, 0.0),
        };
        assert_eq!(scale.map(0.0), 200.0);
        assert_eq!(scale.map(100.0), 0.0);
    }
}

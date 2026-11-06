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

// Byron & Wattenberg, "Stacked Graphs – Geometry & Aesthetics" (2008): the baseline that minimizes weighted wiggle, as d3.stackOffsetWiggle.
pub fn wiggle_baseline(series: &[Vec<f64>]) -> Vec<f64> {
    let len = series.iter().map(Vec::len).min().unwrap_or(0);
    let mut baseline = vec![0.0; len];
    for j in 1..len {
        let (mut total, mut weighted) = (0.0, 0.0);
        let mut below = 0.0;
        for values in series {
            let change = values[j] - values[j - 1];
            weighted += values[j] * (below + change / 2.0);
            below += change;
            total += values[j];
        }
        baseline[j] = baseline[j - 1] - if total > 0.0 { weighted / total } else { 0.0 };
    }
    baseline
}

pub fn inside_out(series: &[Vec<f64>]) -> Vec<usize> {
    let peak = |values: &Vec<f64>| {
        values
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map_or(0, |(i, _)| i)
    };
    let mut by_peak: Vec<usize> = (0..series.len()).collect();
    by_peak.sort_by_key(|i| peak(&series[*i]));
    let (mut top, mut bottom) = (Vec::new(), Vec::new());
    let (mut top_sum, mut bottom_sum) = (0.0, 0.0);
    for i in by_peak {
        let sum: f64 = series[i].iter().sum();
        if top_sum < bottom_sum {
            top.push(i);
            top_sum += sum;
        } else {
            bottom.push(i);
            bottom_sum += sum;
        }
    }
    bottom.reverse();
    bottom.into_iter().chain(top).collect()
}

pub fn stream_bands(series: &[Vec<f64>]) -> (Vec<usize>, Vec<Vec<(f64, f64)>>) {
    let order = inside_out(series);
    let ordered: Vec<Vec<f64>> = order.iter().map(|i| series[*i].clone()).collect();
    let baseline = wiggle_baseline(&ordered);
    let len = baseline.len();
    let totals: Vec<f64> = (0..len)
        .map(|j| ordered.iter().map(|v| v[j]).sum())
        .collect();
    let center = (0..len).map(|j| baseline[j] + totals[j] / 2.0).sum::<f64>() / len.max(1) as f64;
    let mut lower: Vec<f64> = baseline.iter().map(|b| b - center).collect();
    let bands = ordered
        .iter()
        .map(|values| {
            (0..len)
                .map(|j| {
                    let band = (lower[j], lower[j] + values[j]);
                    lower[j] = band.1;
                    band
                })
                .collect()
        })
        .collect();
    (order, bands)
}

pub fn monotone_samples(points: &[(f64, f64)], steps: usize) -> Vec<(f64, f64)> {
    let Some(first) = points.first() else {
        return Vec::new();
    };
    let mut samples = vec![*first];
    for (controls, (start, end)) in monotone_controls(points)
        .iter()
        .zip(points.iter().zip(&points[1..]))
    {
        for step in 1..=steps.max(1) {
            let t = step as f64 / steps.max(1) as f64;
            let u = 1.0 - t;
            let bezier = |a: f64, b: f64, c: f64, d: f64| {
                u * u * u * a + 3.0 * u * u * t * b + 3.0 * u * t * t * c + t * t * t * d
            };
            samples.push((
                bezier(start.0, controls[0].0, controls[1].0, end.0),
                bezier(start.1, controls[0].1, controls[1].1, end.1),
            ));
        }
    }
    samples
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Window {
    pub start: f64,
    pub span: f64,
}

impl Window {
    pub fn full(total: f64) -> Self {
        Self {
            start: 0.0,
            span: total.max(1.0),
        }
    }

    pub fn fraction(&self, position: f64) -> f64 {
        (position - self.start) / self.span
    }

    pub fn position(&self, fraction: f64) -> f64 {
        self.start + fraction * self.span
    }

    fn clamped(self, total: f64) -> Self {
        let span = self.span.min(total.max(1.0));
        Self {
            start: self.start.clamp(0.0, (total - span).max(0.0)),
            span,
        }
    }

    pub fn zoom_at(self, fraction: f64, factor: f64, total: f64, min_span: f64) -> Self {
        let anchor = self.position(fraction);
        let span = (self.span / factor).clamp(min_span.min(total), total.max(1.0));
        Self {
            start: anchor - fraction * span,
            span,
        }
        .clamped(total)
    }

    pub fn pan(self, delta: f64, total: f64) -> Self {
        Self {
            start: self.start + delta,
            ..self
        }
        .clamped(total)
    }
}

const TIME_STEPS: [f64; 15] = [
    1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0, 300.0, 600.0, 900.0, 1800.0, 3600.0, 7200.0,
    21600.0,
];

pub fn time_ticks(span: f64, desired: usize) -> Vec<f64> {
    let step = TIME_STEPS
        .into_iter()
        .find(|step| span / step <= desired.max(1) as f64)
        .unwrap_or(86400.0);
    (0..=(span / step).floor() as usize)
        .map(|i| i as f64 * step)
        .collect()
}

pub fn format_age(seconds: f64, now: &str) -> String {
    match seconds.round() as i64 {
        0 => now.to_string(),
        s if s < 60 => format!("−{s} s"),
        s if s < 3600 => format!("−{} min", s / 60),
        s => format!("−{} h", s / 3600),
    }
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
    #[test]
    fn a_lone_stream_grows_evenly_about_the_center() {
        let (_, bands) = stream_bands(&[vec![2.0, 4.0, 6.0]]);
        assert_eq!(bands[0], vec![(-1.0, 1.0), (-2.0, 2.0), (-3.0, 3.0)]);
    }

    #[test]
    fn steady_streams_keep_a_flat_baseline() {
        assert_eq!(wiggle_baseline(&[vec![1.0; 4], vec![3.0; 4]]), vec![0.0; 4]);
    }

    #[test]
    fn inside_out_puts_the_earliest_peaks_in_the_middle() {
        let late = vec![0.0, 0.0, 9.0];
        let early = vec![9.0, 0.0, 0.0];
        let middle = vec![0.0, 9.0, 0.0];
        let order = inside_out(&[late, early, middle]);
        assert_eq!(order[order.len() / 2], 1);
    }

    #[test]
    fn monotone_samples_pass_through_every_point() {
        let points = [(0.0, 0.0), (1.0, 4.0), (2.0, 1.0)];
        let samples = monotone_samples(&points, 4);
        assert_eq!(samples.len(), 9);
        assert_eq!(samples[4], (1.0, 4.0));
        assert_eq!(samples[8], (2.0, 1.0));
    }
    #[test]
    fn zooming_keeps_the_point_under_the_pointer_in_place() {
        let window = Window::full(100.0).zoom_at(0.25, 2.0, 100.0, 4.0);
        assert_eq!(
            window,
            Window {
                start: 12.5,
                span: 50.0
            }
        );
        assert_eq!(window.position(0.25), 25.0);
    }

    #[test]
    fn zoom_and_pan_never_leave_the_data() {
        let tight = Window::full(100.0).zoom_at(1.0, 1000.0, 100.0, 4.0);
        assert_eq!(
            tight,
            Window {
                start: 96.0,
                span: 4.0
            }
        );
        assert_eq!(tight.pan(50.0, 100.0).start, 96.0);
        assert_eq!(tight.pan(-500.0, 100.0).start, 0.0);
        assert_eq!(
            Window::full(100.0).zoom_at(0.5, 0.1, 100.0, 4.0),
            Window::full(100.0)
        );
    }

    #[test]
    fn a_minute_of_history_gets_ticks_every_fifteen_seconds() {
        assert_eq!(time_ticks(59.0, 5), vec![0.0, 15.0, 30.0, 45.0]);
        assert_eq!(
            time_ticks(3600.0, 5),
            vec![0.0, 900.0, 1800.0, 2700.0, 3600.0]
        );
    }

    #[test]
    fn ages_read_as_now_seconds_minutes_or_hours() {
        assert_eq!(format_age(0.0, "ahora"), "ahora");
        assert_eq!(format_age(30.0, "ahora"), "−30 s");
        assert_eq!(format_age(120.0, "ahora"), "−2 min");
        assert_eq!(format_age(7200.0, "ahora"), "−2 h");
    }
}

//! O1 of ADR 0102: a spring of the kit differs from the recording of the real one by at most MAX_ERROR of the travel after the start is
//! aligned within one frame. The references are SwiftUI's `Spring(duration: 0.344, bounce: 0)` sampled at 60 fps by the probe `swiftuitx`.

use std::time::Duration;

use bk::animation::Spring;

const MAX_ERROR: f32 = 0.02;
const SMOOTH: Duration = Duration::from_millis(344);

fn reference(csv: &str) -> (Vec<(f32, f32)>, Option<f32>) {
    let interrupt = csv.lines().next().and_then(|line| {
        let after = line.split("at ").nth(1)?;
        after.split(' ').next()?.parse().ok()
    });
    let samples = csv
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let (t, v) = line.split_once(',')?;
            Some((t.parse().ok()?, v.parse().ok()?))
        })
        .collect();
    (samples, interrupt)
}

fn shifts() -> impl Iterator<Item = f32> {
    (-17..=17).map(|ms| ms as f32 / 1000.0)
}

fn secs(t: f32) -> Duration {
    Duration::from_secs_f32(t.max(0.0))
}

fn worst(samples: &[(f32, f32)], model: impl Fn(f32) -> f32) -> f32 {
    samples
        .iter()
        .map(|&(t, v)| (v - model(t)).abs())
        .fold(0.0, f32::max)
}

#[test]
fn a_smooth_spring_matches_the_swiftui_recording() {
    let (samples, _) = reference(include_str!("../reference/swiftui-spring-smooth.csv"));
    let spring = Spring::new(0.0, 1.0, SMOOTH, 0.0);
    let error = shifts()
        .map(|shift| worst(&samples, |t| spring.value(secs(t - shift))))
        .fold(f32::MAX, f32::min);
    assert!(error <= MAX_ERROR, "{error}");
}

#[test]
fn an_interrupted_smooth_spring_matches_the_swiftui_recording() {
    let (samples, interrupt) =
        reference(include_str!("../reference/swiftui-spring-interrupted.csv"));
    let interrupt = interrupt.expect("the interruption time is in the header");
    let first = Spring::new(0.0, 1.0, SMOOTH, 0.0);
    let error = shifts()
        .flat_map(|start| shifts().map(move |cut| (start, cut)))
        .map(|(start, cut)| {
            let at = secs(interrupt + cut - start);
            let back =
                Spring::new(first.value(at), 0.0, SMOOTH, 0.0).with_velocity(first.velocity(at));
            worst(&samples, |t| {
                let t = t - start;
                if t < interrupt + cut - start {
                    first.value(secs(t))
                } else {
                    back.value(secs(t - (interrupt + cut - start)))
                }
            })
        })
        .fold(f32::MAX, f32::min);
    assert!(error <= MAX_ERROR, "{error}");
}

// A 60 fps recording shows each frame at an instant known only within one frame step, so after the best alignment a curve is
// known to within the largest change of the curve in half a step. The live check of a kit window against its recording uses this
// floor, not MAX_ERROR (ADR 0102).
fn recording_floor(period: Duration) -> f32 {
    let spring = Spring::new(0.0, 1.0, period, 0.0);
    let half_step = 1.0 / 120.0;
    (0..2000)
        .map(|i| {
            let t = i as f32 / 2000.0;
            (spring.value(secs(t)) - spring.value(secs(t - half_step))).abs()
        })
        .fold(0.0, f32::max)
}

#[test]
fn a_60_fps_recording_cannot_tell_a_smooth_spring_apart_closer_than_its_floor() {
    let floor = recording_floor(SMOOTH);
    assert!((0.05..0.06).contains(&floor), "{floor}");
    assert!(recording_floor(Duration::from_millis(274)) > floor);
}

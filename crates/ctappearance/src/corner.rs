use std::f32::consts::{FRAC_PI_2, SQRT_2};

use ctdesign::CORNER_CONCENTRIC_REGION;

const ROWS: usize = 16;

fn bezier(p: [f32; 4], t: f32) -> f32 {
    let u = 1.0 - t;
    u * u * u * p[0] + 3.0 * u * u * t * p[1] + 3.0 * u * t * t * p[2] + t * t * t * p[3]
}

fn solve(y_points: [f32; 4], x_points: [f32; 4], y: f32) -> f32 {
    let (mut low, mut high) = (0.0_f32, 1.0_f32);
    for _ in 0..40 {
        let middle = 0.5 * (low + high);
        if bezier(y_points, middle) < y {
            low = middle;
        } else {
            high = middle;
        }
    }
    bezier(x_points, 0.5 * (low + high))
}

pub fn edge(radius: f32, smoothing: f32, y: f32) -> f32 {
    if radius <= 0.0 || y >= (1.0 + smoothing) * radius {
        return 0.0;
    }
    if y <= 0.0 {
        return (1.0 + smoothing) * radius;
    }
    let p = (1.0 + smoothing) * radius;
    let arc_measure = FRAC_PI_2 * (1.0 - smoothing);
    let arc_section = (arc_measure / 2.0).sin() * radius * SQRT_2;
    let theta = (FRAC_PI_2 - arc_measure) / 2.0;
    let beta = FRAC_PI_2 / 2.0 * smoothing;
    let c = radius * (theta / 2.0).tan() * beta.cos();
    let d = c * beta.tan();
    let b = (p - arc_section - c - d) / 3.0;
    let a = 2.0 * b;
    let corner_end = p - a - b - c;
    if y <= d {
        return solve([0.0, 0.0, 0.0, d], [p, p - a, p - a - b, corner_end], y);
    }
    if y <= corner_end {
        let up = radius - y;
        return radius - (radius * radius - up * up).max(0.0).sqrt();
    }
    solve([corner_end, p - a - b, p - a, p], [d, 0.0, 0.0, 0.0], y)
}

pub fn coverage(radius: f32, smoothing: f32, x: f32, y: f32, edge_x: f32, edge_y: f32) -> f32 {
    let mut inside = 0.0;
    for row in 0..ROWS {
        let depth = y + (row as f32 + 0.5) / ROWS as f32 - edge_y;
        if depth < 0.0 {
            continue;
        }
        inside += (x + 1.0 - edge_x - edge(radius, smoothing, depth)).clamp(0.0, 1.0);
    }
    inside / ROWS as f32
}

pub fn tile(radius: f32, smoothing: f32, size: usize, edge_x: f32, edge_y: f32) -> Vec<f32> {
    (0..size * size)
        .map(|index| {
            coverage(
                radius,
                smoothing,
                (index % size) as f32,
                (index / size) as f32,
                edge_x,
                edge_y,
            )
        })
        .collect()
}

pub fn concentric(container: f32, inset_x: f32, inset_y: f32, minimum: f32) -> f32 {
    let limit = container * CORNER_CONCENTRIC_REGION;
    if inset_x > limit || inset_y > limit {
        return minimum;
    }
    (container - inset_x.min(inset_y)).max(minimum)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ctdesign::{CORNER_POPOVER, CORNER_SMOOTHING, CORNER_WINDOW, WINDOW_CORNER};

    const SAMPLES: &str = include_str!("../tests/data/corner-samples.csv");

    // Mirrors window_sdf and window_coverage of malaysia (grade-graphics/src/window_shape.glsl); change both together.
    fn superellipse_coverage(extent: f32, exponent: f32, x: f32, y: f32) -> f32 {
        let dx = (extent - (x + 0.5)).max(0.0);
        let dy = (extent - (y + 0.5)).max(0.0);
        if dx <= 0.0 && dy <= 0.0 {
            return 1.0;
        }
        (0.5 - ((dx.powf(exponent) + dy.powf(exponent)).powf(1.0 / exponent) - extent))
            .clamp(0.0, 1.0)
    }

    struct Sample {
        edge_x: f32,
        edge_y: f32,
        size: usize,
        alpha: Vec<f32>,
    }

    fn sample(name: &str) -> Sample {
        let line = SAMPLES
            .lines()
            .find(|line| line.split(',').next() == Some(name))
            .expect("sample");
        let fields: Vec<&str> = line.split(',').skip(1).collect();
        let size: usize = fields[0].parse().unwrap();
        Sample {
            edge_x: fields[1].parse::<f32>().unwrap() / 1000.0,
            edge_y: fields[2].parse::<f32>().unwrap() / 1000.0,
            size,
            alpha: fields[3..]
                .iter()
                .map(|value| value.parse::<f32>().unwrap() / 255.0)
                .collect(),
        }
    }

    fn rms_and_max(sample: &Sample, shown: impl Fn(usize, usize) -> f32) -> (f32, f32) {
        let (mut sum, mut max) = (0.0, 0.0_f32);
        for row in 0..sample.size {
            for column in 0..sample.size {
                let error = shown(column, row) - sample.alpha[row * sample.size + column];
                sum += error * error;
                max = max.max(error.abs());
            }
        }
        ((sum / (sample.size * sample.size) as f32).sqrt(), max)
    }

    fn continuous(sample: &Sample, radius: f32) -> (f32, f32) {
        rms_and_max(sample, |x, y| {
            coverage(
                radius,
                CORNER_SMOOTHING,
                x as f32,
                y as f32,
                sample.edge_x,
                sample.edge_y,
            )
        })
    }

    #[test]
    fn the_circle_has_the_arc_and_nothing_else() {
        assert!((edge(16.0, 0.0, 0.0) - 16.0).abs() < 1e-4);
        assert!(edge(16.0, 0.0, 16.0).abs() < 1e-4);
        let diagonal = 16.0 * (1.0 - std::f32::consts::FRAC_1_SQRT_2);
        assert!((edge(16.0, 0.0, diagonal) - diagonal).abs() < 1e-3);
    }

    #[test]
    fn the_continuous_corner_reaches_further_along_the_edges_and_meets_the_circle_on_the_diagonal()
    {
        let smoothing = CORNER_SMOOTHING;
        assert!((edge(16.0, smoothing, 0.0) - 16.0 * (1.0 + smoothing)).abs() < 1e-3);
        let diagonal = 16.0 * (1.0 - std::f32::consts::FRAC_1_SQRT_2);
        assert!((edge(16.0, smoothing, diagonal) - diagonal).abs() < 1e-2);
    }

    #[test]
    fn a_circular_layer_through_the_window_server_is_the_exact_circle() {
        let measured = sample("layer-circular-16");
        let (rms, _) = rms_and_max(&measured, |x, y| {
            coverage(16.0, 0.0, x as f32, y as f32, 0.0, 0.0)
        });
        assert!(rms < 0.02, "rms {rms}");
    }

    #[test]
    fn a_continuous_layer_has_its_nominal_radius_and_the_measured_smoothing() {
        for (name, radius) in [
            ("layer-continuous-8", 8.0),
            ("layer-continuous-16", 16.0),
            ("layer-continuous-26", 26.0),
        ] {
            let measured = sample(name);
            let (rms, max) = continuous(&measured, radius);
            assert!(rms < 0.02 && max < 0.2, "{name}: rms {rms} max {max}");
            let (wrong, _) = continuous(&measured, radius + 2.0);
            assert!(wrong > 2.0 * rms, "{name}: the radius is fixed by the data");
        }
    }

    #[test]
    fn a_window_silhouette_is_the_continuous_corner_of_the_silhouette_radius() {
        let measured = sample("window-16");
        let (rms, max) = continuous(&measured, CORNER_WINDOW.fitted);
        assert!(rms < 0.02 && max < 0.15, "rms {rms} max {max}");
        let (nominal, _) = continuous(&measured, CORNER_WINDOW.radius);
        assert!(
            nominal > 2.0 * rms,
            "the nominal radius is not the silhouette: rms {nominal}"
        );
    }

    #[test]
    fn a_sheet_has_the_corner_of_a_window() {
        let measured = sample("sheet-16");
        let (rms, _) = continuous(&measured, CORNER_WINDOW.fitted);
        assert!(rms < 0.02, "rms {rms}");
    }

    #[test]
    fn a_popover_corner_is_a_continuous_corner_off_the_pixel_grid() {
        let measured = sample("popover-240x160");
        let (rms, max) = continuous(&measured, CORNER_POPOVER.fitted);
        assert!(rms < 0.02 && max < 0.2, "rms {rms} max {max}");
    }

    #[test]
    fn the_compositors_superellipse_stand_in_follows_a_continuous_layer_within_its_tolerance() {
        for (name, radius) in [("layer-continuous-16", 16.0), ("layer-continuous-26", 26.0)] {
            let measured = sample(name);
            let (rms, _) = rms_and_max(&measured, |x, y| {
                superellipse_coverage(
                    radius * WINDOW_CORNER.expansion,
                    WINDOW_CORNER.exponent,
                    x as f32,
                    y as f32,
                )
            });
            assert!(rms < 0.016, "{name}: rms {rms}");
        }
    }

    #[test]
    fn the_superellipse_with_the_api_radius_is_too_square_for_a_window() {
        let measured = sample("window-16");
        let (rms, _) = rms_and_max(&measured, |x, y| {
            superellipse_coverage(
                16.0 * WINDOW_CORNER.expansion,
                WINDOW_CORNER.exponent,
                x as f32,
                y as f32,
            )
        });
        let (silhouette, _) = rms_and_max(&measured, |x, y| {
            superellipse_coverage(
                CORNER_WINDOW.fitted * WINDOW_CORNER.expansion,
                WINDOW_CORNER.exponent,
                x as f32,
                y as f32,
            )
        });
        assert!(rms > 2.0 * silhouette, "{rms} against {silhouette}");
    }

    #[test]
    fn the_concentric_radius_is_the_container_minus_the_smaller_inset() {
        assert_eq!(concentric(16.0, 0.0, 0.0, 0.0), 16.0);
        assert_eq!(concentric(16.0, 4.0, 4.0, 0.0), 12.0);
        assert_eq!(concentric(16.0, 0.0, 12.0, 0.0), 16.0);
        assert_eq!(concentric(16.0, 12.0, 4.0, 0.0), 12.0);
        assert_eq!(concentric(16.0, 20.0, 20.0, 0.0), 0.0);
    }

    #[test]
    fn the_concentric_minimum_and_region_follow_the_api() {
        assert_eq!(concentric(16.0, 16.0, 16.0, 2.0), 2.0);
        assert_eq!(concentric(16.0, 16.0, 0.0, 0.0), 16.0);
        assert_eq!(concentric(16.0, 17.0, 0.0, 0.0), 0.0);
        assert_eq!(concentric(16.0, 17.0, 0.0, 6.0), 6.0);
        assert_eq!(concentric(16.0, 8.0, 8.0, 12.0), 12.0);
        assert_eq!(concentric(26.0, 6.0, 6.0, 0.0), 20.0);
        assert_eq!(
            concentric(concentric(20.0, 0.0, 0.0, 0.0), 6.0, 6.0, 0.0),
            14.0
        );
    }
}

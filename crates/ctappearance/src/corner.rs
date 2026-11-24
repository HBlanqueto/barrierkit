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

pub fn ease_out(progress: f32) -> f32 {
    1.0 - (1.0 - progress).powi(3)
}

pub fn ease_in_out(progress: f32) -> f32 {
    if progress < 0.5 {
        4.0 * progress.powi(3)
    } else {
        1.0 - (2.0 - 2.0 * progress).powi(3) / 2.0
    }
}

// Core Animation's `easeInEaseOut`, the cubic Bézier (0.42, 0, 0.58, 1): AppKit's sidebar shows and hides along it.
pub fn ease_in_ease_out(progress: f32) -> f32 {
    let bezier = |a: f32, b: f32, s: f32| {
        3.0 * (1.0 - s).powi(2) * s * a + 3.0 * (1.0 - s) * s * s * b + s.powi(3)
    };
    let (mut low, mut high) = (0.0f32, 1.0f32);
    for _ in 0..24 {
        let middle = (low + high) / 2.0;
        if bezier(0.42, 0.58, middle) < progress {
            low = middle;
        } else {
            high = middle;
        }
    }
    bezier(0.0, 1.0, (low + high) / 2.0)
}

pub fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

pub fn progress(start: f32, now: f32, length: f32) -> f32 {
    if length <= 0.0 {
        return if now >= start { 1.0 } else { 0.0 };
    }
    ((now - start) / length).clamp(0.0, 1.0)
}

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

// Core Animation's `CAMediaTimingFunction`: the cubic Bézier through (0, 0), (x1, y1), (x2, y2), (1, 1), as a function of the progress.
pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, progress: f32) -> f32 {
    let bezier = |a: f32, b: f32, s: f32| {
        3.0 * (1.0 - s).powi(2) * s * a + 3.0 * (1.0 - s) * s * s * b + s.powi(3)
    };
    let (mut low, mut high) = (0.0f32, 1.0f32);
    for _ in 0..24 {
        let middle = (low + high) / 2.0;
        if bezier(x1, x2, middle) < progress {
            low = middle;
        } else {
            high = middle;
        }
    }
    bezier(y1, y2, (low + high) / 2.0)
}

// Core Animation's `easeInEaseOut`, the cubic Bézier (0.42, 0, 0.58, 1): AppKit's sidebar shows and hides along it.
pub fn ease_in_ease_out(progress: f32) -> f32 {
    cubic_bezier(0.42, 0.0, 0.58, 1.0, progress)
}

// Core Animation's default timing function, the Bézier (0.25, 0.1, 0.25, 1): SwiftUI's list, disclosure and section changes run along it.
pub fn ease_default(progress: f32) -> f32 {
    cubic_bezier(0.25, 0.1, 0.25, 1.0, progress)
}

// Core Animation's `easeOut`, the Bézier (0, 0, 0.58, 1): `NSOutlineView` expands along it. The cubic `ease_out` is another curve.
pub fn ease_out_appkit(progress: f32) -> f32 {
    cubic_bezier(0.0, 0.0, 0.58, 1.0, progress)
}

// The slider knob's press and release, fitted to AppKit: the Bézier (0, 0.13, 0.53, 1.11) overshoots a little.
pub fn slider_press(progress: f32) -> f32 {
    cubic_bezier(0.0, 0.13, 0.53, 1.11, progress)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_animation_default_reaches_four_fifths_at_half_time() {
        assert!((ease_default(0.5) - 0.8024).abs() < 2e-3);
    }

    #[test]
    fn cubic_bezier_starts_at_zero_ends_at_one_and_overshoots() {
        assert!(slider_press(0.0).abs() < 1e-3);
        assert!((slider_press(1.0) - 1.0).abs() < 1e-3);
        assert!((1..100).any(|i| slider_press(i as f32 / 100.0) > 1.0));
    }
}

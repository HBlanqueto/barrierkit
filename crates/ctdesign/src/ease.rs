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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_curve_starts_at_zero_and_ends_at_one() {
        for curve in [ease_out, ease_in_out, ease_in_ease_out] {
            assert!(curve(0.0).abs() < 1e-6);
            assert!((curve(1.0) - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn ease_in_out_is_half_way_at_the_middle() {
        assert!((ease_in_out(0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn no_curve_goes_backwards() {
        for curve in [ease_out, ease_in_out, ease_in_ease_out] {
            let mut previous = -1.0f32;
            for step in 0..=100 {
                let value = curve(step as f32 / 100.0);
                assert!(value >= previous, "step {step}: {value} < {previous}");
                previous = value;
            }
        }
    }

    #[test]
    fn ease_in_ease_out_is_core_animations_bezier() {
        for (progress, expected) in [(0.1, 0.0197), (0.25, 0.1292), (0.5, 0.5), (0.75, 0.8708)] {
            assert!((ease_in_ease_out(progress) - expected).abs() < 1e-3);
        }
    }

    #[test]
    fn smoothstep_starts_at_zero_and_ends_at_one() {
        assert!(smoothstep(0.0).abs() < 1e-6);
        assert!((smoothstep(1.0) - 1.0).abs() < 1e-6);
        assert!((smoothstep(0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn progress_clamps_to_the_unit_range() {
        assert!(progress(0.0, 0.0, 1.0).abs() < 1e-6);
        assert!((progress(0.0, 0.5, 1.0) - 0.5).abs() < 1e-6);
        assert!((progress(0.0, 2.0, 1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn progress_with_no_length_is_on_or_off() {
        assert_eq!(progress(0.0, 1.0, 0.0), 1.0);
        assert_eq!(progress(1.0, 0.0, 0.0), 0.0);
    }
}

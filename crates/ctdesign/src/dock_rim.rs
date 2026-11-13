use crate::{DOCK_RIM_ALPHA, DOCK_RIM_RAMP, DOCK_RIM_TINT, Rgba};

/// The light of the Dock's rim over a backdrop of this colour: `rim` (white, with the opacity the
/// state of the Dock measured over a reference backdrop) tinted by the backdrop and as strong as
/// the backdrop is bright, so it fades where the wallpaper is dark.
pub fn dock_rim(backdrop: Rgba, rim: Rgba) -> Rgba {
    let [dark, per_brightness, reference] = DOCK_RIM_ALPHA;
    let [lift, share] = DOCK_RIM_TINT;
    let brightness = backdrop.r.max(backdrop.g).max(backdrop.b);
    let strength = (dark + per_brightness * brightness) / (dark + per_brightness * reference);
    let tinted = |channel: f32| 1.0 + (channel + (1.0 - channel) * lift - 1.0) * share;
    Rgba {
        r: tinted(backdrop.r),
        g: tinted(backdrop.g),
        b: tinted(backdrop.b),
        a: (rim.a * strength).min(1.0),
    }
}

/// How much of the Dock's frame is the dark outline and how much the light rim, at a height of
/// the bar from 0 (top) to 1 (bottom): all outline along the sides, all rim on top and bottom,
/// handing over around the corners.
pub fn dock_rim_dark_share(height: f32) -> f32 {
    let [from, to] = DOCK_RIM_RAMP;
    let distance = height.min(1.0 - height);
    let progress = ((distance - from) / (to - from)).clamp(0.0, 1.0);
    progress * progress * (3.0 - 2.0 * progress)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DOCK_RIM_GLOW;

    fn backdrop(r: f32, g: f32, b: f32) -> Rgba {
        Rgba { r, g, b, a: 1.0 }
    }

    const MEASURED: Rgba = Rgba {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.22,
    };

    #[test]
    fn the_rim_is_the_measured_one_over_the_reference_backdrop() {
        let reference = DOCK_RIM_ALPHA[2];
        let rim = dock_rim(backdrop(reference, reference, reference), MEASURED);
        assert!((rim.a - MEASURED.a).abs() < 1e-6, "{rim:?}");
    }

    #[test]
    fn the_rim_is_fainter_over_a_dark_backdrop_than_over_a_bright_one() {
        let dark = dock_rim(backdrop(0.05, 0.08, 0.12), MEASURED).a;
        let bright = dock_rim(backdrop(0.9, 0.9, 0.9), MEASURED).a;
        assert!(dark < MEASURED.a && MEASURED.a < bright, "{dark} {bright}");
    }

    #[test]
    fn the_rim_is_white_tinted_toward_the_backdrop() {
        let white = dock_rim(backdrop(1.0, 1.0, 1.0), MEASURED);
        assert_eq!((white.r, white.g, white.b), (1.0, 1.0, 1.0));
        let blue = dock_rim(backdrop(0.0, 0.0, 1.0), MEASURED);
        assert!(blue.r < blue.b && blue.b == 1.0, "{blue:?}");
        assert!(blue.r > 0.5, "still mostly white: {blue:?}");
    }

    #[test]
    fn the_frame_is_outline_on_the_sides_and_rim_on_top_and_bottom_and_symmetric() {
        assert_eq!(dock_rim_dark_share(0.0), 0.0);
        assert_eq!(dock_rim_dark_share(1.0), 0.0);
        assert_eq!(dock_rim_dark_share(0.5), 1.0);
        for step in 0..=20 {
            let height = step as f32 / 20.0;
            let share = dock_rim_dark_share(height);
            assert!((share - dock_rim_dark_share(1.0 - height)).abs() < 1e-6);
            assert!((0.0..=1.0).contains(&share));
        }
        assert!(dock_rim_dark_share(0.3) > 0.0 && dock_rim_dark_share(0.3) < 1.0);
    }

    #[test]
    fn the_glow_inside_the_rim_dies_away() {
        assert!(DOCK_RIM_GLOW.windows(2).all(|pair| pair[0] > pair[1]));
    }
}

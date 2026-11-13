use crate::{DOCK_RIM_ALPHA, DOCK_RIM_FADE, DOCK_RIM_TINT, Rgba};

/// The Dock's rim over a backdrop of this colour: white light tinted by the backdrop, as opaque as
/// the backdrop is bright (macOS).
pub fn dock_rim(backdrop: Rgba) -> Rgba {
    let [dark, per_brightness] = DOCK_RIM_ALPHA;
    let [lift, share] = DOCK_RIM_TINT;
    let brightness = backdrop.r.max(backdrop.g).max(backdrop.b);
    let tinted = |channel: f32| 1.0 + (channel + (1.0 - channel) * lift - 1.0) * share;
    Rgba {
        r: tinted(backdrop.r),
        g: tinted(backdrop.g),
        b: tinted(backdrop.b),
        a: dark + per_brightness * brightness,
    }
}

/// How much of the rim shows from the top of the bar (0) to its bottom (1): full on top, thinning
/// along the side, gone around the lower corner and back on the bottom edge. `corner` is the
/// corner radius over the bar's height.
pub fn dock_rim_fade(corner: f32) -> [(f32, f32); 6] {
    let [top, upper_corner, side, lower_start, lower_corner, bottom] = DOCK_RIM_FADE;
    [
        (0.0, top),
        (corner * 0.8, upper_corner),
        (0.5, side),
        (1.0 - corner * 0.9, lower_start),
        (1.0 - corner * 0.45, lower_corner),
        (1.0, bottom),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn backdrop(r: f32, g: f32, b: f32) -> Rgba {
        Rgba { r, g, b, a: 1.0 }
    }

    #[test]
    fn the_rim_is_fainter_over_a_dark_backdrop_than_over_a_bright_one() {
        let dark = dock_rim(backdrop(0.05, 0.08, 0.12)).a;
        let bright = dock_rim(backdrop(0.2, 0.5, 0.86)).a;
        assert!(dark < bright);
        assert_eq!(dock_rim(backdrop(0.0, 0.0, 0.0)).a, DOCK_RIM_ALPHA[0]);
        assert_eq!(dock_rim(backdrop(1.0, 1.0, 1.0)).a, DOCK_RIM_ALPHA[0] + DOCK_RIM_ALPHA[1]);
    }

    #[test]
    fn the_rim_is_white_tinted_toward_the_backdrop() {
        let white = dock_rim(backdrop(1.0, 1.0, 1.0));
        assert_eq!((white.r, white.g, white.b), (1.0, 1.0, 1.0));
        let blue = dock_rim(backdrop(0.0, 0.0, 1.0));
        assert!(blue.r < blue.b && blue.b == 1.0, "{blue:?}");
        assert!(blue.r > 0.5, "still mostly white: {blue:?}");
    }

    #[test]
    fn the_fade_runs_down_the_bar_and_disappears_around_the_lower_corner() {
        let fade = dock_rim_fade(0.24);
        assert!(fade.windows(2).all(|pair| pair[0].0 < pair[1].0), "{fade:?}");
        assert_eq!(fade[0].1, 1.0);
        let lowest = fade.iter().map(|stop| stop.1).fold(1.0, f32::min);
        assert_eq!(lowest, fade[4].1);
        assert!(fade[5].1 > fade[4].1 && fade[5].1 < fade[0].1);
    }
}

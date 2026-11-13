use crate::{DOCK_RIM_RAMP, DockSurface, Rgba};

/// The light of the Dock's rim over a backdrop of this colour. macOS lights the rim to a
/// luminance that follows the glass's: `lift` plus `share` of it (all between 0 and 1). The rim is
/// white, as opaque as that takes over the glass, so it fades where the glass is dark and
/// brightens where it is light.
pub fn dock_rim(surface: DockSurface, backdrop: Rgba) -> Rgba {
    let glass = glass_luminance(surface, backdrop);
    let [lift, share] = surface.rim;
    overlay(glass, lift + share * glass - glass)
}

/// The divider between groups of icons over a backdrop: the glass's luminance moved by
/// `surface.divider`, as a white or black overlay.
pub fn dock_divider(surface: DockSurface, backdrop: Rgba) -> Rgba {
    let glass = glass_luminance(surface, backdrop);
    overlay(glass, surface.divider)
}

// The glass over the backdrop, as the luminance of what shows through it.
fn glass_luminance(surface: DockSurface, backdrop: Rgba) -> f32 {
    let over = |behind: f32, glass: f32| behind * (1.0 - surface.glass.a) + glass * surface.glass.a;
    0.2126 * over(backdrop.r, surface.glass.r)
        + 0.7152 * over(backdrop.g, surface.glass.g)
        + 0.0722 * over(backdrop.b, surface.glass.b)
}

// The overlay that moves a luminance by `shift`: white to lighten it, black to darken it.
fn overlay(luminance: f32, shift: f32) -> Rgba {
    let (channel, room) = if shift >= 0.0 {
        (1.0, 1.0 - luminance)
    } else {
        (0.0, luminance)
    };
    Rgba {
        r: channel,
        g: channel,
        b: channel,
        a: (shift.abs() / room.max(1e-3)).min(1.0),
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
    use crate::{
        DOCK_DIVIDER_DARK, DOCK_DIVIDER_LIGHT, DOCK_RIM_DARK, DOCK_RIM_GLOW, DOCK_RIM_LIGHT,
    };

    fn grey(level: u8) -> Rgba {
        let value = f32::from(level) / 255.0;
        Rgba {
            r: value,
            g: value,
            b: value,
            a: 1.0,
        }
    }

    fn solid(glass: u8, rim: [f32; 2], divider: f32) -> DockSurface {
        DockSurface {
            glass: grey(glass),
            outline: grey(0),
            rim,
            divider,
        }
    }

    // Luminance (0..255) of white at this opacity over grey `level`.
    fn lit(level: u8, overlay: Rgba) -> f32 {
        let base = f32::from(level);
        let covered = base * (1.0 - overlay.a) + overlay.r * 255.0 * overlay.a;
        covered.round()
    }

    #[test]
    fn the_solid_docks_have_the_rims_and_dividers_measured_in_macos() {
        let dark = solid(10, DOCK_RIM_DARK, DOCK_DIVIDER_DARK);
        let light = solid(117, DOCK_RIM_LIGHT, DOCK_DIVIDER_LIGHT);
        let any = grey(200);
        assert_eq!(lit(10, dock_rim(dark, any)), 81.0);
        assert_eq!(lit(10, dock_divider(dark, any)), 77.0);
        assert_eq!(lit(117, dock_rim(light, any)), 177.0);
        assert_eq!(lit(117, dock_divider(light, any)), 59.0);
    }

    #[test]
    fn a_translucent_dock_takes_the_backdrop_through_the_glass() {
        // Dark glass (grey 39 at 30.5 %): 12 over black and 144 over grey 190, as measured.
        let glass = Rgba {
            a: 0.305,
            ..grey(39)
        };
        let dark = DockSurface {
            glass,
            ..solid(0, DOCK_RIM_DARK, DOCK_DIVIDER_DARK)
        };
        let over_black = dock_rim(dark, grey(0));
        assert_eq!(lit(12, over_black), 83.0, "{over_black:?}");
        let over_grey = dock_rim(dark, grey(190));
        assert!((lit(144, over_grey) - 168.0).abs() <= 1.0, "{over_grey:?}");
        assert!((lit(144, dock_divider(dark, grey(190))) - 211.0).abs() <= 1.0);
    }

    #[test]
    fn a_light_translucent_dock_matches_the_captures_over_black_and_over_grey() {
        // Light glass (grey 154 at 24.7 %): 38 over black and 181 over grey 190.
        let light = DockSurface {
            glass: Rgba {
                a: 0.247,
                ..grey(154)
            },
            ..solid(0, DOCK_RIM_LIGHT, DOCK_DIVIDER_LIGHT)
        };
        assert!((lit(38, dock_rim(light, grey(0))) - 98.0).abs() <= 1.0);
        assert!((lit(181, dock_rim(light, grey(190))) - 241.0).abs() <= 1.0);
        assert_eq!(lit(181, dock_divider(light, grey(190))), 123.0);
    }

    #[test]
    fn the_light_rim_is_a_fixed_step_above_the_glass_and_the_dark_one_a_shrinking_step() {
        let clear = |rim, divider| DockSurface {
            glass: Rgba { a: 0.0, ..grey(0) },
            ..solid(0, rim, divider)
        };
        let light = clear(DOCK_RIM_LIGHT, DOCK_DIVIDER_LIGHT);
        let dark = clear(DOCK_RIM_DARK, DOCK_DIVIDER_DARK);
        let step = |surface: DockSurface, level: u8| {
            lit(level, dock_rim(surface, grey(level))) - f32::from(level)
        };
        assert!((step(light, 38) - step(light, 181)).abs() <= 1.0);
        assert!(step(dark, 12) > step(dark, 144));
    }

    #[test]
    fn a_rim_cannot_go_beyond_white_or_below_nothing() {
        let light = solid(0, DOCK_RIM_LIGHT, DOCK_DIVIDER_LIGHT);
        for level in [0, 128, 255] {
            let rim = dock_rim(light, grey(level));
            assert!((0.0..=1.0).contains(&rim.a), "{level}: {rim:?}");
        }
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

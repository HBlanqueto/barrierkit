use crate::{DockSurface, Rgba};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DOCK_DIVIDER_DARK, DOCK_DIVIDER_LIGHT, DOCK_RIM_CORNER_LIGHT};

    fn grey(level: u8) -> Rgba {
        let value = f32::from(level) / 255.0;
        Rgba {
            r: value,
            g: value,
            b: value,
            a: 1.0,
        }
    }

    fn glass(glass: Rgba, divider: f32) -> DockSurface {
        DockSurface {
            glass,
            outline: grey(0),
            rim: [0.0, 1.0],
            divider,
            corner: DOCK_RIM_CORNER_LIGHT,
            tone: None,
        }
    }

    // Luminance (0..255) of white or black at this opacity over grey `level`.
    fn over(level: u8, overlay: Rgba) -> f32 {
        let base = f32::from(level);
        (base * (1.0 - overlay.a) + overlay.r * 255.0 * overlay.a).round()
    }

    #[test]
    fn the_solid_docks_have_the_dividers_measured_in_macos() {
        let any = grey(200);
        assert_eq!(
            over(10, dock_divider(glass(grey(10), DOCK_DIVIDER_DARK), any)),
            77.0
        );
        assert_eq!(
            over(117, dock_divider(glass(grey(117), DOCK_DIVIDER_LIGHT), any)),
            59.0
        );
    }

    #[test]
    fn a_translucent_divider_takes_the_backdrop_through_the_glass() {
        let dark = glass(
            Rgba {
                a: 0.305,
                ..grey(39)
            },
            DOCK_DIVIDER_DARK,
        );
        assert!((over(144, dock_divider(dark, grey(190))) - 211.0).abs() <= 1.0);
        let light = glass(
            Rgba {
                a: 0.247,
                ..grey(154)
            },
            DOCK_DIVIDER_LIGHT,
        );
        assert_eq!(over(181, dock_divider(light, grey(190))), 123.0);
    }
}

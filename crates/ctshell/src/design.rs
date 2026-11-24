use crate::{DockSurface, Rgba};

/// A white or black overlay that moves the glass's luminance over `backdrop` by `surface.divider`.
pub fn dock_divider(surface: DockSurface, backdrop: Rgba) -> Rgba {
    let glass = glass_luminance(surface, backdrop);
    overlay(glass, surface.divider)
}

fn glass_luminance(surface: DockSurface, backdrop: Rgba) -> f32 {
    let over = |behind: f32, glass: f32| behind * (1.0 - surface.glass.a) + glass * surface.glass.a;
    Rgba {
        r: over(backdrop.r, surface.glass.r),
        g: over(backdrop.g, surface.glass.g),
        b: over(backdrop.b, surface.glass.b),
        a: 1.0,
    }
    .luma()
}

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

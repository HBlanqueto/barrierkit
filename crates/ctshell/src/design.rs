use ctdesign::{CornerShape, DockSurface, Font, MaterialLook, Rgba};

pub const CIRCLE_EXPONENT: f32 = 2.0;

pub const ALERT_LAYER_NAMESPACE: &str = "malaysia-dialog";
pub const NOTIFICATIONS_LAYER_NAMESPACE: &str = "malaysia-notifications";
pub const MENUBAR_LAYER_NAMESPACE: &str = "malaysia-menubar";
pub const DOCK_LAYER_NAMESPACE: &str = "malaysia-dock";
pub const WALLPAPER_LAYER_NAMESPACE: &str = "malaysia-wallpaper";

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

pub const RIM_BINS: usize = 35;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassLook {
    pub blur: f32,
    pub blur_opacity: [f32; 4],
    pub blur_distance: [f32; 4],
    pub white: f32,
    pub black: f32,
    pub saturation: f32,
    pub max_luma: f32,
    pub fill: Rgba,
    pub refraction: [f32; 2],
    pub tone_delta: [f32; 9],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassRim {
    pub angle_1: [f32; 12],
    pub angle_2: [f32; 12],
    pub distance_1: [f32; RIM_BINS],
    pub distance_2: [f32; RIM_BINS],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassScale {
    pub refraction_amount: [f32; 4],
    pub refraction_height: [f32; 4],
    pub blur_opacity_0: [f32; 4],
    pub blur_opacity_1: [f32; 4],
    pub blur_distance_0: [f32; 4],
    pub max_luma: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassSet {
    pub active: GlassLook,
    pub inactive: GlassLook,
    pub reduced: GlassLook,
    pub increased_contrast: GlassLook,
    pub reduced_contrast: GlassLook,
    pub rim: GlassRim,
    pub scale: GlassScale,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassLens {
    pub scale: f32,
    pub decay: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassTint {
    pub blur: [f32; 3],
    pub blur_opacity: [f32; 4],
    pub blur_fill_opacity: [f32; 3],
    pub blur_fill_radius: f32,
    pub fill_rgb: [f32; 3],
    pub fill_alpha: [f32; 3],
    pub max_luma: [f32; 3],
    pub read_halvings: [f32; 2],
    pub tone_shift_t25: [f32; 9],
    pub tone_shift_t50: [f32; 9],
    pub tone_shift_t75: [f32; 9],
    pub tone_shift_t100: [f32; 9],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShellGlass {
    pub sigma: f32,
    pub halo_sigma: f32,
    pub halo_weight: f32,
    pub tone: [f32; 12],
    pub chroma: [f32; 5],
}

include!(concat!(env!("OUT_DIR"), "/tengen.rs"));

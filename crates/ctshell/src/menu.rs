use ctappearance::backdrop::Oklab;
use ctdesign::{DockSurface, Rgba};

const CAP_SOFTNESS: f32 = 0.1;

fn smooth_union(a: f32, b: f32, softness: f32) -> f32 {
    let blend = (softness - (a - b).abs()).max(0.0) / softness;
    a.min(b) - blend * blend * softness * 0.25
}

pub fn panel_over(glass: &DockSurface, behind: Rgba) -> Rgba {
    let seen = behind.mix(glass.glass, glass.glass.a);
    let Some(tone) = glass.tone else {
        return seen;
    };
    let lifted = Oklab::from_rgba(seen).l + tone.lift;
    let lightness = smooth_union(lifted, tone.cap, CAP_SOFTNESS);
    let chroma = tone.chroma * (lightness / lifted.max(0.001)).min(1.0);
    let behind = Oklab::from_rgba(behind);
    Oklab {
        l: lightness,
        a: behind.a * chroma,
        b: behind.b * chroma,
    }
    .to_rgba()
}

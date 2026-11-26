use ctdesign::{
    ICON_LOGO_CLEAR, ICON_LOGO_CLEAR_DARK, ICON_LOGO_TINTED, ICON_LOGO_TINTED_DARK, ICON_PLATE,
    ICON_PLATE_CLEAR, ICON_PLATE_CLEAR_DARK, ICON_PLATE_GRAPHITE, ICON_PLATE_LOBE_DARK,
    ICON_PLATE_LOBE_LIGHT, ICON_PLATE_SILVER, ICON_PLATE_TINTED, ICON_PLATE_TINTED_DARK,
    IconLogoEnds, IconPlateLook, Rgba,
};

use crate::backdrop::Oklab;
use crate::icon_style::{IconStyle, duotone, gray};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lobe {
    pub hue: f32,
    pub chroma: f32,
    pub lightness: f32,
    pub share: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plate {
    pub top: Rgba,
    pub bottom: Rgba,
    /// Along the top edge and along the bottom edge.
    pub light: (Rgba, Rgba),
    /// Closed round clear and tinted plates; transparent on the others.
    pub outline: Rgba,
    pub glass: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Logo {
    pub lobe: Option<Lobe>,
    pub bright: bool,
}

impl Logo {
    /// `colors` are the opaque pixels of an app icon, in sRGB.
    pub fn from_colors(colors: &[Rgba]) -> Self {
        let colors: Vec<Oklab> = colors.iter().map(|&c| Oklab::from_rgba(c)).collect();
        let mean = colors.iter().map(|c| c.l).sum::<f32>() / colors.len().max(1) as f32;
        let bright = mean >= ICON_PLATE.bright_lightness;
        let candidates = lobes(&colors)
            .into_iter()
            .filter(|lobe| lobe.share >= ICON_PLATE.lobe_min_share);
        let lobe = if bright {
            candidates.min_by(|a, b| a.lightness.total_cmp(&b.lightness))
        } else {
            candidates.max_by(|a, b| a.lightness.total_cmp(&b.lightness))
        };
        Self { lobe, bright }
    }
}

fn hue_chroma(color: Oklab) -> (f32, f32) {
    let hue = color.b.atan2(color.a).to_degrees().rem_euclid(360.0);
    (hue, color.a.hypot(color.b))
}

fn lch(color: Rgba) -> (f32, f32, f32) {
    let color = Oklab::from_rgba(color);
    let (hue, chroma) = hue_chroma(color);
    (color.l, chroma, hue)
}

/// Runs of hue bins that hold chroma, each with its chroma-weighted hue, chroma, lightness and share.
fn lobes(colors: &[Oklab]) -> Vec<Lobe> {
    let chromatic: Vec<(f32, f32, f32)> = colors
        .iter()
        .map(|&color| {
            let (hue, chroma) = hue_chroma(color);
            (hue, chroma, color.l)
        })
        .filter(|&(_, chroma, _)| chroma >= ICON_PLATE.neutral_chroma)
        .collect();
    let bin =
        |hue: f32| (hue / (360.0 / ICON_PLATE.hue_bins as f32)) as usize % ICON_PLATE.hue_bins;
    let mut weight = [0.0_f32; ICON_PLATE.hue_bins];
    for &(hue, chroma, _) in &chromatic {
        weight[bin(hue)] += chroma;
    }
    let total: f32 = weight.iter().sum();
    let peak = weight.iter().copied().fold(0.0, f32::max);
    if total == 0.0 {
        return Vec::new();
    }
    let occupied: Vec<bool> = weight
        .iter()
        .map(|&w| w >= ICON_PLATE.hue_bin_occupied * peak)
        .collect();
    let start = occupied.iter().position(|&o| !o).map_or(0, |gap| gap + 1);
    let order: Vec<usize> = (0..ICON_PLATE.hue_bins)
        .map(|step| (start + step) % ICON_PLATE.hue_bins)
        .collect();
    order
        .split(|&index| !occupied[index])
        .filter(|run| !run.is_empty())
        .map(|run| {
            let members: Vec<_> = chromatic
                .iter()
                .filter(|&&(hue, ..)| run.contains(&bin(hue)))
                .collect();
            let weight: f32 = members.iter().map(|m| m.1).sum();
            let sin: f32 = members.iter().map(|m| m.1 * m.0.to_radians().sin()).sum();
            let cos: f32 = members.iter().map(|m| m.1 * m.0.to_radians().cos()).sum();
            Lobe {
                hue: sin.atan2(cos).to_degrees().rem_euclid(360.0),
                chroma: members.iter().map(|m| m.1 * m.1).sum::<f32>() / weight,
                lightness: members.iter().map(|m| m.1 * m.2).sum::<f32>() / weight,
                share: weight / total,
            }
        })
        .collect()
}

/// An OKLCH color; the chroma shrinks until it fits the sRGB gamut.
fn fit(lightness: f32, chroma: f32, hue: f32) -> Rgba {
    let mut chroma = chroma;
    loop {
        let angle = hue.to_radians();
        let color = Oklab {
            l: lightness,
            a: chroma * angle.cos(),
            b: chroma * angle.sin(),
        };
        let linear = color.to_linear();
        let inside = [linear.r, linear.g, linear.b]
            .iter()
            .all(|channel| (-0.002..=1.002).contains(channel));
        if inside || chroma <= 0.0 {
            return color.to_rgba();
        }
        chroma = (chroma - 0.004).max(0.0);
    }
}

fn shade(look: &IconPlateLook, hue: f32, chroma: f32) -> (Rgba, Rgba) {
    let top = fit(
        look.top_lightness,
        (chroma * look.top_chroma_scale).min(look.top_chroma_cap),
        hue,
    );
    let bottom = fit(
        look.bottom_lightness,
        (chroma * look.bottom_chroma_scale).min(look.bottom_chroma_cap),
        hue + look.bottom_hue_shift,
    );
    (
        Rgba {
            a: look.opacity_top,
            ..top
        },
        Rgba {
            a: look.opacity_bottom,
            ..bottom
        },
    )
}

/// The plate for an icon: its own hue by default, graphite when dark, translucent for clear and
/// the tint for tinted; `dark` is `IconStyle::dark`.
pub fn plate(style: IconStyle, dark: bool, tint: Rgba, logo: &Logo) -> Plate {
    let (tint_hue, tint_chroma) = hue_chroma(Oklab::from_rgba(tint));
    let (look, hue, chroma) = match (style, dark) {
        (IconStyle::Default | IconStyle::Dark, false) => {
            let look = match (logo.bright, logo.lobe) {
                (true, None) => &ICON_PLATE_SILVER,
                (true, Some(_)) => &ICON_PLATE_LOBE_DARK,
                (false, _) => &ICON_PLATE_LOBE_LIGHT,
            };
            let (hue, chroma) = logo.lobe.map_or((0.0, 0.0), |lobe| (lobe.hue, lobe.chroma));
            (look, hue, chroma)
        }
        (IconStyle::Default | IconStyle::Dark, true) => {
            let (hue, chroma) = logo.lobe.map_or((0.0, 0.0), |lobe| (lobe.hue, lobe.chroma));
            (&ICON_PLATE_GRAPHITE, hue, chroma)
        }
        (IconStyle::Clear, false) => (&ICON_PLATE_CLEAR, 0.0, 0.0),
        (IconStyle::Clear, true) => (&ICON_PLATE_CLEAR_DARK, 0.0, 0.0),
        (IconStyle::Tinted, false) => (&ICON_PLATE_TINTED, tint_hue, tint_chroma),
        (IconStyle::Tinted, true) => (&ICON_PLATE_TINTED_DARK, tint_hue, tint_chroma),
    };
    let (top, bottom) = shade(look, hue, chroma);
    edged(look, top, bottom)
}

/// The plate and the glyph colour of a symbol tile, as Settings' sidebar draws it, in the icon
/// style; `color` is the tile's own. `None` keeps the tile's own glyph colour.
pub fn tile(style: IconStyle, dark: bool, tint: Rgba, color: Rgba) -> (Plate, Option<Rgba>) {
    let (lightness, chroma, hue) = lch(color);
    match (style, dark) {
        (IconStyle::Default | IconStyle::Dark, false) => {
            let top = fit(lightness + ICON_PLATE.tile_top_lift, chroma, hue);
            let bottom = fit(lightness + ICON_PLATE.tile_bottom_lift, chroma, hue);
            (unlit(edged(&ICON_PLATE_LOBE_DARK, top, bottom)), None)
        }
        (IconStyle::Default | IconStyle::Dark, true) => {
            let (top, bottom) = shade(&ICON_PLATE_GRAPHITE, 0.0, 0.0);
            let glyph = fit(lightness.max(ICON_PLATE.dark_glyph_lightness), chroma, hue);
            (unlit(edged(&ICON_PLATE_GRAPHITE, top, bottom)), Some(glyph))
        }
        _ => {
            let logo = Logo {
                lobe: None,
                bright: false,
            };
            let glyph = logo_ends(style, dark, tint).map(|(_, light)| light);
            (plate(style, dark, tint, &logo), glyph)
        }
    }
}

// The Dock's toned rim: the colour scaled per channel to `lift` more luma, with its own chroma.
fn lifted(color: Rgba, lift: f32) -> Rgba {
    let luma = color.luma().max(0.001);
    let gain = (luma + lift) / luma;
    let scaled = Rgba {
        r: (color.r * gain).min(1.0),
        g: (color.g * gain).min(1.0),
        b: (color.b * gain).min(1.0),
        a: 1.0,
    };
    let (_, chroma, hue) = lch(color);
    fit(Oklab::from_rgba(scaled).l, chroma, hue)
}

fn unlit(plate: Plate) -> Plate {
    Plate {
        light: (Rgba::TRANSPARENT, Rgba::TRANSPARENT),
        ..plate
    }
}

fn edged(look: &IconPlateLook, top: Rgba, bottom: Rgba) -> Plate {
    let light = |color: Rgba, a: f32| {
        let light = if look.light_tone > 0.0 {
            lifted(color, look.light_tone)
        } else {
            gray(1.0)
        };
        Rgba { a, ..light }
    };
    let (lightness, chroma, hue) = lch(top);
    Plate {
        top,
        bottom,
        light: (light(top, look.light_top), light(bottom, look.light_bottom)),
        outline: Rgba {
            a: look.outline_alpha,
            ..fit(lightness * look.outline_lightness, chroma, hue)
        },
        glass: look.opacity_top < 1.0,
    }
}

fn ends(ends: &IconLogoEnds, hue: f32, chroma: f32) -> (Rgba, Rgba) {
    (
        fit(
            ends.dark_lightness,
            (chroma * ends.dark_chroma_scale).min(ends.dark_chroma_cap),
            hue,
        ),
        fit(
            ends.light_lightness,
            (chroma * ends.light_chroma_scale).min(ends.light_chroma_cap),
            hue,
        ),
    )
}

fn logo_ends(style: IconStyle, dark: bool, tint: Rgba) -> Option<(Rgba, Rgba)> {
    let (tint_hue, tint_chroma) = hue_chroma(Oklab::from_rgba(tint));
    match (style, dark) {
        (IconStyle::Default | IconStyle::Dark, _) => None,
        (IconStyle::Clear, false) => Some(ends(&ICON_LOGO_CLEAR, 0.0, 0.0)),
        (IconStyle::Clear, true) => Some(ends(&ICON_LOGO_CLEAR_DARK, 0.0, 0.0)),
        (IconStyle::Tinted, false) => Some(ends(&ICON_LOGO_TINTED, tint_hue, tint_chroma)),
        (IconStyle::Tinted, true) => Some(ends(&ICON_LOGO_TINTED_DARK, tint_hue, tint_chroma)),
    }
}

/// The color matrix for the icon drawn on its plate; `None` keeps its colors.
pub fn logo_matrix(style: IconStyle, dark: bool, tint: Rgba) -> Option<[f32; 20]> {
    logo_ends(style, dark, tint).map(|(dark_end, light_end)| duotone(dark_end, light_end))
}

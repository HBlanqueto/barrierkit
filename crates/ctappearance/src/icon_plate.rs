use ctdesign::{
    ICON_LOGO_CLEAR, ICON_LOGO_CLEAR_DARK, ICON_LOGO_TINTED, ICON_LOGO_TINTED_DARK, ICON_PLATE,
    ICON_PLATE_CLEAR, ICON_PLATE_CLEAR_DARK, ICON_PLATE_GRAPHITE, ICON_PLATE_LOBE_DARK,
    ICON_PLATE_LOBE_LIGHT, ICON_PLATE_TINTED, ICON_PLATE_TINTED_DARK, IconLogoEnds, IconPlateLook,
    Rgba,
};

use crate::backdrop::Oklab;
use crate::icon_style::{IconStyle, LUMA, duotone, gray};

const BINS: usize = 24;
const OCCUPIED: f32 = 0.03;

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
    let bin = |hue: f32| (hue / (360.0 / BINS as f32)) as usize % BINS;
    let mut weight = [0.0_f32; BINS];
    for &(hue, chroma, _) in &chromatic {
        weight[bin(hue)] += chroma;
    }
    let total: f32 = weight.iter().sum();
    let peak = weight.iter().copied().fold(0.0, f32::max);
    if total == 0.0 {
        return Vec::new();
    }
    let occupied: Vec<bool> = weight.iter().map(|&w| w >= OCCUPIED * peak).collect();
    let start = occupied.iter().position(|&o| !o).map_or(0, |gap| gap + 1);
    let mut runs: Vec<Vec<usize>> = Vec::new();
    let mut run = Vec::new();
    for step in 0..BINS {
        let index = (start + step) % BINS;
        if occupied[index] {
            run.push(index);
        } else if !run.is_empty() {
            runs.push(std::mem::take(&mut run));
        }
    }
    if !run.is_empty() {
        runs.push(run);
    }
    runs.iter()
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
            let look = if logo.bright {
                &ICON_PLATE_LOBE_DARK
            } else {
                &ICON_PLATE_LOBE_LIGHT
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
            (edged(&ICON_PLATE_LOBE_DARK, top, bottom), None)
        }
        (IconStyle::Default | IconStyle::Dark, true) => {
            let (top, bottom) = shade(&ICON_PLATE_GRAPHITE, 0.0, 0.0);
            let glyph = fit(lightness.max(ICON_PLATE.dark_glyph_lightness), chroma, hue);
            (edged(&ICON_PLATE_GRAPHITE, top, bottom), Some(glyph))
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
    let luma = (LUMA[0] * color.r + LUMA[1] * color.g + LUMA[2] * color.b).max(0.001);
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

#[cfg(test)]
mod tests {
    use super::*;

    const AMBER: Rgba = Rgba::rgb8(0xe8, 0xa3, 0x3d);

    fn pixels(parts: &[(Rgba, usize)]) -> Vec<Rgba> {
        parts
            .iter()
            .flat_map(|&(color, count)| std::iter::repeat_n(color, count))
            .collect()
    }

    #[test]
    fn a_bright_logo_takes_the_hue_of_its_darkest_color() {
        let logo = Logo::from_colors(&pixels(&[
            (Rgba::rgb8(0xff, 0xd3, 0x4d), 70),
            (Rgba::rgb8(0x6a, 0x1b, 0x9a), 30),
        ]));
        assert!(logo.bright);
        let hue = logo.lobe.unwrap().hue;
        assert!((290.0..340.0).contains(&hue), "{hue}");
    }

    #[test]
    fn a_dark_logo_takes_the_hue_of_its_lightest_color() {
        let logo = Logo::from_colors(&pixels(&[
            (Rgba::rgb8(0x2a, 0x10, 0x38), 70),
            (Rgba::rgb8(0xff, 0xd3, 0x4d), 30),
        ]));
        assert!(!logo.bright);
        let hue = logo.lobe.unwrap().hue;
        assert!((70.0..120.0).contains(&hue), "{hue}");
    }

    #[test]
    fn a_gray_logo_has_no_hue_to_take() {
        let logo = Logo::from_colors(&pixels(&[(Rgba::rgb8(0x80, 0x80, 0x80), 100)]));
        assert_eq!(logo.lobe, None);
    }

    #[test]
    fn a_small_color_is_not_a_lobe() {
        let logo = Logo::from_colors(&pixels(&[
            (Rgba::rgb8(0xff, 0xd3, 0x4d), 95),
            (Rgba::rgb8(0x6a, 0x1b, 0x9a), 5),
        ]));
        assert!((70.0..120.0).contains(&logo.lobe.unwrap().hue));
    }

    #[test]
    fn dark_is_graphite_whatever_the_icon() {
        let logo = Logo::from_colors(&pixels(&[(Rgba::rgb8(0xff, 0x50, 0x20), 100)]));
        let plate = plate(IconStyle::Dark, true, AMBER, &logo);
        assert!((plate.top.r - plate.top.b).abs() < 0.01 && plate.top.r < 0.3);
        assert!(!plate.glass);
    }

    #[test]
    fn clear_is_see_through_and_tinted_is_the_tint() {
        let logo = Logo::from_colors(&pixels(&[(Rgba::rgb8(0xff, 0x50, 0x20), 100)]));
        let clear = plate(IconStyle::Clear, false, AMBER, &logo);
        assert!(clear.glass && clear.top.a < 0.3);
        let tinted = plate(IconStyle::Tinted, false, AMBER, &logo);
        assert!(tinted.top.r > tinted.top.b && tinted.top.a == 1.0);
    }

    #[test]
    fn clear_and_tinted_plates_are_outlined_and_default_and_dark_are_lit_white() {
        let logo = Logo::from_colors(&pixels(&[(Rgba::rgb8(0xff, 0x50, 0x20), 100)]));
        for (style, dark) in [(IconStyle::Clear, false), (IconStyle::Clear, true)]
            .into_iter()
            .chain([(IconStyle::Tinted, false), (IconStyle::Tinted, true)])
        {
            assert!(
                plate(style, dark, AMBER, &logo).outline.a > 0.5,
                "{style:?} {dark}"
            );
        }
        for (style, dark) in [(IconStyle::Default, false), (IconStyle::Dark, true)] {
            let plate = plate(style, dark, AMBER, &logo);
            assert_eq!(plate.outline.a, 0.0);
            assert!(plate.light.0.r == 1.0 && plate.light.0.b == 1.0);
            assert!(plate.light.0.a > plate.light.1.a);
        }
    }

    #[test]
    fn a_dark_tile_is_black_and_its_glyph_takes_the_tile_colour() {
        let blue = Rgba::rgb8(0x2f, 0x6f, 0xe4);
        let (plate, glyph) = tile(IconStyle::Dark, true, AMBER, blue);
        assert!(plate.top.r < 0.15 && plate.bottom.r < plate.top.r);
        let glyph = glyph.unwrap();
        assert!(glyph.b > glyph.r && glyph.b > 0.8);
        let graphite = Rgba::rgb8(0x2b, 0x2b, 0x2b);
        let (_, gray) = tile(IconStyle::Dark, true, AMBER, graphite);
        assert!(gray.unwrap().r > 0.5);
        let (light, glyph) = tile(IconStyle::Default, false, AMBER, blue);
        assert!(light.top.b > light.bottom.b && glyph.is_none());
    }

    #[test]
    fn a_gray_tint_has_no_hue_and_default_and_dark_have_no_logo_matrix() {
        let logo = Logo::from_colors(&pixels(&[(Rgba::rgb8(0xff, 0x50, 0x20), 100)]));
        let plate = plate(
            IconStyle::Tinted,
            false,
            Rgba::rgb8(0x8e, 0x8e, 0x8e),
            &logo,
        );
        assert!((plate.top.r - plate.top.b).abs() < 0.02);
        assert_eq!(logo_matrix(IconStyle::Default, true, AMBER), None);
        assert_eq!(logo_matrix(IconStyle::Dark, true, AMBER), None);
        assert!(logo_matrix(IconStyle::Tinted, true, AMBER).is_some());
    }
}

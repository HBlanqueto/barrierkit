use ctdesign::{
    ICON_LOGO_CLEAR, ICON_LOGO_CLEAR_DARK, ICON_LOGO_TINTED, ICON_LOGO_TINTED_DARK, ICON_PLATE,
    ICON_PLATE_CLEAR, ICON_PLATE_CLEAR_DARK, ICON_PLATE_GRAPHITE, ICON_PLATE_LOBE_DARK,
    ICON_PLATE_LOBE_LIGHT, ICON_PLATE_TINTED, ICON_PLATE_TINTED_DARK, IconLogoEnds, IconPlateLook,
    Rgba,
};

use crate::ColorScheme;
use crate::backdrop::Oklab;
use crate::icon_style::{IconStyle, duotone};

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
    pub rim: Rgba,
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

/// The plate for an icon: its own hue by default, graphite in dark, translucent for clear and the
/// accent for tinted; monochrome is tinted without chroma.
pub fn plate(style: IconStyle, scheme: ColorScheme, accent: Rgba, logo: &Logo) -> Plate {
    let dark = scheme == ColorScheme::Dark;
    let (accent_hue, accent_chroma) = hue_chroma(Oklab::from_rgba(accent));
    let (look, hue, chroma) = match (style, dark) {
        (IconStyle::Default, false) => {
            let look = if logo.bright {
                &ICON_PLATE_LOBE_DARK
            } else {
                &ICON_PLATE_LOBE_LIGHT
            };
            let (hue, chroma) = logo.lobe.map_or((0.0, 0.0), |lobe| (lobe.hue, lobe.chroma));
            (look, hue, chroma)
        }
        (IconStyle::Default, true) => {
            let (hue, chroma) = logo.lobe.map_or((0.0, 0.0), |lobe| (lobe.hue, lobe.chroma));
            (&ICON_PLATE_GRAPHITE, hue, chroma)
        }
        (IconStyle::Clear, false) => (&ICON_PLATE_CLEAR, 0.0, 0.0),
        (IconStyle::Clear, true) => (&ICON_PLATE_CLEAR_DARK, 0.0, 0.0),
        (IconStyle::Tinted, false) => (&ICON_PLATE_TINTED, accent_hue, accent_chroma),
        (IconStyle::Tinted, true) => (&ICON_PLATE_TINTED_DARK, accent_hue, accent_chroma),
        (IconStyle::Monochrome, false) => (&ICON_PLATE_TINTED, accent_hue, 0.0),
        (IconStyle::Monochrome, true) => (&ICON_PLATE_TINTED_DARK, accent_hue, 0.0),
    };
    let (top, bottom) = shade(look, hue, chroma);
    let rim = fit(
        ICON_PLATE.rim_lightness,
        ICON_PLATE.rim_chroma * (chroma / ICON_PLATE.full_chroma).min(1.0),
        hue,
    );
    Plate {
        top,
        bottom,
        rim: Rgba {
            a: look.rim_alpha,
            ..rim
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

/// The color matrix for the icon drawn on its plate; `None` keeps its colors.
pub fn logo_matrix(style: IconStyle, scheme: ColorScheme, accent: Rgba) -> Option<[f32; 20]> {
    let dark = scheme == ColorScheme::Dark;
    let (accent_hue, accent_chroma) = hue_chroma(Oklab::from_rgba(accent));
    let (dark_end, light_end) = match (style, dark) {
        (IconStyle::Default, _) => return None,
        (IconStyle::Monochrome, _) => return style.matrix(accent),
        (IconStyle::Clear, false) => ends(&ICON_LOGO_CLEAR, 0.0, 0.0),
        (IconStyle::Clear, true) => ends(&ICON_LOGO_CLEAR_DARK, 0.0, 0.0),
        (IconStyle::Tinted, false) => ends(&ICON_LOGO_TINTED, accent_hue, accent_chroma),
        (IconStyle::Tinted, true) => ends(&ICON_LOGO_TINTED_DARK, accent_hue, accent_chroma),
    };
    Some(duotone(dark_end, light_end))
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
        let plate = plate(IconStyle::Default, ColorScheme::Dark, AMBER, &logo);
        assert!((plate.top.r - plate.top.b).abs() < 0.01 && plate.top.r < 0.3);
        assert!(!plate.glass);
    }

    #[test]
    fn clear_is_see_through_and_tinted_is_the_accent() {
        let logo = Logo::from_colors(&pixels(&[(Rgba::rgb8(0xff, 0x50, 0x20), 100)]));
        let clear = plate(IconStyle::Clear, ColorScheme::Light, AMBER, &logo);
        assert!(clear.glass && clear.top.a < 0.3 && clear.top.a > clear.bottom.a);
        let tinted = plate(IconStyle::Tinted, ColorScheme::Light, AMBER, &logo);
        assert!(tinted.top.r > tinted.top.b && tinted.top.a == 1.0);
    }

    #[test]
    fn monochrome_has_no_hue_and_default_has_no_logo_matrix() {
        let logo = Logo::from_colors(&pixels(&[(Rgba::rgb8(0xff, 0x50, 0x20), 100)]));
        let plate = plate(IconStyle::Monochrome, ColorScheme::Light, AMBER, &logo);
        assert!((plate.top.r - plate.top.b).abs() < 0.02);
        assert_eq!(
            logo_matrix(IconStyle::Default, ColorScheme::Light, AMBER),
            None
        );
        assert!(logo_matrix(IconStyle::Tinted, ColorScheme::Dark, AMBER).is_some());
    }
}

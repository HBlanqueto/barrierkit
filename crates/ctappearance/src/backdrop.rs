use ctdesign::{BACKDROP, DOMINANT_LIGHTNESS, DOMINANT_MIN_CHROMA, LIGHT, Rgba};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Oklab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

// https://bottosson.github.io/posts/oklab/
impl Oklab {
    pub fn from_rgba(color: Rgba) -> Self {
        let Rgba { r, g, b, .. } = color.to_linear();
        let l = (0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b).cbrt();
        let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
        let s = (0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b).cbrt();
        Self {
            l: 0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
            a: 1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
            b: 0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
        }
    }

    pub fn to_rgba(self) -> Rgba {
        let l = (self.l + 0.396_337_78 * self.a + 0.215_803_76 * self.b).powi(3);
        let m = (self.l - 0.105_561_346 * self.a - 0.063_854_17 * self.b).powi(3);
        let s = (self.l - 0.089_484_18 * self.a - 1.291_485_5 * self.b).powi(3);
        Rgba::from_linear(Rgba {
            r: 4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s,
            g: -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
            b: -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
            a: 1.0,
        })
    }

    fn chroma(self) -> f32 {
        self.a.hypot(self.b)
    }

    fn distance(self, other: Self) -> f32 {
        ((self.l - other.l).powi(2) + (self.a - other.a).powi(2) + (self.b - other.b).powi(2))
            .sqrt()
    }

    fn shaped(self, [low, high]: [f32; 2], max_chroma: f32) -> Self {
        let chroma = self.chroma();
        let scale = if chroma > max_chroma {
            max_chroma / chroma
        } else {
            1.0
        };
        Self {
            l: self.l.clamp(low, high),
            a: self.a * scale,
            b: self.b * scale,
        }
    }
}

// macOS desktop tinting: the gray keeps its lightness and takes part of the wallpaper's hue (ADR 0065).
pub fn take_chroma(base: Rgba, source: Rgba, amount: f32) -> Rgba {
    let (p, q) = (Oklab::from_rgba(base), Oklab::from_rgba(source));
    let t = amount.clamp(0.0, 1.0);
    let mut tinted = Oklab {
        l: p.l,
        a: p.a + (q.a - p.a) * t,
        b: p.b + (q.b - p.b) * t,
    }
    .to_rgba();
    tinted.a = base.a;
    tinted
}

fn mix_oklab(from: Rgba, to: Rgba, amount: f32) -> Rgba {
    let (p, q) = (Oklab::from_rgba(from), Oklab::from_rgba(to));
    let t = amount.clamp(0.0, 1.0);
    let mut mixed = Oklab {
        l: p.l + (q.l - p.l) * t,
        a: p.a + (q.a - p.a) * t,
        b: p.b + (q.b - p.b) * t,
    }
    .to_rgba();
    mixed.a = from.a + (to.a - from.a) * t;
    mixed
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImagePalette {
    pub colors: [Rgba; 3],
    pub vivid: Rgba,
}

struct Cluster {
    centre: Oklab,
    share: f32,
}

fn clusters(rgba8: &[u8]) -> Vec<Cluster> {
    let pixels: Vec<Oklab> = rgba8
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|[_, _, _, alpha]| *alpha >= 128)
        .map(|[r, g, b, _]| Oklab::from_rgba(Rgba::rgb8(*r, *g, *b)))
        .collect();
    if pixels.is_empty() {
        return Vec::new();
    }
    let k = BACKDROP.clusters as usize;
    let mut centres: Vec<Oklab> = (0..k)
        .map(|i| pixels[(2 * i + 1) * pixels.len() / (2 * k)])
        .collect();
    let mut owner = vec![0usize; pixels.len()];
    for _ in 0..14 {
        let mut sums = vec![(0.0f32, 0.0f32, 0.0f32, 0usize); k];
        for (pixel, slot) in pixels.iter().zip(owner.iter_mut()) {
            *slot = nearest(&centres, *pixel);
            let sum = &mut sums[*slot];
            sum.0 += pixel.l;
            sum.1 += pixel.a;
            sum.2 += pixel.b;
            sum.3 += 1;
        }
        for (centre, (l, a, b, n)) in centres.iter_mut().zip(sums) {
            if n > 0 {
                let n = n as f32;
                *centre = Oklab {
                    l: l / n,
                    a: a / n,
                    b: b / n,
                };
            }
        }
    }
    let mut clusters: Vec<Cluster> = centres
        .into_iter()
        .enumerate()
        .map(|(index, centre)| Cluster {
            centre,
            share: owner.iter().filter(|o| **o == index).count() as f32 / pixels.len() as f32,
        })
        .filter(|cluster| cluster.share > 0.0)
        .collect();
    clusters.sort_by(|p, q| q.share.total_cmp(&p.share));
    clusters
}

pub fn palette_from_pixels(rgba8: &[u8]) -> Option<ImagePalette> {
    let clusters = clusters(rgba8);
    if clusters.is_empty() {
        return None;
    }
    let mid_tones: Vec<&Cluster> = clusters
        .iter()
        .filter(|c| c.centre.l > 0.12 && c.centre.l < 0.95)
        .collect();
    let pool: Vec<&Cluster> = if mid_tones.len() >= 3 {
        mid_tones
    } else {
        clusters.iter().collect()
    };
    let mut chosen = vec![pool[0]];
    while chosen.len() < 3 && chosen.len() < pool.len() {
        let next = pool
            .iter()
            .filter(|c| !chosen.iter().any(|d| std::ptr::eq(**c, *d)))
            .max_by(|p, q| spread(p, &chosen).total_cmp(&spread(q, &chosen)))
            .copied()?;
        chosen.push(next);
    }
    let vivid = pool
        .iter()
        .take(5)
        .filter(|c| c.share > 0.03)
        .max_by(|p, q| p.centre.chroma().total_cmp(&q.centre.chroma()))
        .unwrap_or(&pool[0]);
    let pick = |i: usize| chosen[i.min(chosen.len() - 1)].centre.to_rgba();
    Some(ImagePalette {
        colors: [pick(0), pick(1), pick(2)],
        vivid: vivid.centre.to_rgba(),
    })
}

pub fn dominant_from_pixels(rgba8: &[u8]) -> Option<Rgba> {
    let weight = |cluster: &Cluster| cluster.share * cluster.centre.chroma();
    let dominant = clusters(rgba8)
        .into_iter()
        .filter(|cluster| cluster.centre.chroma() >= DOMINANT_MIN_CHROMA)
        .max_by(|p, q| weight(p).total_cmp(&weight(q)))?;
    let (hue, saturation, lightness) = dominant.centre.to_rgba().to_hsl();
    let [low, high] = DOMINANT_LIGHTNESS;
    Some(Rgba::from_hsl(
        hue,
        saturation,
        lightness.clamp(low, high),
        1.0,
    ))
}

fn nearest(centres: &[Oklab], pixel: Oklab) -> usize {
    centres
        .iter()
        .enumerate()
        .min_by(|(_, p), (_, q)| p.distance(pixel).total_cmp(&q.distance(pixel)))
        .map_or(0, |(index, _)| index)
}

fn spread(cluster: &Cluster, chosen: &[&Cluster]) -> f32 {
    chosen
        .iter()
        .map(|c| c.centre.distance(cluster.centre))
        .fold(f32::MAX, f32::min)
        * cluster.share.sqrt()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Backdrop {
    pub layers: [Rgba; 3],
    pub base: Rgba,
}

impl Backdrop {
    pub fn from_colors(colors: [Rgba; 3]) -> Self {
        let layers =
            colors.map(|c| Oklab::from_rgba(c).shaped(BACKDROP.lightness, BACKDROP.chroma));
        let mean = Oklab {
            l: layers.iter().map(|c| c.l).sum::<f32>() / 3.0 - BACKDROP.base_darken,
            a: layers.iter().map(|c| c.a).sum::<f32>() / 3.0,
            b: layers.iter().map(|c| c.b).sum::<f32>() / 3.0,
        };
        Self {
            layers: layers.map(Oklab::to_rgba),
            base: mean
                .shaped(BACKDROP.base_lightness, BACKDROP.base_chroma)
                .to_rgba(),
        }
    }

    pub fn solid(color: Rgba) -> Self {
        Self {
            layers: [color; 3],
            base: color,
        }
    }

    pub fn mix(self, other: Self, amount: f32) -> Self {
        Self {
            layers: [0, 1, 2].map(|i| mix_oklab(self.layers[i], other.layers[i], amount)),
            base: mix_oklab(self.base, other.base, amount),
        }
    }

    pub fn average(self) -> Rgba {
        let layers = self.layers.map(Oklab::from_rgba);
        let base = Oklab::from_rgba(self.base);
        let weight = BACKDROP.opacity / 2.0;
        let channel = |f: fn(&Oklab) -> f32| {
            let layer = layers.iter().map(f).sum::<f32>() / 3.0;
            layer * weight + f(&base) * (1.0 - weight)
        };
        Oklab {
            l: channel(|c| c.l),
            a: channel(|c| c.a),
            b: channel(|c| c.b),
        }
        .to_rgba()
    }

    pub fn ink(self) -> Rgba {
        ink_on(self.average())
    }

    pub fn accent(self, vivid: Rgba) -> Rgba {
        self.highlight(vivid)
    }

    pub fn highlights(self) -> [Rgba; 3] {
        self.layers.map(|layer| self.highlight(layer))
    }

    fn highlight(self, color: Rgba) -> Rgba {
        let color = Oklab::from_rgba(color);
        let chroma = color.chroma().max(BACKDROP.accent_min_chroma);
        let hue = color.b.atan2(color.a);
        let l = if self.ink() == LIGHT.on_accent {
            BACKDROP.accent_lightness_on_dark
        } else {
            BACKDROP.accent_lightness_on_light
        };
        Oklab {
            l,
            a: chroma * hue.cos(),
            b: chroma * hue.sin(),
        }
        .to_rgba()
    }
}

pub fn ink_on(background: Rgba) -> Rgba {
    let (light, dark) = (LIGHT.on_accent, LIGHT.ink_strong);
    if light.contrast_ratio(background) >= dark.contrast_ratio(background) {
        light
    } else {
        dark
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(colors: &[(Rgba, usize)]) -> Vec<u8> {
        colors
            .iter()
            .flat_map(|(c, n)| std::iter::repeat_n(c.to_rgba8(), *n))
            .flatten()
            .collect()
    }

    fn close(p: Rgba, q: Rgba) -> bool {
        Oklab::from_rgba(p).distance(Oklab::from_rgba(q)) < 0.02
    }

    #[test]
    fn oklab_round_trips_srgb() {
        for color in [
            Rgba::rgb8(12, 200, 90),
            Rgba::rgb8(250, 250, 250),
            Rgba::rgb8(90, 20, 140),
        ] {
            let back = Oklab::from_rgba(color).to_rgba().to_rgba8();
            for (p, q) in back.iter().zip(color.to_rgba8()) {
                assert!(p.abs_diff(q) <= 1, "{color:?} became {back:?}");
            }
        }
    }

    #[test]
    fn the_palette_finds_the_colours_of_a_three_colour_image() {
        let (red, teal, gold) = (
            Rgba::rgb8(200, 50, 60),
            Rgba::rgb8(30, 130, 140),
            Rgba::rgb8(220, 170, 40),
        );
        let palette = palette_from_pixels(&image(&[(red, 600), (teal, 300), (gold, 100)])).unwrap();
        assert!(close(palette.colors[0], red));
        for wanted in [teal, gold] {
            assert!(
                palette.colors.iter().any(|c| close(*c, wanted)),
                "missing {wanted:?}"
            );
        }
    }

    #[test]
    fn the_same_image_always_gives_the_same_palette() {
        let pixels = image(&[
            (Rgba::rgb8(10, 90, 200), 500),
            (Rgba::rgb8(240, 120, 30), 500),
        ]);
        assert_eq!(palette_from_pixels(&pixels), palette_from_pixels(&pixels));
    }

    #[test]
    fn a_grey_image_has_no_dominant_colour() {
        assert_eq!(
            dominant_from_pixels(&image(&[(Rgba::rgb8(128, 128, 128), 256)])),
            None
        );
    }

    #[test]
    fn a_wide_colour_beats_a_more_saturated_small_one() {
        let (green, grey, red) = (
            Rgba::rgb8(5, 125, 5),
            Rgba::rgb8(128, 128, 128),
            Rgba::rgb8(255, 0, 0),
        );
        let dominant =
            dominant_from_pixels(&image(&[(green, 600), (grey, 150), (red, 250)])).unwrap();
        assert!(
            dominant.g > dominant.r && dominant.g > dominant.b,
            "{dominant:?}"
        );
    }

    #[test]
    fn a_dark_dominant_is_lifted_into_the_lightness_band() {
        let dominant = dominant_from_pixels(&image(&[(Rgba::rgb8(77, 26, 0), 256)])).unwrap();
        let (_, _, lightness) = dominant.to_hsl();
        assert!(
            (DOMINANT_LIGHTNESS[0]..=DOMINANT_LIGHTNESS[1]).contains(&lightness),
            "{lightness}"
        );
    }

    #[test]
    fn backdrop_layers_stay_inside_the_readable_range() {
        let neon = [
            Rgba::rgb8(0, 255, 0),
            Rgba::rgb8(255, 255, 255),
            Rgba::rgb8(0, 0, 0),
        ];
        for layer in Backdrop::from_colors(neon).layers {
            let lab = Oklab::from_rgba(layer);
            assert!(lab.l >= BACKDROP.lightness[0] - 0.01 && lab.l <= BACKDROP.lightness[1] + 0.01);
            assert!(lab.chroma() <= BACKDROP.chroma + 0.01);
        }
    }

    #[test]
    fn the_ink_is_the_one_with_more_contrast() {
        let dark = Backdrop::solid(Rgba::rgb8(40, 30, 50));
        let light = Backdrop::solid(Rgba::rgb8(235, 230, 220));
        assert_eq!(dark.ink(), LIGHT.on_accent);
        assert_eq!(light.ink(), LIGHT.ink_strong);
    }

    #[test]
    fn a_backdrop_mix_starts_and_ends_on_its_endpoints() {
        let from = Backdrop::solid(Rgba::rgb8(200, 40, 40));
        let to = Backdrop::solid(Rgba::rgb8(40, 40, 200));
        assert!(close(from.mix(to, 0.0).base, from.base));
        assert!(close(from.mix(to, 1.0).base, to.base));
    }
}

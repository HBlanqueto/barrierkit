use ctdesign::{
    BACKDROP, DOMINANT_LIGHTNESS, DOMINANT_MIN_CHROMA, LIGHT, Rgba, WALLPAPER_TINTING_SIDEBAR,
    WALLPAPER_TINTING_SIDEBAR_LIFT, WALLPAPER_TINTING_TARGET,
};

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
        Rgba::from_linear(self.to_linear())
    }

    /// Linear sRGB, not clamped: a channel outside 0..1 means the color is outside the gamut.
    pub fn to_linear(self) -> Rgba {
        let l = (self.l + 0.396_337_78 * self.a + 0.215_803_76 * self.b).powi(3);
        let m = (self.l - 0.105_561_346 * self.a - 0.063_854_17 * self.b).powi(3);
        let s = (self.l - 0.089_484_18 * self.a - 1.291_485_5 * self.b).powi(3);
        Rgba {
            r: 4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s,
            g: -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
            b: -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
            a: 1.0,
        }
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

// The gray keeps its lightness and takes part of the source's hue: the accent tint, and the hue of the wallpaper tint.
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

// The glass saturates smoothly towards `cap`, so a vivid wallpaper does not turn the panel into a window (ADR 0078).
pub fn take_chroma_capped(base: Rgba, source: Rgba, amount: f32, cap: f32) -> Rgba {
    let (p, q) = (Oklab::from_rgba(base), Oklab::from_rgba(source));
    let t = amount.clamp(0.0, 1.0);
    let (a, b) = (p.a + (q.a - p.a) * t, p.b + (q.b - p.b) * t);
    let soft = 1.0 / (1.0 + (a.hypot(b) / cap).powi(2)).sqrt();
    let mut tinted = Oklab {
        l: p.l,
        a: a * soft,
        b: b * soft,
    }
    .to_rgba();
    tinted.a = base.a;
    tinted
}

// macOS 26 also lifts a dark window towards this lightness as it takes the wallpaper's hue (ADR 0065).
pub fn lift(color: Rgba, amount: f32) -> Rgba {
    let mut lab = Oklab::from_rgba(color);
    lab.l += (WALLPAPER_TINTING_TARGET - lab.l) * amount.clamp(0.0, 1.0);
    let mut lifted = lab.to_rgba();
    lifted.a = color.a;
    lifted
}

// The layers over a macOS window are translucent and show its tint as it is: an opaque layer takes the same OKLab shift, a translucent one already shows it.
pub(crate) fn shift_like(color: Rgba, from: Rgba, to: Rgba) -> Rgba {
    if color.a < 1.0 {
        return color;
    }
    let (c, p, q) = (
        Oklab::from_rgba(color),
        Oklab::from_rgba(from),
        Oklab::from_rgba(to),
    );
    let mut shifted = Oklab {
        l: c.l + q.l - p.l,
        a: c.a + q.a - p.a,
        b: c.b + q.b - p.b,
    }
    .to_rgba();
    shifted.a = color.a;
    shifted
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WallpaperTinting {
    pub chroma: f32,
    pub lift: f32,
}

impl WallpaperTinting {
    pub fn sidebar(self) -> Self {
        Self {
            chroma: self.chroma * WALLPAPER_TINTING_SIDEBAR,
            lift: self.lift * WALLPAPER_TINTING_SIDEBAR_LIFT,
        }
    }

    pub fn take(self, gray: Rgba, wallpaper: Rgba) -> Rgba {
        lift(take_chroma(gray, wallpaper, self.chroma), self.lift)
    }
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

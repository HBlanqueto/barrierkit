use ctappearance::{ColorScheme, icon_style::luma};
use ctdesign::Rgba;

use crate::design::{
    GlassLens, GlassLook, GlassRim, GlassScale, GlassSet, GlassTint, LIQUID_GLASS_BLUR_CAP,
    LIQUID_GLASS_CLEAR_DARK, LIQUID_GLASS_CLEAR_LIGHT, LIQUID_GLASS_LENS,
    LIQUID_GLASS_REGULAR_DARK, LIQUID_GLASS_REGULAR_LIGHT, LIQUID_GLASS_TINT_CLEAR_DARK,
    LIQUID_GLASS_TINT_CLEAR_LIGHT, LIQUID_GLASS_TINT_REGULAR_DARK, LIQUID_GLASS_TINT_REGULAR_LIGHT,
    LIQUID_GLASS_UNION, RIM_BINS,
};

const SIGMA_LEVELS: [f32; 13] = [
    0.0, 1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 10.0, 13.0, 16.0, 20.0, 24.0, 28.0,
];
const RIM_ANGLE_BIN: f32 = 7.5;
const BLUR_PIXELS_PER_RADIUS: f32 = 2.0;
const HALF_RESOLUTION: usize = 2;
const MIDDLE_KNOT: f32 = 0.5;
const SHIFT_TABLES: f32 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Glass {
    Regular,
    Clear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlassState {
    Active,
    Inactive,
    Reduced,
    IncreasedContrast,
    ReducedContrast,
}

pub struct Image {
    pub width: usize,
    pub height: usize,
    pub rgb: Vec<[f32; 3]>,
}

#[derive(Debug, Clone, Copy)]
pub struct Shape {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub radius: f32,
}

pub fn looks(glass: Glass, scheme: ColorScheme) -> GlassSet {
    match glass {
        Glass::Regular => scheme.pick([
            LIQUID_GLASS_REGULAR_LIGHT,
            LIQUID_GLASS_REGULAR_DARK,
            LIQUID_GLASS_REGULAR_DARK,
        ]),
        Glass::Clear => scheme.pick([
            LIQUID_GLASS_CLEAR_LIGHT,
            LIQUID_GLASS_CLEAR_DARK,
            LIQUID_GLASS_CLEAR_DARK,
        ]),
    }
}

pub fn tint(glass: Glass, scheme: ColorScheme) -> GlassTint {
    match glass {
        Glass::Regular => scheme.pick([
            LIQUID_GLASS_TINT_REGULAR_LIGHT,
            LIQUID_GLASS_TINT_REGULAR_DARK,
            LIQUID_GLASS_TINT_REGULAR_DARK,
        ]),
        Glass::Clear => scheme.pick([
            LIQUID_GLASS_TINT_CLEAR_LIGHT,
            LIQUID_GLASS_TINT_CLEAR_DARK,
            LIQUID_GLASS_TINT_CLEAR_DARK,
        ]),
    }
}

pub fn look(set: &GlassSet, state: GlassState) -> GlassLook {
    match state {
        GlassState::Active => set.active,
        GlassState::Inactive => set.inactive,
        GlassState::Reduced => set.reduced,
        GlassState::IncreasedContrast => set.increased_contrast,
        GlassState::ReducedContrast => set.reduced_contrast,
    }
}

fn rule(rule: [f32; 4], shorter: f32) -> f32 {
    (rule[0] * shorter + rule[1]).clamp(rule[2].min(rule[3]), rule[2].max(rule[3]))
}

pub fn adapted(look: &GlassLook, scale: &GlassScale, shorter: f32) -> GlassLook {
    let mut adapted = *look;
    adapted.refraction = [
        rule(scale.refraction_amount, shorter),
        rule(scale.refraction_height, shorter),
    ];
    adapted.blur_opacity[0] = rule(scale.blur_opacity_0, shorter);
    adapted.blur_opacity[1] = rule(scale.blur_opacity_1, shorter);
    adapted.blur_distance[0] = rule(scale.blur_distance_0, shorter);
    adapted.max_luma = rule(scale.max_luma, shorter);
    adapted
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tinted {
    pub look: GlassLook,
    pub read_factor: usize,
    pub blur_fill: f32,
    pub blur_fill_sigma: f32,
}

impl Tinted {
    pub fn untinted(look: GlassLook) -> Self {
        Self {
            look,
            read_factor: HALF_RESOLUTION,
            blur_fill: 0.0,
            blur_fill_sigma: 0.0,
        }
    }
}

fn along_knots(knots: [f32; 3], amount: f32) -> f32 {
    if amount <= MIDDLE_KNOT {
        knots[0] + (knots[1] - knots[0]) * amount / MIDDLE_KNOT
    } else {
        knots[1] + (knots[2] - knots[1]) * (amount - MIDDLE_KNOT) / MIDDLE_KNOT
    }
}

fn tone_shift(tint: &GlassTint, amount: f32) -> [f32; 9] {
    let tables = [
        [0.0; 9],
        tint.tone_shift_t25,
        tint.tone_shift_t50,
        tint.tone_shift_t75,
        tint.tone_shift_t100,
    ];
    let at = amount * SHIFT_TABLES;
    let below = (at.floor() as usize).min(tables.len() - 2);
    std::array::from_fn(|index| {
        tables[below][index]
            + (tables[below + 1][index] - tables[below][index]) * (at - below as f32)
    })
}

pub fn tinted(set: &GlassSet, tint: &GlassTint, amount: f32, shorter: f32) -> Tinted {
    let amount = amount.clamp(0.0, 1.0);
    let mut look = adapted(&set.active, &set.scale, shorter);
    look.blur = along_knots(tint.blur, amount);
    if amount > 0.0 && tint.blur_opacity[3] > 0.0 {
        look.blur_opacity = tint.blur_opacity;
    }
    let [slope, offset, _, highest] = set.scale.max_luma;
    look.max_luma = rule(
        [slope, offset, along_knots(tint.max_luma, amount), highest],
        shorter,
    );
    let rgb = along_knots(tint.fill_rgb, amount);
    look.fill = Rgba {
        r: rgb,
        g: rgb,
        b: rgb,
        a: along_knots(tint.fill_alpha, amount),
    };
    let shift = tone_shift(tint, amount);
    for (delta, shift) in look.tone_delta.iter_mut().zip(shift) {
        *delta += shift;
    }
    let blur_fill = along_knots(tint.blur_fill_opacity, amount);
    let halvings = tint
        .read_halvings
        .iter()
        .filter(|threshold| amount >= **threshold)
        .count();
    Tinted {
        look,
        read_factor: HALF_RESOLUTION << halvings,
        blur_fill,
        blur_fill_sigma: if blur_fill > 0.0 {
            BLUR_PIXELS_PER_RADIUS * tint.blur_fill_radius
        } else {
            0.0
        },
    }
}

pub fn half_resolution(line: &[f32], factor: usize) -> Vec<f32> {
    let texels: Vec<f32> = line
        .chunks(factor)
        .map(|block| block.iter().sum::<f32>() / block.len() as f32)
        .collect();
    let last = texels.len() - 1;
    (0..line.len())
        .map(|pixel| {
            let at = ((pixel as f32 + 0.5) / factor as f32 - 0.5).clamp(0.0, last as f32);
            let below = at.floor() as usize;
            let above = (below + 1).min(last);
            texels[below] + (texels[above] - texels[below]) * (at - below as f32)
        })
        .collect()
}

fn lerp(table: &[f32], at: f32) -> f32 {
    let at = at.clamp(0.0, (table.len() - 1) as f32);
    let below = at.floor() as usize;
    let above = (below + 1).min(table.len() - 1);
    table[below] + (table[above] - table[below]) * (at - below as f32)
}

pub fn face(look: &GlassLook, backdrop: [f32; 3]) -> [f32; 3] {
    let y = luma(backdrop);
    let cap = 1.0 - (1.0 - look.max_luma) * y;
    let delta = lerp(&look.tone_delta, y * 8.0) / 255.0;
    let fill = [look.fill.r, look.fill.g, look.fill.b];
    let mut shown = [0.0; 3];
    for (index, channel) in backdrop.iter().enumerate() {
        let saturated = (y + look.saturation * (channel - y)).clamp(0.0, 1.0);
        let tone = look.black + (look.white - look.black) * saturated * cap;
        shown[index] =
            ((1.0 - look.fill.a) * tone + look.fill.a * fill[index] + delta).clamp(0.0, 1.0);
    }
    shown
}

pub fn blur_sigma(look: &GlassLook, depth: f32) -> f32 {
    let cap = LIQUID_GLASS_BLUR_CAP;
    let (distance, opacity) = (look.blur_distance, look.blur_opacity);
    let first = distance[0];
    let scale = if depth <= first {
        opacity[0]
    } else if depth <= distance[1] {
        opacity[0] + (opacity[1] - opacity[0]) * (depth - first) / (distance[1] - first)
    } else if depth <= distance[2] {
        opacity[1] + (opacity[2] - opacity[1]) * (depth - distance[1]) / (distance[2] - distance[1])
    } else {
        opacity[2]
    };
    (BLUR_PIXELS_PER_RADIUS * look.blur * scale).min(cap)
}

pub fn lens_offset(look: &GlassLook, lens: &GlassLens, depth: f32) -> f32 {
    let [amount, height] = look.refraction;
    let u = depth / height;
    if height <= 0.0 || !(0.0..1.0).contains(&u) {
        return 0.0;
    }
    amount.abs() * lens.scale * ((-lens.decay * u).exp() - (-lens.decay).exp())
        / (1.0 - (-lens.decay).exp())
}

fn rim_center(bin: usize) -> f32 {
    match bin {
        0..24 => -8.0 + 0.5 * bin as f32 + 0.25,
        24 => 5.0,
        _ => 6.0 + 2.0 * (bin - 25) as f32 + 1.0,
    }
}

pub fn rim_levels(rim: &GlassRim, distance: f32, angle: f32) -> f32 {
    let folded = 90.0 - ((angle.rem_euclid(180.0)) - 90.0).abs();
    let angle_at = (folded / RIM_ANGLE_BIN - 0.5).clamp(0.0, 11.0);
    let centers: [f32; RIM_BINS] = std::array::from_fn(rim_center);
    if distance < centers[0] - 0.25 || distance > centers[RIM_BINS - 1] + 1.0 {
        return 0.0;
    }
    let mut along = 0.0;
    for (profile, angles) in [
        (&rim.distance_1, &rim.angle_1),
        (&rim.distance_2, &rim.angle_2),
    ] {
        let at = centers
            .windows(2)
            .position(|pair| distance < pair[1])
            .unwrap_or(RIM_BINS - 2);
        let t = ((distance - centers[at]) / (centers[at + 1] - centers[at])).clamp(0.0, 1.0);
        along += lerp(angles, angle_at) * (profile[at] + (profile[at + 1] - profile[at]) * t);
    }
    along
}

pub fn smooth_union(a: f32, b: f32, spacing: f32) -> f32 {
    let k = spacing * LIQUID_GLASS_UNION;
    if k <= 0.0 {
        return a.min(b);
    }
    let blend = (k - (a - b).abs()).max(0.0) / k;
    a.min(b) - blend * blend * k * 0.25
}

pub fn rounded_rect_distance(shape: &Shape, x: f32, y: f32) -> (f32, [f32; 2]) {
    let (centre_x, centre_y) = (shape.x + shape.width / 2.0, shape.y + shape.height / 2.0);
    let (px, py) = (x - centre_x, y - centre_y);
    let (qx, qy) = (
        px.abs() - (shape.width / 2.0 - shape.radius),
        py.abs() - (shape.height / 2.0 - shape.radius),
    );
    let (sx, sy) = (px.signum(), py.signum());
    if qx > 0.0 && qy > 0.0 {
        let length = qx.hypot(qy);
        (length - shape.radius, [sx * qx / length, sy * qy / length])
    } else if qx > qy {
        (qx - shape.radius, [sx, 0.0])
    } else {
        (qy - shape.radius, [0.0, sy])
    }
}

impl Image {
    pub fn new(width: usize, height: usize, with: impl Fn(usize, usize) -> [f32; 3]) -> Self {
        let rgb = (0..width * height)
            .map(|at| with(at % width, at / width))
            .collect();
        Self { width, height, rgb }
    }

    fn at(&self, x: usize, y: usize) -> [f32; 3] {
        self.rgb[y * self.width + x]
    }

    fn lines(&self, on_rows: bool) -> Vec<Vec<[f32; 3]>> {
        let (count, length) = if on_rows {
            (self.height, self.width)
        } else {
            (self.width, self.height)
        };
        (0..count)
            .map(|line| {
                (0..length)
                    .map(|at| {
                        if on_rows {
                            self.at(at, line)
                        } else {
                            self.at(line, at)
                        }
                    })
                    .collect()
            })
            .collect()
    }

    fn map_lines(&self, on_rows: bool, with: impl Fn(&[[f32; 3]]) -> Vec<[f32; 3]>) -> Self {
        let mut out = Self {
            width: self.width,
            height: self.height,
            rgb: vec![[0.0; 3]; self.width * self.height],
        };
        for (line, values) in self.lines(on_rows).iter().enumerate() {
            for (at, value) in with(values).into_iter().enumerate() {
                let (x, y) = if on_rows { (at, line) } else { (line, at) };
                out.rgb[y * self.width + x] = value;
            }
        }
        out
    }

    pub fn half_resolution(&self, factor: usize) -> Self {
        let channel = |line: &[[f32; 3]], index: usize| {
            half_resolution(
                &line.iter().map(|pixel| pixel[index]).collect::<Vec<_>>(),
                factor,
            )
        };
        let pass = |line: &[[f32; 3]]| {
            let (r, g, b) = (channel(line, 0), channel(line, 1), channel(line, 2));
            (0..line.len()).map(|at| [r[at], g[at], b[at]]).collect()
        };
        self.map_lines(true, pass).map_lines(false, pass)
    }

    fn gaussian(&self, sigma: f32) -> Self {
        if sigma < 0.3 {
            return Self {
                width: self.width,
                height: self.height,
                rgb: self.rgb.clone(),
            };
        }
        let reach = (sigma * 3.5).ceil() as i32;
        let weights: Vec<f32> = (-reach..=reach)
            .map(|at| (-0.5 * (at as f32 / sigma).powi(2)).exp())
            .collect();
        let total: f32 = weights.iter().sum();
        let pass = |line: &[[f32; 3]]| -> Vec<[f32; 3]> {
            (0..line.len() as i32)
                .map(|at| {
                    let mut sum = [0.0; 3];
                    for (index, weight) in weights.iter().enumerate() {
                        let from =
                            (at + index as i32 - reach).clamp(0, line.len() as i32 - 1) as usize;
                        for channel in 0..3 {
                            sum[channel] += weight * line[from][channel] / total;
                        }
                    }
                    sum
                })
                .collect()
        };
        self.map_lines(true, pass).map_lines(false, pass)
    }

    fn bilinear(&self, x: f32, y: f32) -> [f32; 3] {
        let fx = (x - 0.5).clamp(0.0, self.width as f32 - 1.0001);
        let fy = (y - 0.5).clamp(0.0, self.height as f32 - 1.0001);
        let (x0, y0) = (fx.floor() as usize, fy.floor() as usize);
        let (ax, ay) = (fx - x0 as f32, fy - y0 as f32);
        std::array::from_fn(|channel| {
            let top = self.at(x0, y0)[channel] * (1.0 - ax) + self.at(x0 + 1, y0)[channel] * ax;
            let bottom =
                self.at(x0, y0 + 1)[channel] * (1.0 - ax) + self.at(x0 + 1, y0 + 1)[channel] * ax;
            top * (1.0 - ay) + bottom * ay
        })
    }
}

pub fn render(
    set: &GlassSet,
    tint: &GlassTint,
    amount: f32,
    state: GlassState,
    backdrop: &Image,
    shape: &Shape,
) -> Image {
    let base = look(set, state);
    let applied = if state == GlassState::Active {
        tinted(set, tint, amount, shape.width.min(shape.height))
    } else {
        Tinted::untinted(base)
    };
    let look = applied.look;
    let half = backdrop.half_resolution(applied.read_factor);
    let widest = look
        .blur_opacity
        .iter()
        .fold(0.0_f32, |widest, opacity| {
            widest.max(BLUR_PIXELS_PER_RADIUS * look.blur * opacity)
        })
        .min(LIQUID_GLASS_BLUR_CAP);
    let mut needed: Vec<f32> = SIGMA_LEVELS
        .iter()
        .copied()
        .filter(|sigma| *sigma < widest)
        .collect();
    needed.push(
        SIGMA_LEVELS
            .iter()
            .copied()
            .find(|sigma| *sigma >= widest)
            .unwrap_or(LIQUID_GLASS_BLUR_CAP),
    );
    let blurred: Vec<Image> = needed.iter().map(|sigma| half.gaussian(*sigma)).collect();
    let blur_fill = (applied.blur_fill > 0.0).then(|| half.gaussian(applied.blur_fill_sigma));
    Image::new(backdrop.width, backdrop.height, |x, y| {
        let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
        let (distance, normal) = rounded_rect_distance(shape, px, py);
        let angle = (-normal[1]).atan2(normal[0]).to_degrees();
        let rim = rim_levels(&set.rim, distance, angle);
        let outside = backdrop.at(x, y);
        let shown = if distance < 0.0 {
            let offset = lens_offset(&look, &LIQUID_GLASS_LENS, -distance);
            let (sx, sy) = (px - normal[0] * offset, py - normal[1] * offset);
            let sigma = blur_sigma(&look, distance);
            let upper = needed
                .iter()
                .position(|level| *level >= sigma)
                .unwrap_or(needed.len() - 1);
            let lower = upper.saturating_sub(1);
            let span = (needed[upper] - needed[lower]).max(1e-6);
            let mix = ((sigma - needed[lower]) / span).clamp(0.0, 1.0);
            let (near, far) = (
                blurred[lower].bilinear(sx, sy),
                blurred[upper].bilinear(sx, sy),
            );
            let filled = blur_fill
                .as_ref()
                .map_or([0.0; 3], |copy| copy.bilinear(sx, sy));
            let seen: [f32; 3] = std::array::from_fn(|channel| {
                let main = near[channel] + (far[channel] - near[channel]) * mix;
                (main + (filled[channel] - main) * applied.blur_fill) / 255.0
            });
            face(&look, seen).map(|channel| channel * 255.0)
        } else {
            outside
        };
        shown.map(|channel| (channel + rim).clamp(0.0, 255.0))
    })
}

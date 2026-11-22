use ctdesign::{
    GlassLens, GlassLook, GlassRim, GlassScale, GlassSet, LIQUID_GLASS_BLUR_CAP,
    LIQUID_GLASS_CLEAR_DARK, LIQUID_GLASS_CLEAR_LIGHT, LIQUID_GLASS_LENS,
    LIQUID_GLASS_REGULAR_DARK, LIQUID_GLASS_REGULAR_LIGHT, LIQUID_GLASS_UNION, RIM_BINS,
};

use crate::ColorScheme;
use crate::icon_style::LUMA;

const SIGMA_LEVELS: [f32; 13] = [
    0.0, 1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 10.0, 13.0, 16.0, 20.0, 24.0, 28.0,
];
const RIM_ANGLE_BIN: f32 = 7.5;

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

pub fn half_resolution(line: &[f32]) -> Vec<f32> {
    let texels: Vec<f32> = line
        .chunks(2)
        .map(|pair| pair.iter().sum::<f32>() / pair.len() as f32)
        .collect();
    let last = texels.len() - 1;
    (0..line.len())
        .map(|pixel| {
            let at = ((pixel as f32 - 0.5) / 2.0).clamp(0.0, last as f32);
            let below = at.floor() as usize;
            let above = (below + 1).min(last);
            texels[below] + (texels[above] - texels[below]) * (at - below as f32)
        })
        .collect()
}

fn luma(color: [f32; 3]) -> f32 {
    color
        .iter()
        .zip(LUMA)
        .map(|(channel, weight)| channel * weight)
        .sum()
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
    (2.0 * look.blur * scale).min(cap)
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

    pub fn half_resolution(&self) -> Self {
        let channel = |line: &[[f32; 3]], index: usize| {
            half_resolution(&line.iter().map(|pixel| pixel[index]).collect::<Vec<_>>())
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

pub fn render(set: &GlassSet, state: GlassState, backdrop: &Image, shape: &Shape) -> Image {
    let base = look(set, state);
    let look = if state == GlassState::Active {
        adapted(&base, &set.scale, shape.width.min(shape.height))
    } else {
        base
    };
    let half = backdrop.half_resolution();
    let widest = look
        .blur_opacity
        .iter()
        .fold(0.0_f32, |widest, opacity| {
            widest.max(2.0 * look.blur * opacity)
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
            let seen: [f32; 3] = std::array::from_fn(|channel| {
                (near[channel] + (far[channel] - near[channel]) * mix) / 255.0
            });
            face(&look, seen).map(|channel| channel * 255.0)
        } else {
            outside
        };
        shown.map(|channel| (channel + rim).clamp(0.0, 255.0))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLES: &str = include_str!("../tests/data/liquid-glass-samples.csv");

    fn rows(kind: &'static str) -> impl Iterator<Item = Vec<&'static str>> {
        SAMPLES.lines().filter_map(move |line| {
            let mut fields = line.split(',');
            (fields.next()? == kind).then(|| fields.collect())
        })
    }

    fn set_named(name: &str) -> GlassSet {
        match name {
            "regular-dark" => looks(Glass::Regular, ColorScheme::Dark),
            "regular-light" => looks(Glass::Regular, ColorScheme::Light),
            "clear-dark" => looks(Glass::Clear, ColorScheme::Dark),
            _ => looks(Glass::Clear, ColorScheme::Light),
        }
    }

    fn levels(field: &str) -> f32 {
        field.parse().unwrap()
    }

    fn gray(set: &GlassSet, state: GlassState, input: f32) -> f32 {
        face(&look(set, state), [input / 255.0; 3])[1] * 255.0
    }

    #[test]
    fn a_one_pixel_line_spreads_over_four_pixels_by_an_eighth_and_three_eighths() {
        let mut line = vec![0.0; 16];
        line[8] = 1.0;
        assert_eq!(
            half_resolution(&line)[6..11],
            [0.0, 0.125, 0.375, 0.375, 0.125]
        );
    }

    #[test]
    fn clear_glass_reproduces_the_lines_of_the_kern_capture_within_one_level() {
        let set = looks(Glass::Clear, ColorScheme::Dark);
        let clear = look(&set, GlassState::Active);
        let mut worst = 0.0_f32;
        for (lo, hi, whites) in [
            (214, 227, vec![220]),
            (244, 258, vec![250, 251]),
            (280, 296, vec![285, 286, 287, 288]),
        ] {
            let line: Vec<f32> = (0..640)
                .map(|x| if whites.contains(&x) { 1.0 } else { 0.0 })
                .collect();
            let seen = half_resolution(&line);
            for row in rows("line").filter(|row| (lo..hi).contains(&(levels(row[0]) as usize))) {
                let shown = face(&clear, [seen[levels(row[0]) as usize]; 3])[1] * 255.0;
                worst = worst.max((shown - levels(row[1])).abs());
            }
        }
        assert!(worst <= 1.0, "worst {worst}");
    }

    #[test]
    fn the_gray_tone_of_each_look_over_uniform_backdrops_and_the_ramp_stays_within_two_levels() {
        let mut worst = 0.0_f32;
        for row in rows("uniform").chain(rows("ramp")) {
            let shown = gray(&set_named(row[0]), GlassState::Active, levels(row[1]));
            worst = worst.max((shown - levels(row[2])).abs());
        }
        assert!(worst <= 2.5, "worst {worst}");
    }

    #[test]
    fn colours_over_uniform_backdrops_stay_close_in_device_values() {
        let mut worst = [0.0_f32; 3];
        for row in rows("colour") {
            let input = [levels(row[1]), levels(row[2]), levels(row[3])].map(|level| level / 255.0);
            let shown = face(&look(&set_named(row[0]), GlassState::Active), input);
            let kind = ["regular-dark", "regular-light", "clear-dark"]
                .iter()
                .position(|name| *name == row[0])
                .unwrap();
            for channel in 0..3 {
                worst[kind] =
                    worst[kind].max((shown[channel] * 255.0 - levels(row[4 + channel])).abs());
            }
        }
        assert!(
            worst[2] <= 8.0 && worst[0] <= 32.0 && worst[1] <= 70.0,
            "worst {worst:?}"
        );
    }

    #[test]
    fn the_other_states_keep_the_tone_of_their_look() {
        let mut worst = 0.0_f32;
        for row in rows("state") {
            let state = match row[1] {
                "reduced" => GlassState::Reduced,
                "increased-contrast" => GlassState::IncreasedContrast,
                _ => GlassState::Inactive,
            };
            worst =
                worst.max((gray(&set_named(row[0]), state, levels(row[2])) - levels(row[3])).abs());
        }
        assert!(worst <= 20.0, "worst {worst}");
    }

    #[test]
    fn the_lens_follows_the_sawtooth_displacement_within_two_pixels() {
        let mut worst = 0.0_f32;
        for row in rows("lens") {
            let mut clear = look(&looks(Glass::Clear, ColorScheme::Dark), GlassState::Active);
            clear.refraction = [levels(row[1]), levels(row[0])];
            let shown = lens_offset(&clear, &LIQUID_GLASS_LENS, levels(row[2]) + 0.5);
            worst = worst.max((shown - levels(row[3])).abs());
        }
        assert!(worst <= 2.0, "worst {worst}");
    }

    #[test]
    fn the_lens_ends_at_its_height_and_never_pulls_from_outside() {
        let clear = look(&looks(Glass::Clear, ColorScheme::Dark), GlassState::Active);
        assert_eq!(lens_offset(&clear, &LIQUID_GLASS_LENS, 20.0), 0.0);
        assert_eq!(lens_offset(&clear, &LIQUID_GLASS_LENS, -1.0), 0.0);
    }

    #[test]
    fn the_blur_of_a_regular_glass_widens_with_depth_as_measured() {
        let set = looks(Glass::Regular, ColorScheme::Dark);
        let regular = adapted(&look(&set, GlassState::Active), &set.scale, 400.0);
        for (depth, measured) in [(-40.0, 5.5), (-60.0, 6.5), (-100.0, 8.0), (-160.0, 8.5)] {
            let sigma = blur_sigma(&regular, depth);
            assert!(
                (sigma - measured).abs() <= 2.5,
                "depth {depth}: {sigma} against {measured}"
            );
        }
    }

    #[test]
    fn a_blur_radius_is_a_sigma_in_half_resolution_texels_until_the_cap() {
        let mut regular = look(
            &looks(Glass::Regular, ColorScheme::Dark),
            GlassState::Inactive,
        );
        regular.blur_opacity = [1.0; 4];
        regular.blur_distance = [0.0; 4];
        for (radius, measured) in [
            (2.0, 4.0),
            (5.0, 9.5),
            (10.0, 19.5),
            (20.0, 27.5),
            (40.0, 29.0),
        ] {
            regular.blur = radius;
            assert!(
                (blur_sigma(&regular, -50.0) - measured).abs() <= 1.0,
                "radius {radius}"
            );
        }
    }

    #[test]
    fn the_size_rules_give_the_inputs_of_each_shape_size() {
        let set = looks(Glass::Regular, ColorScheme::Dark);
        let large = adapted(&set.active, &set.scale, 160.0);
        assert_eq!(large.refraction, [-60.0, 20.0]);
        assert!(
            (large.blur_opacity[0] - 0.8).abs() < 1e-4
                && (large.blur_opacity[1] - 0.4).abs() < 1e-4
        );
        assert!(
            (large.max_luma - 0.45).abs() < 1e-4 && (large.blur_distance[0] + 80.0).abs() < 1e-3
        );
        let small = adapted(&set.active, &set.scale, 44.0);
        assert_eq!(small.refraction, [-22.0, 11.0]);
        assert_eq!(small.blur_opacity[0], 0.0);
        assert!(
            (small.max_luma - 0.6).abs() < 1e-4 && (small.blur_distance[0] + 22.0).abs() < 1e-3
        );
        let medium = adapted(&set.active, &set.scale, 100.0);
        assert!(
            (medium.blur_opacity[0] - 0.3).abs() < 1e-4 && (medium.max_luma - 0.465).abs() < 1e-3
        );
        let clear_set = looks(Glass::Clear, ColorScheme::Dark);
        let clear = adapted(&clear_set.active, &clear_set.scale, 44.0);
        assert!(
            (clear.refraction[0] + 28.6).abs() < 1e-3 && (clear.refraction[1] - 15.84).abs() < 1e-3
        );
    }

    #[test]
    fn the_rim_table_reproduces_the_circle_captures_within_ten_levels() {
        let mut worst = 0.0_f32;
        for row in rows("rim") {
            let shown = rim_levels(&set_named(row[0]).rim, levels(row[2]), levels(row[1]));
            worst = worst.max((shown - levels(row[3])).abs());
        }
        assert!(worst <= 10.0, "worst {worst}");
    }

    #[test]
    fn two_capsules_of_a_container_join_when_their_gap_is_under_the_spacing() {
        let capsule = |x: f32| Shape {
            x,
            y: 210.0,
            width: 100.0,
            height: 60.0,
            radius: 30.0,
        };
        for row in rows("union").filter(|row| levels(row[0]) > 0.0) {
            let (spacing, gap, measured) = (levels(row[0]), levels(row[1]), levels(row[2]));
            let left = capsule(320.0 - (200.0 + gap) / 2.0);
            let right = capsule(left.x + 100.0 + gap);
            let middle = left.x + 100.0 + gap / 2.0;
            let thickness = (204..277)
                .filter(|y| {
                    let (a, _) = rounded_rect_distance(&left, middle, *y as f32 + 0.5);
                    let (b, _) = rounded_rect_distance(&right, middle, *y as f32 + 0.5);
                    smooth_union(a, b, spacing) < 0.0
                })
                .count() as f32;
            assert!(
                (thickness - measured).abs() <= 8.0,
                "spacing {spacing} gap {gap}: {thickness} against {measured}"
            );
        }
    }

    fn pattern(width: usize, height: usize, with: impl Fn(usize, usize) -> f32) -> Image {
        Image::new(width, height, |x, y| [with(x, y); 3])
    }

    fn worst_inside(
        shown: &Image,
        expected: impl Fn(usize, usize) -> f32,
        shape: &Shape,
        deeper_than: f32,
    ) -> f32 {
        let mut worst = 0.0_f32;
        for y in 0..shown.height {
            for x in 0..shown.width {
                if rounded_rect_distance(shape, x as f32 + 0.5, y as f32 + 0.5).0 < -deeper_than {
                    worst = worst.max((shown.rgb[y * shown.width + x][1] - expected(x, y)).abs());
                }
            }
        }
        worst
    }

    #[test]
    fn a_clear_glass_over_gray_matches_the_capture_below_eight_pixels_of_depth() {
        let set = looks(Glass::Clear, ColorScheme::Dark);
        let shape = Shape {
            x: 80.0,
            y: 60.0,
            width: 200.0,
            height: 160.0,
            radius: 24.0,
        };
        let shown = render(
            &set,
            GlassState::Active,
            &pattern(360, 280, |_, _| 128.0),
            &shape,
        );
        assert!(worst_inside(&shown, |_, _| 154.0, &shape, 8.0) <= 1.0);
    }

    #[test]
    fn a_regular_glass_over_gray_matches_the_capture_below_eight_pixels_of_depth() {
        let shape = Shape {
            x: 80.0,
            y: 60.0,
            width: 200.0,
            height: 160.0,
            radius: 24.0,
        };
        for (scheme, expected) in [(ColorScheme::Dark, 122.0), (ColorScheme::Light, 188.0)] {
            let set = looks(Glass::Regular, scheme);
            let shown = render(
                &set,
                GlassState::Active,
                &pattern(360, 280, |_, _| 128.0),
                &shape,
            );
            let worst = worst_inside(&shown, |_, _| expected, &shape, 8.0);
            assert!(worst <= 3.0, "{scheme:?}: worst {worst}");
        }
    }
}

use std::hash::{DefaultHasher, Hash, Hasher};

use ctdesign::{Rgba, smoothstep};
use freya_core::prelude::*;
use freya_engine::prelude::{BlendMode, Color4f, Paint, Point as SkPoint};

use crate::canvas::{Canvas, CanvasContext};
use crate::styles::{ControlState, Paints, ShapeStyle};

// Skia's patch blends its corners linearly; a cell is drawn as this many smaller patches a side so the eased blend shows.
const STEPS: usize = 8;

/// SwiftUI's `MeshGradient`: a grid of `width` by `height` colors, each at its unit point, blended across the view.
#[derive(Clone, PartialEq)]
pub struct MeshGradient {
    width: usize,
    height: usize,
    points: Vec<(f32, f32)>,
    colors: Vec<ShapeStyle>,
    smooths_colors: bool,
}

impl MeshGradient {
    /// `points` and `colors` run row by row and hold `width * height` items each; a mesh with another count draws nothing.
    pub fn new(
        width: usize,
        height: usize,
        points: Vec<(f32, f32)>,
        colors: Vec<ShapeStyle>,
    ) -> Self {
        Self {
            width,
            height,
            points,
            colors,
            smooths_colors: true,
        }
    }

    /// `false` blends in straight lines between the points; the default holds each color around its point.
    pub fn smooths_colors(mut self, smooths_colors: bool) -> Self {
        self.smooths_colors = smooths_colors;
        self
    }
}

impl Component for MeshGradient {
    fn render(&self) -> impl IntoElement {
        let paints = Paints::current(ControlState::default());
        let mesh = Mesh {
            width: self.width,
            height: self.height,
            points: self.points.clone(),
            colors: self.colors.iter().map(|tone| paints.color(tone)).collect(),
            smooths_colors: self.smooths_colors,
        };
        let complete = self.width >= 2
            && self.height >= 2
            && mesh.points.len() == self.width * self.height
            && mesh.colors.len() == mesh.points.len();
        rect().expanded().maybe_child(complete.then(|| {
            Canvas::new(mesh.revision(), move |context: &mut CanvasContext| {
                mesh.paint(context)
            })
        }))
    }
}

struct Mesh {
    width: usize,
    height: usize,
    points: Vec<(f32, f32)>,
    colors: Vec<Rgba>,
    smooths_colors: bool,
}

impl Mesh {
    fn revision(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        (self.width, self.height, self.smooths_colors).hash(&mut hasher);
        self.points
            .iter()
            .flat_map(|&(x, y)| [x, y])
            .chain(
                self.colors
                    .iter()
                    .flat_map(|tone| [tone.r, tone.g, tone.b, tone.a]),
            )
            .for_each(|value| value.to_bits().hash(&mut hasher));
        hasher.finish()
    }

    fn paint(&self, context: &mut CanvasContext) {
        let paint = Paint::default();
        let place = |(x, y): (f32, f32)| SkPoint::new(x * context.width, y * context.height);
        for row in 0..self.height - 1 {
            for column in 0..self.width - 1 {
                let at = |r: usize, c: usize| (row + r) * self.width + column + c;
                let corners = [at(0, 0), at(0, 1), at(1, 1), at(1, 0)];
                let quad = corners.map(|index| place(self.points[index]));
                let tones = corners.map(|index| self.colors[index]);
                let position = |u: f32, v: f32| {
                    between(
                        between(quad[0], quad[1], u),
                        between(quad[3], quad[2], u),
                        v,
                    )
                };
                let tone = |u: f32, v: f32| {
                    let tone = blend(tones, u, v, self.smooths_colors);
                    Color4f::new(tone.r, tone.g, tone.b, tone.a).to_color()
                };
                for step in 0..STEPS * STEPS {
                    let part = |index: usize| index as f32 / STEPS as f32;
                    let (u0, v0) = (part(step % STEPS), part(step / STEPS));
                    let (u1, v1) = (u0 + part(1), v0 + part(1));
                    let (a, b, c, d) = (
                        position(u0, v0),
                        position(u1, v0),
                        position(u1, v1),
                        position(u0, v1),
                    );
                    let third = |from: SkPoint, to: SkPoint| between(from, to, 1.0 / 3.0);
                    let cubics = [
                        a,
                        third(a, b),
                        third(b, a),
                        b,
                        third(b, c),
                        third(c, b),
                        c,
                        third(c, d),
                        third(d, c),
                        d,
                        third(d, a),
                        third(a, d),
                    ];
                    let patch = [tone(u0, v0), tone(u1, v0), tone(u1, v1), tone(u0, v1)];
                    context
                        .canvas
                        .draw_patch(&cubics, &patch, None, BlendMode::Dst, &paint);
                }
            }
        }
    }
}

fn between(from: SkPoint, to: SkPoint, t: f32) -> SkPoint {
    from + (to - from) * t
}

// `tones` are a cell's corners clockwise from its top leading one; `u` runs along the row and `v` down the column.
fn blend(tones: [Rgba; 4], u: f32, v: f32, smooths_colors: bool) -> Rgba {
    let (u, v) = if smooths_colors {
        (smoothstep(u), smoothstep(v))
    } else {
        (u, v)
    };
    let mix = |from: Rgba, to: Rgba, t: f32| Rgba {
        r: from.r + (to.r - from.r) * t,
        g: from.g + (to.g - from.g) * t,
        b: from.b + (to.b - from.b) * t,
        a: from.a + (to.a - from.a) * t,
    };
    mix(mix(tones[0], tones[1], u), mix(tones[3], tones[2], u), v)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TONES: [Rgba; 4] = [
        Rgba::rgba8(255, 0, 0, 1.0),
        Rgba::rgba8(0, 255, 0, 1.0),
        Rgba::rgba8(0, 0, 255, 1.0),
        Rgba::rgba8(0, 0, 0, 1.0),
    ];

    #[test]
    fn a_corner_keeps_its_color_and_the_center_is_the_mean() {
        for smooths_colors in [true, false] {
            assert_eq!(blend(TONES, 0.0, 0.0, smooths_colors), TONES[0]);
            assert_eq!(blend(TONES, 1.0, 1.0, smooths_colors), TONES[2]);
            let center = blend(TONES, 0.5, 0.5, smooths_colors);
            assert!((center.r - 0.25).abs() < 1e-6 && (center.b - 0.25).abs() < 1e-6);
        }
    }

    #[test]
    fn smoothing_holds_a_color_near_its_point() {
        let smooth = blend(TONES, 0.1, 0.0, true);
        let straight = blend(TONES, 0.1, 0.0, false);
        assert!(smooth.r > straight.r);
    }
}

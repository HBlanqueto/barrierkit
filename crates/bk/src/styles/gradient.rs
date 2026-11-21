use std::f32::consts::FRAC_PI_2;

use ctdesign::Rgba;
use freya_core::prelude::{Fill, FillRenderContext, LinearGradient, RenderCallback, RenderTarget};
use freya_engine::prelude::gradient::interpolation::{ColorSpace, HueMethod, InPremul};
use freya_engine::prelude::{
    Color4f, Colors, Gradient as SkGradient, Interpolation, Paint, Shader, SkMatrix, SkPoint,
    SkRect, TileMode, shaders,
};

use super::ShapeStyle;
use crate::theme::color;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GradientColorSpace {
    #[default]
    Device,
    Perceptual,
}

impl GradientColorSpace {
    fn interpolation(self) -> Interpolation {
        match self {
            Self::Device => Interpolation::default(),
            Self::Perceptual => Interpolation {
                in_premul: InPremul::Yes,
                color_space: ColorSpace::OKLab,
                hue_method: HueMethod::Shorter,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum GradientShape {
    Linear { angle: f32 },
    Angular { angle: f32 },
    Elliptical,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    shape: GradientShape,
    pub(super) stops: Vec<(ShapeStyle, f32)>,
    color_space: GradientColorSpace,
}

impl Gradient {
    pub fn new(angle: f32, stops: Vec<(ShapeStyle, f32)>) -> Self {
        Self::of(GradientShape::Linear { angle }, stops)
    }

    pub fn angular(angle: f32, stops: Vec<(ShapeStyle, f32)>) -> Self {
        Self::of(GradientShape::Angular { angle }, stops)
    }

    pub fn elliptical(stops: Vec<(ShapeStyle, f32)>) -> Self {
        Self::of(GradientShape::Elliptical, stops)
    }

    pub fn color_space(mut self, color_space: GradientColorSpace) -> Self {
        self.color_space = color_space;
        self
    }

    fn of(shape: GradientShape, stops: Vec<(ShapeStyle, f32)>) -> Self {
        Self {
            shape,
            stops,
            color_space: GradientColorSpace::default(),
        }
    }

    pub(super) fn fill(&self, color_of: impl Fn(&ShapeStyle) -> Rgba) -> Fill {
        let stops: Vec<(Rgba, f32)> = self
            .stops
            .iter()
            .map(|(stop, position)| (color_of(stop), *position))
            .collect();
        match (self.shape, self.color_space) {
            (GradientShape::Linear { angle }, GradientColorSpace::Device) => stops
                .iter()
                .fold(
                    LinearGradient::new().angle(angle),
                    |linear, (stop, position)| linear.stop((color(*stop), position * 100.0)),
                )
                .into(),
            (shape, space) => Fill::Callback(RenderCallback::new(move |context| {
                draw(shape, space, &stops, context)
            })),
        }
    }
}

fn draw(
    shape: GradientShape,
    space: GradientColorSpace,
    stops: &[(Rgba, f32)],
    context: &mut FillRenderContext,
) {
    let (x, y) = match context.target {
        RenderTarget::Background { .. } => (0.0, 0.0),
        RenderTarget::TextStyle(_) => (context.origin.x, context.origin.y),
    };
    let area = SkRect::from_xywh(x, y, context.size.width, context.size.height);
    let Some(shader) = shader(shape, space, stops, area) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_shader(shader);
    context.set_paint(&paint);
}

fn shader(
    shape: GradientShape,
    space: GradientColorSpace,
    stops: &[(Rgba, f32)],
    area: SkRect,
) -> Option<Shader> {
    let colors: Vec<Color4f> = stops
        .iter()
        .map(|(c, _)| Color4f::new(c.r, c.g, c.b, c.a))
        .collect();
    let offsets: Vec<f32> = stops.iter().map(|&(_, position)| position).collect();
    let gradient = SkGradient::new(
        Colors::new(&colors, Some(&offsets), TileMode::Clamp, None),
        space.interpolation(),
    );
    let center = area.center();
    match shape {
        GradientShape::Linear { angle } => {
            shaders::linear_gradient(linear_ends(angle, area), &gradient, None)
        }
        GradientShape::Angular { angle } => {
            shaders::sweep_gradient(center, (angle, angle + 360.0), &gradient, None)
        }
        GradientShape::Elliptical => {
            let (width, height) = (area.width(), area.height());
            if width <= 0.0 {
                return None;
            }
            let mut squash = SkMatrix::new_identity();
            squash.set_scale((1.0, height / width), center);
            shaders::radial_gradient((center, width / 2.0), &gradient, &squash)
        }
    }
}

// Freya's own `LinearGradient::prepare_shader` places the ends the same way; its interpolation is not selectable.
fn linear_ends(angle: f32, area: SkRect) -> (SkPoint, SkPoint) {
    let (dy, dx) = (angle.to_radians() + FRAC_PI_2).sin_cos();
    let size = SkPoint::new(area.width(), area.height());
    let farthest = SkPoint::new(
        if dx > 0.0 { size.x } else { 0.0 },
        if dy > 0.0 { size.y } else { 0.0 },
    );
    let delta = farthest - size / 2.0;
    let u = delta.x * dy - delta.y * dx;
    let end = farthest + SkPoint::new(-u * dy, u * dx);
    let origin = SkPoint::new(area.left, area.top);
    (size - end + origin, end + origin)
}

#[cfg(test)]
mod tests {
    use freya_engine::prelude::{AlphaType, ColorType, ImageInfo, raster_n32_premul};

    use super::*;

    fn stops(from: [f32; 4], to: [f32; 4]) -> Vec<(Rgba, f32)> {
        let rgba = |[r, g, b, a]: [f32; 4]| Rgba { r, g, b, a };
        vec![(rgba(from), 0.0), (rgba(to), 1.0)]
    }

    fn pixels(
        shape: GradientShape,
        space: GradientColorSpace,
        stops: &[(Rgba, f32)],
        (width, height): (i32, i32),
        at: &[(i32, i32)],
    ) -> Vec<[u8; 4]> {
        let mut surface = raster_n32_premul((width, height)).unwrap();
        let area = SkRect::from_wh(width as f32, height as f32);
        let mut paint = Paint::default();
        paint.set_shader(shader(shape, space, stops, area).unwrap());
        surface.canvas().draw_rect(area, &paint);
        let info = ImageInfo::new((1, 1), ColorType::RGBA8888, AlphaType::Premul, None);
        at.iter()
            .map(|&point| {
                let mut out = [0u8; 4];
                surface.read_pixels(&info, &mut out, 4, point);
                out
            })
            .collect()
    }

    fn near(actual: [u8; 4], expected: [u8; 4]) {
        assert!(
            actual.iter().zip(expected).all(|(a, e)| a.abs_diff(e) <= 3),
            "{actual:?} is not near {expected:?}"
        );
    }

    const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
    const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
    const LEFT_TO_RIGHT: GradientShape = GradientShape::Linear { angle: 270.0 };

    #[test]
    fn perceptual_gradient_mixes_red_and_blue_as_swiftui_does() {
        let at = pixels(
            LEFT_TO_RIGHT,
            GradientColorSpace::Perceptual,
            &stops(RED, BLUE),
            (101, 4),
            &[(25, 2), (50, 2), (75, 2)],
        );
        near(at[0], [196, 74, 110, 255]);
        near(at[1], [140, 83, 162, 255]);
        near(at[2], [81, 71, 210, 255]);
    }

    #[test]
    fn perceptual_gradient_fades_to_a_transparent_stop_in_premultiplied_space() {
        let to = [0.0, 0.0, 1.0, 0.0];
        let at = pixels(
            LEFT_TO_RIGHT,
            GradientColorSpace::Perceptual,
            &stops(RED, to),
            (101, 4),
            &[(50, 2)],
        );
        near(at[0], [127, 0, 0, 128]);
    }

    #[test]
    fn angular_gradient_starts_on_the_right_and_runs_clockwise() {
        let at = pixels(
            GradientShape::Angular { angle: 0.0 },
            GradientColorSpace::Perceptual,
            &stops(RED, BLUE),
            (101, 101),
            &[(95, 50), (50, 95), (5, 50)],
        );
        near(at[0], [255, 0, 0, 255]);
        near(at[1], [198, 73, 109, 255]);
        near(at[2], [141, 82, 162, 255]);
    }

    #[test]
    fn elliptical_gradient_reaches_half_the_width_and_half_the_height_at_the_same_tone() {
        let white = [1.0; 4];
        let black = [0.0, 0.0, 0.0, 1.0];
        let at = pixels(
            GradientShape::Elliptical,
            GradientColorSpace::Perceptual,
            &stops(white, black),
            (200, 100),
            &[(150, 50), (100, 75)],
        );
        near(at[0], [98, 98, 98, 255]);
        near(at[1], [98, 98, 98, 255]);
    }
}

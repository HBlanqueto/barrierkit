use std::f32::consts::FRAC_PI_2;

use ctdesign::Rgba;
use freya_core::prelude::{Fill, FillRenderContext, LinearGradient, RenderCallback, RenderTarget};
use freya_engine::prelude::gradient::interpolation::{ColorSpace, HueMethod, InPremul};
use freya_engine::prelude::{
    Color4f, Colors, Gradient as SkGradient, Interpolation, Paint, Shader, SkMatrix, SkPoint,
    SkRect, TileMode, shaders,
};

use crate::styles::ShapeStyle;
use crate::theme::color;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GradientColorSpace {
    Device,
    #[default]
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

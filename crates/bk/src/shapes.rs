use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use freya_core::prelude::*;
use freya_engine::prelude::{
    BlendMode, Paint, PaintStyle, Path as SkPath, PathBuilder, RRect, Rect as SkRect, SaveLayerRec,
};
use torin::prelude::{Area, Size};

use crate::canvas::{Canvas, CanvasContext};
use crate::elements::forward_element;
use crate::styles::{
    ControlState, CornerStyle, LayoutDirection, Paints, Shape, ShapeStyle, StateValue,
    system_effect,
};

// Skia scales radii down to fit the frame, so a capsule or a circle clips with a radius no frame reaches.
pub(crate) const UNBOUNDED: f32 = 16_384.0;

type Point = (f32, f32);

#[derive(Clone, Copy, Debug, PartialEq)]
enum Segment {
    Move(Point),
    Line(Point),
    Curve(Point, Point, Point),
    Close,
}

#[derive(Clone, Debug, PartialEq)]
enum Outline {
    Rounded(Shape),
    Ellipse,
    Path(Vec<Segment>),
}

impl Outline {
    fn path(&self, width: f32, height: f32) -> SkPath {
        match self {
            Self::Rounded(shape) => rounded_path(*shape, width, height),
            Self::Ellipse => PathBuilder::new()
                .add_oval(SkRect::from_xywh(0.0, 0.0, width, height), None, None)
                .detach(),
            Self::Path(segments) => segments
                .iter()
                .fold(PathBuilder::new(), |mut path, segment| {
                    match *segment {
                        Segment::Move(to) => path.move_to(to),
                        Segment::Line(to) => path.line_to(to),
                        Segment::Curve(to, first, second) => path.cubic_to(first, second, to),
                        Segment::Close => path.close(),
                    };
                    path
                })
                .detach(),
        }
    }

    // The outline that a border of `by` on each side leaves inside, with the corner that stays concentric.
    fn inset(&self, by: f32) -> Self {
        match self {
            Self::Rounded(Shape::RoundedRectangle { radius, style }) => {
                Self::Rounded(Shape::RoundedRectangle {
                    radius: (radius - by).max(0.0),
                    style: *style,
                })
            }
            other => other.clone(),
        }
    }
}

fn rounded_path(shape: Shape, width: f32, height: f32) -> SkPath {
    let corner = shape.corner_radius(width, height, LayoutDirection::LeftToRight);
    let frame = if shape == Shape::Circle {
        let side = width.min(height);
        SkRect::from_xywh((width - side) / 2.0, (height - side) / 2.0, side, side)
    } else {
        SkRect::from_xywh(0.0, 0.0, width, height)
    };
    let round = |radius: f32| (radius, radius).into();
    let rrect = RRect::new_rect_radii(
        frame,
        &[
            round(corner.top_left()),
            round(corner.top_right()),
            round(corner.bottom_right()),
            round(corner.bottom_left()),
        ],
    );
    if corner.smoothing() > 0.0 {
        PathBuilder::new_path(&corner.smoothed_path(rrect))
            .close()
            .detach()
    } else {
        PathBuilder::new().add_rrect(rrect, None, None).detach()
    }
}

#[derive(Clone)]
struct Drawing {
    outline: Outline,
    fill: Option<Fill>,
    stroke: Option<(Fill, f32)>,
}

fn fill_of(style: ShapeStyle) -> Fill {
    Paints::current(ControlState::default(), system_effect).fill(&StateValue::new(style))
}

// A gradient or backdrop is a render callback that fills everything; the path's coverage keeps part of it.
fn ink(context: &mut CanvasContext, path: &SkPath, fill: &Fill, line_width: Option<f32>) {
    let size = (context.width, context.height);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    if let Some(width) = line_width {
        paint.set_style(PaintStyle::Stroke);
        paint.set_stroke_width(width);
    }
    let Fill::Callback(callback) = fill else {
        fill.apply_to_paint(&mut paint, Area::new((0.0, 0.0).into(), size.into()));
        context.canvas.draw_path(path, &paint);
        return;
    };
    context.canvas.save_layer(&SaveLayerRec::default());
    context.canvas.draw_path(path, &paint);
    let mut within = Paint::default();
    within.set_blend_mode(BlendMode::SrcIn);
    context
        .canvas
        .save_layer(&SaveLayerRec::default().paint(&within));
    callback.call(&mut FillRenderContext {
        target: RenderTarget::Background {
            canvas: context.canvas,
            font_collection: &mut *context.fonts,
        },
        scale_factor: f64::from(context.scale),
        origin: (0.0, 0.0).into(),
        size: size.into(),
        text_style_state: &TextStyleState::default(),
    });
    context.canvas.restore();
    context.canvas.restore();
}

impl Drawing {
    fn revision(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        format!("{:?}", self.outline).hash(&mut hasher);
        self.fill.hash(&mut hasher);
        self.stroke
            .as_ref()
            .map(|(fill, width)| (fill, width.to_bits()))
            .hash(&mut hasher);
        hasher.finish()
    }

    fn canvas(&self) -> Canvas {
        let drawing = self.clone();
        Canvas::new(self.revision(), move |context| drawing.paint(context))
    }

    fn paint(&self, context: &mut CanvasContext) {
        let (width, height) = (context.width, context.height);
        if let Some(fill) = &self.fill {
            ink(context, &self.outline.path(width, height), fill, None);
        }
        if let Some((stroke, line_width)) = &self.stroke {
            // A path keeps its own coordinates; the others sit inside the frame, which the canvas clips to.
            let inset = if matches!(self.outline, Outline::Path(_)) {
                0.0
            } else {
                line_width / 2.0
            };
            let path = self
                .outline
                .inset(inset)
                .path(width - 2.0 * inset, height - 2.0 * inset)
                .with_offset((inset, inset));
            ink(context, &path, stroke, Some(*line_width));
        }
    }
}

fn frame_of(drawing: &Drawing) -> Rect {
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .child(drawing.canvas())
}

macro_rules! shape_view {
    ($name:ident, $stroke:ident) => {
        pub struct $name(Rect, Drawing);

        forward_element!($name);

        impl $name {
            fn of(outline: Outline) -> Self {
                let drawing = Drawing {
                    outline,
                    fill: None,
                    stroke: None,
                };
                Self(frame_of(&drawing), drawing)
            }

            fn drawn(self, drawing: Drawing) -> Self {
                let mut frame = self.0;
                *frame.children_mut() = vec![drawing.canvas().into()];
                Self(frame, drawing)
            }

            pub fn fill(self, style: ShapeStyle) -> Self {
                let drawing = Drawing {
                    fill: Some(fill_of(style)),
                    ..self.1.clone()
                };
                self.drawn(drawing)
            }

            pub fn $stroke(self, style: ShapeStyle, line_width: f32) -> Self {
                let drawing = Drawing {
                    stroke: Some((fill_of(style), line_width)),
                    ..self.1.clone()
                };
                self.drawn(drawing)
            }
        }
    };
}

macro_rules! unit_shape {
    ($name:ident, $outline:expr) => {
        impl $name {
            pub fn new() -> Self {
                Self::of($outline)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

shape_view!(Rectangle, stroke_border);
shape_view!(RoundedRectangle, stroke_border);
shape_view!(Capsule, stroke_border);
shape_view!(Circle, stroke_border);
shape_view!(Ellipse, stroke_border);
shape_view!(Path, stroke);

unit_shape!(Rectangle, Outline::Rounded(Shape::Rectangle));
unit_shape!(Capsule, Outline::Rounded(Shape::Capsule));
unit_shape!(Circle, Outline::Rounded(Shape::Circle));
unit_shape!(Ellipse, Outline::Ellipse);
unit_shape!(Path, Outline::Path(Vec::new()));

impl RoundedRectangle {
    pub fn new(corner_radius: f32) -> Self {
        Self::of(Outline::Rounded(Shape::RoundedRectangle {
            radius: corner_radius,
            style: CornerStyle::Continuous,
        }))
    }
}

impl Path {
    fn with(self, segment: Segment) -> Self {
        let mut segments = match &self.1.outline {
            Outline::Path(segments) => segments.clone(),
            _ => Vec::new(),
        };
        segments.push(segment);
        let drawing = Drawing {
            outline: Outline::Path(segments),
            ..self.1.clone()
        };
        self.drawn(drawing)
    }

    pub fn move_to(self, to: Point) -> Self {
        self.with(Segment::Move(to))
    }

    pub fn add_line(self, to: Point) -> Self {
        self.with(Segment::Line(to))
    }

    pub fn add_curve(self, to: Point, control1: Point, control2: Point) -> Self {
        self.with(Segment::Curve(to, control1, control2))
    }

    pub fn close_subpath(self) -> Self {
        self.with(Segment::Close)
    }
}

pub trait ShapeModifiers: DecorationExt + EffectExt + Sized {
    fn clip_shape(self, shape: Shape) -> Self {
        self.corner_radius(shape.corner_radius(UNBOUNDED, UNBOUNDED, LayoutDirection::LeftToRight))
            .overflow(Overflow::Clip)
    }
}

impl<T: DecorationExt + EffectExt> ShapeModifiers for T {}

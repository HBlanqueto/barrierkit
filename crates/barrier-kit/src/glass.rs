//! Progressive glass behind the full-screen titlebar: the window's own content
//! blurred in stacked layers whose radius halves from the window edge inwards.

use std::borrow::Cow;
use std::rc::Rc;

use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{
    BlendMode, Color4f, Colors, Gradient, Interpolation, Paint, PaintStyle, RRect, Rect as SkRect,
    SaveLayerRec, TileMode, blur, shaders,
};
use torin::prelude::{Position, Size};
use vow_design::{FULLSCREEN_GLASS, MEDIA, Rgba};

use crate::theme::{SurfaceKind, surface_background};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum GlassEdge {
    Top,
    Left,
    Right,
}

#[derive(PartialEq)]
struct Glass {
    edge: GlassEdge,
    tint: Rgba,
    layout: LayoutData,
}

pub(crate) fn glass_tint() -> Rgba {
    if crate::backdrop::tinted().is_some() {
        crate::style::Palette::current().raised_background
    } else {
        surface_background(&crate::theme::resolved(), SurfaceKind::Toplevel)
    }
}

pub(crate) fn progressive_glass(edge: GlassEdge) -> Element {
    let depth = Size::px(FULLSCREEN_GLASS.height);
    let (width, height, position) = match edge {
        GlassEdge::Top => (Size::fill(), depth, Position::new_absolute()),
        GlassEdge::Left => (depth, Size::fill(), Position::new_absolute().left(0.0)),
        GlassEdge::Right => (depth, Size::fill(), Position::new_absolute().right(0.0)),
    };
    let layout = torin::node::Node {
        width,
        height,
        position,
        ..Default::default()
    };
    Element::Element {
        key: DiffKey::None,
        element: Rc::new(Glass {
            edge,
            tint: glass_tint(),
            layout: layout.into(),
        }),
        elements: Vec::new(),
    }
}

fn strength_at(stops: &[(f32, f32)], t: f32) -> f32 {
    stops
        .windows(2)
        .find(|pair| t <= pair[1].0)
        .map_or(0.0, |pair| {
            let ((a, from), (b, to)) = (pair[0], pair[1]);
            from + (to - from) * ((t - a) / (b - a).max(f32::EPSILON)).clamp(0.0, 1.0)
        })
}

// Each layer fades in between the previous radius and its own, so stacked radii blend without seams.
fn layer_mask(floor: f32, level: f32) -> Vec<(f32, f32)> {
    const SAMPLES: usize = 16;
    (0..=SAMPLES)
        .map(|sample| {
            let t = sample as f32 / SAMPLES as f32;
            let strength = strength_at(FULLSCREEN_GLASS.mask, t);
            (t, ((strength - floor) / (level - floor)).clamp(0.0, 1.0))
        })
        .collect()
}

fn inwards(edge: GlassEdge, bounds: SkRect, color: Rgba, stops: &[(f32, f32)]) -> Paint {
    let (colors, offsets): (Vec<Color4f>, Vec<f32>) = stops
        .iter()
        .map(|&(offset, alpha)| {
            (
                Color4f::new(color.r, color.g, color.b, color.a * alpha),
                offset,
            )
        })
        .unzip();
    let gradient = Gradient::new(
        Colors::new(&colors, Some(&offsets), TileMode::Clamp, None),
        Interpolation::default(),
    );
    let mut paint = Paint::default();
    paint.set_dither(true);
    let line = match edge {
        GlassEdge::Top => ((bounds.left, bounds.top), (bounds.left, bounds.bottom)),
        GlassEdge::Left => ((bounds.left, bounds.top), (bounds.right, bounds.top)),
        GlassEdge::Right => ((bounds.right, bounds.top), (bounds.left, bounds.top)),
    };
    paint.set_shader(shaders::linear_gradient(line, &gradient, None));
    paint
}

impl ElementExt for Glass {
    fn changed(&self, other: &Rc<dyn ElementExt>) -> bool {
        (other.as_ref() as &dyn std::any::Any)
            .downcast_ref::<Self>()
            .is_none_or(|other| other != self)
    }

    fn diff(&self, other: &Rc<dyn ElementExt>) -> DiffModifies {
        match (other.as_ref() as &dyn std::any::Any).downcast_ref::<Self>() {
            Some(other) if other == self => DiffModifies::empty(),
            Some(_) => DiffModifies::STYLE,
            None => DiffModifies::all(),
        }
    }

    fn layout(&'_ self) -> Cow<'_, LayoutData> {
        Cow::Borrowed(&self.layout)
    }

    fn render(&self, context: RenderContext) {
        let area = context.layout_node.visible_area();
        let bounds = SkRect::new(area.min_x(), area.min_y(), area.max_x(), area.max_y());
        let canvas = context.canvas;
        let layers = FULLSCREEN_GLASS.layers;
        for layer in 0..layers {
            let level = 0.5f32.powi((layers - 1 - layer) as i32);
            let sigma = FULLSCREEN_GLASS.blur * level * context.scale_factor as f32;
            // Uncropped, so the blur samples the content past the glass instead of smearing its last row.
            let Some(filter) = blur((sigma, sigma), TileMode::Clamp, None, None) else {
                continue;
            };
            let mut mask = inwards(
                self.edge,
                bounds,
                Rgba::rgb8(255, 255, 255),
                &layer_mask(level / 2.0, level),
            );
            mask.set_blend_mode(BlendMode::DstIn);
            canvas.save_layer(&SaveLayerRec::default().bounds(&bounds).backdrop(&filter));
            canvas.draw_rect(bounds, &mask);
            canvas.restore();
        }
        canvas.draw_rect(
            bounds,
            &inwards(self.edge, bounds, self.tint, FULLSCREEN_GLASS.tint),
        );
    }
}

#[derive(PartialEq)]
struct Material {
    corner_radius: f32,
    layout: LayoutData,
}

pub(crate) fn material(corner_radius: f32) -> Element {
    let layout = torin::node::Node {
        width: Size::fill(),
        height: Size::fill(),
        position: Position::new_absolute(),
        ..Default::default()
    };
    Element::Element {
        key: DiffKey::None,
        element: Rc::new(Material {
            corner_radius,
            layout: layout.into(),
        }),
        elements: Vec::new(),
    }
}

impl ElementExt for Material {
    fn changed(&self, other: &Rc<dyn ElementExt>) -> bool {
        (other.as_ref() as &dyn std::any::Any)
            .downcast_ref::<Self>()
            .is_none_or(|other| other != self)
    }

    fn diff(&self, other: &Rc<dyn ElementExt>) -> DiffModifies {
        match (other.as_ref() as &dyn std::any::Any).downcast_ref::<Self>() {
            Some(other) if other == self => DiffModifies::empty(),
            Some(_) => DiffModifies::STYLE,
            None => DiffModifies::all(),
        }
    }

    fn layout(&'_ self) -> Cow<'_, LayoutData> {
        Cow::Borrowed(&self.layout)
    }

    fn render(&self, context: RenderContext) {
        let area = context.layout_node.visible_area();
        let bounds = SkRect::new(area.min_x(), area.min_y(), area.max_x(), area.max_y());
        let radius = self.corner_radius * context.scale_factor as f32;
        let shape = RRect::new_rect_xy(bounds, radius, radius);
        let canvas = context.canvas;
        let sigma = MEDIA.material_blur * context.scale_factor as f32;
        canvas.save();
        canvas.clip_rrect(shape, None, Some(true));
        if let Some(filter) = blur((sigma, sigma), TileMode::Clamp, None, None) {
            canvas.save_layer(&SaveLayerRec::default().bounds(&bounds).backdrop(&filter));
            canvas.restore();
        }
        let mut tint = Paint::default();
        tint.set_color4f(rgba4f(MEDIA.material_tint), None);
        canvas.draw_rrect(shape, &tint);
        canvas.restore();
        let mut rim = Paint::default();
        rim.set_style(PaintStyle::Stroke);
        rim.set_stroke_width(context.scale_factor as f32);
        rim.set_anti_alias(true);
        rim.set_color4f(rgba4f(MEDIA.material_rim), None);
        let inset = context.scale_factor as f32 / 2.0;
        canvas.draw_rrect(
            RRect::new_rect_xy(bounds.with_inset((inset, inset)), radius, radius),
            &rim,
        );
    }
}

fn rgba4f(color: Rgba) -> Color4f {
    Color4f::new(color.r, color.g, color.b, color.a)
}

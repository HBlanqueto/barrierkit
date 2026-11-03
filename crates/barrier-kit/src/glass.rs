//! Progressive glass behind the full-screen titlebar: the window's own content
//! blurred in thin bands whose radius shrinks from the window edge inwards.

use std::borrow::Cow;
use std::rc::Rc;

use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{
    Color4f, Colors, Gradient, Interpolation, Paint, Rect as SkRect, SaveLayerRec, TileMode, blur,
    shaders,
};
use torin::prelude::{Position, Size};
use vow_design::{FULLSCREEN_GLASS, Rgba};

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

fn band_rect(edge: GlassEdge, bounds: SkRect, from: f32, to: f32) -> SkRect {
    let (w, h) = (bounds.width(), bounds.height());
    match edge {
        GlassEdge::Top => SkRect::new(
            bounds.left,
            bounds.top + h * from,
            bounds.right,
            bounds.top + h * to,
        ),
        GlassEdge::Left => SkRect::new(
            bounds.left + w * from,
            bounds.top,
            bounds.left + w * to,
            bounds.bottom,
        ),
        GlassEdge::Right => SkRect::new(
            bounds.right - w * to,
            bounds.top,
            bounds.right - w * from,
            bounds.bottom,
        ),
    }
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
        let bands = FULLSCREEN_GLASS.bands;
        for band in 0..bands {
            let (from, to) = (band as f32 / bands as f32, (band + 1) as f32 / bands as f32);
            let strength = strength_at(FULLSCREEN_GLASS.mask, (from + to) / 2.0);
            let sigma = FULLSCREEN_GLASS.blur * strength * context.scale_factor as f32;
            // Uncropped, so the blur samples the content past the glass instead of smearing its last row.
            let Some(filter) = blur((sigma, sigma), TileMode::Clamp, None, None) else {
                continue;
            };
            let slice = band_rect(self.edge, bounds, from, to);
            canvas.save();
            canvas.clip_rect(slice, None, Some(true));
            canvas.save_layer(&SaveLayerRec::default().bounds(&slice).backdrop(&filter));
            canvas.restore();
            canvas.restore();
        }
        canvas.draw_rect(
            bounds,
            &inwards(self.edge, bounds, self.tint, FULLSCREEN_GLASS.tint),
        );
    }
}

//! Progressive glass behind the full-screen titlebar: the window's own content
//! blurred, strongest at the window edge and fading out inwards.

use std::borrow::Cow;
use std::rc::Rc;

use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{
    BlendMode, Color4f, Colors, Gradient, Interpolation, Paint, Rect as SkRect, SaveLayerRec,
    TileMode, blur, shaders,
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
        let sigma = FULLSCREEN_GLASS.blur * context.scale_factor as f32;
        let Some(filter) = blur((sigma, sigma), TileMode::Clamp, None, bounds) else {
            return;
        };
        let canvas = context.canvas;
        canvas.save_layer(&SaveLayerRec::default().bounds(&bounds).backdrop(&filter));
        let tint = inwards(self.edge, bounds, self.tint, FULLSCREEN_GLASS.tint);
        canvas.draw_rect(bounds, &tint);
        let black = Rgba::rgba8(0, 0, 0, 1.0);
        let mut mask = inwards(self.edge, bounds, black, FULLSCREEN_GLASS.mask);
        mask.set_blend_mode(BlendMode::DstIn);
        canvas.draw_rect(bounds, &mask);
        canvas.restore();
    }
}

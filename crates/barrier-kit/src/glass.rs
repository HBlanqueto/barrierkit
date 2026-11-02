//! Progressive glass behind the full-screen titlebar: the window's own content
//! blurred, strongest at the top and fading out downwards.

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

#[derive(PartialEq)]
struct Glass {
    tint: Rgba,
    layout: LayoutData,
}

pub(crate) fn progressive_glass(tint: Rgba) -> Element {
    let layout = torin::node::Node {
        width: Size::fill(),
        height: Size::px(FULLSCREEN_GLASS.height),
        position: Position::new_absolute(),
        ..Default::default()
    };
    Element::Element {
        key: DiffKey::None,
        element: Rc::new(Glass {
            tint,
            layout: layout.into(),
        }),
        elements: Vec::new(),
    }
}

fn vertical(bounds: SkRect, color: Rgba, stops: &[(f32, f32)]) -> Paint {
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
    paint.set_shader(shaders::linear_gradient(
        ((bounds.left, bounds.top), (bounds.left, bounds.bottom)),
        &gradient,
        None,
    ));
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
        canvas.draw_rect(bounds, &vertical(bounds, self.tint, FULLSCREEN_GLASS.tint));
        let mut mask = vertical(bounds, Rgba::rgba8(0, 0, 0, 1.0), FULLSCREEN_GLASS.mask);
        mask.set_blend_mode(BlendMode::DstIn);
        canvas.draw_rect(bounds, &mask);
        canvas.restore();
    }
}

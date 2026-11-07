use std::any::Any;
use std::borrow::Cow;
use std::rc::Rc;

use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{Canvas as SkCanvas, FontCollection, Rect as SkRect};
use torin::prelude::{Position, Size};

use crate::glass::{layout, leaf};

pub struct CanvasContext<'a> {
    pub canvas: &'a SkCanvas,
    pub width: f32,
    pub height: f32,
    pub scale: f32,
    pub fonts: &'a mut FontCollection,
}

type Painter = Rc<dyn Fn(&mut CanvasContext)>;

struct CanvasElement {
    revision: u64,
    painter: Painter,
    layout: LayoutData,
}

pub struct Canvas {
    revision: u64,
    painter: Painter,
}

impl Canvas {
    pub fn new(revision: u64, painter: impl Fn(&mut CanvasContext) + 'static) -> Self {
        Self {
            revision,
            painter: Rc::new(painter),
        }
    }
}

impl From<Canvas> for Element {
    fn from(canvas: Canvas) -> Self {
        leaf(CanvasElement {
            revision: canvas.revision,
            painter: canvas.painter,
            layout: layout(Size::fill(), Size::fill(), Position::default()),
        })
    }
}

impl ElementExt for CanvasElement {
    fn changed(&self, other: &Rc<dyn ElementExt>) -> bool {
        !self.diff(other).is_empty()
    }

    fn diff(&self, other: &Rc<dyn ElementExt>) -> DiffModifies {
        match (other.as_ref() as &dyn Any).downcast_ref::<CanvasElement>() {
            Some(other) if other.revision == self.revision => DiffModifies::empty(),
            Some(_) => DiffModifies::STYLE,
            None => DiffModifies::all(),
        }
    }

    fn layout(&'_ self) -> Cow<'_, LayoutData> {
        Cow::Borrowed(&self.layout)
    }

    fn render(&self, context: RenderContext) {
        let area = context.layout_node.visible_area();
        let scale = context.scale_factor as f32;
        let canvas = context.canvas;
        canvas.save();
        canvas.clip_rect(
            SkRect::new(area.min_x(), area.min_y(), area.max_x(), area.max_y()),
            None,
            Some(true),
        );
        canvas.translate((area.min_x(), area.min_y()));
        canvas.scale((scale, scale));
        (self.painter)(&mut CanvasContext {
            canvas,
            width: area.width() / scale,
            height: area.height() / scale,
            scale,
            fonts: context.font_collection,
        });
        canvas.restore();
    }
}

use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use ctappearance::SidebarGlass;
use ctdesign::Rgba;
use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{BlendMode, Paint, RRect, Rect as SkRect};
use torin::prelude::{Area, Position, Size};

use crate::glass::{layout, leaf};
use crate::platform::{GlassPanels, GlassSupport, KitSender, set_glass_panels};

pub(crate) fn sidebar_glass() -> Option<SidebarGlass> {
    let supported = try_consume_root_context::<GlassSupport>().is_some_and(|support| support.0);
    let over_backdrop = crate::backdrop::tinted().is_some();
    crate::theme::resolved()
        .sidebar
        .glass
        .filter(|_| supported && !over_backdrop)
}

fn argb(color: Rgba) -> u32 {
    let [red, green, blue, _] = color.to_rgba8();
    u32::from_be_bytes([u8::MAX, red, green, blue])
}

pub(crate) fn use_publish(glass: Option<SidebarGlass>, radius: f32, area: Area) {
    let sent = use_hook(|| Rc::new(RefCell::new(GlassPanels::default())));
    let panels = glass
        .filter(|_| area.width() > 0.0 && area.height() > 0.0)
        .map_or_else(GlassPanels::default, |glass| GlassPanels {
            radius: radius.round() as u32,
            rects: vec![(
                area.min_x().round() as i32,
                area.min_y().round() as i32,
                area.width().round() as i32,
                area.height().round() as i32,
            )],
            base: argb(glass.base),
            solid: argb(glass.solid),
            chroma: (glass.chroma * 1000.0).round() as u32,
        });
    if *sent.borrow() != panels {
        *sent.borrow_mut() = panels.clone();
        set_glass_panels(panels);
    }
    let sender = use_hook(KitSender::get);
    use_drop(move || {
        if *sent.borrow() != GlassPanels::default() {
            sender.send(GlassPanels::default());
        }
    });
}

#[derive(PartialEq)]
struct Hole {
    radius: f32,
    layout: LayoutData,
}

// The compositor draws the glass under the window, so the window must leave the panel transparent.
pub(crate) fn hole(offset: (f32, f32), size: (f32, f32), radius: f32) -> Element {
    leaf(Hole {
        radius,
        layout: layout(
            Size::px(size.0),
            Size::px(size.1),
            Position::new_absolute().left(offset.0).top(offset.1),
        ),
    })
}

impl ElementExt for Hole {
    fn changed(&self, other: &Rc<dyn ElementExt>) -> bool {
        !crate::style_diff(self, other).is_empty()
    }

    fn diff(&self, other: &Rc<dyn ElementExt>) -> DiffModifies {
        crate::style_diff(self, other)
    }

    fn layout(&'_ self) -> Cow<'_, LayoutData> {
        Cow::Borrowed(&self.layout)
    }

    fn render(&self, context: RenderContext) {
        let area = context.layout_node.visible_area();
        let bounds = SkRect::new(area.min_x(), area.min_y(), area.max_x(), area.max_y());
        let radius = self.radius * context.scale_factor as f32;
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_blend_mode(BlendMode::Clear);
        context
            .canvas
            .draw_rrect(RRect::new_rect_xy(bounds, radius, radius), &paint);
    }
}

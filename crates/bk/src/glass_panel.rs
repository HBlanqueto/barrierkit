use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use ctappearance::SidebarGlass;
use ctdesign::{CORNER_WINDOW, Rgba};
use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{BlendMode, Paint, RRect, Rect as SkRect};
use torin::prelude::{Area, Position, Size};

use crate::elements::{Overrides, forward_overrides};
use crate::theme::{SurfaceKind, color, surface_background};

use crate::glass::{layout, leaf};
use crate::platform::{GlassOutbox, GlassPanels, GlassSupport};

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

// The sidebar and the inspector are the platform's own split items, whose glass shows what lies under them.
pub(crate) fn native_pane_glass() -> bool {
    sidebar_glass().is_some() && crate::glass::native_glass()
}

fn rounded(area: Area) -> (i32, i32, i32, i32) {
    (
        area.min_x().round() as i32,
        area.min_y().round() as i32,
        area.width().round() as i32,
        area.height().round() as i32,
    )
}

pub(crate) fn use_publish_inspector(area: Option<Area>) {
    let sent = use_hook(|| Rc::new(Cell::new(None)));
    let rect = area
        .filter(|area| area.width() > 0.0 && area.height() > 0.0)
        .map(rounded);
    let outbox = use_hook(GlassOutbox::get);
    if sent.get() != rect {
        sent.set(rect);
        outbox.put_inspector(rect);
    }
    use_drop(move || {
        if sent.get().is_some() {
            outbox.put_inspector(None);
        }
    });
}

pub(crate) fn use_publish(glass: Option<SidebarGlass>, radius: f32, area: Area) {
    let sent = use_hook(|| Rc::new(RefCell::new(GlassPanels::default())));
    let panels = glass
        .filter(|_| area.width() > 0.0 && area.height() > 0.0)
        .map_or_else(GlassPanels::default, |glass| GlassPanels {
            radius: radius.round() as u32,
            rects: vec![rounded(area)],
            base: argb(glass.base),
            solid: argb(glass.solid),
            chroma: (glass.chroma * infinity_protocol::THOUSANDTHS).round() as u32,
            inspector: None,
        });
    let outbox = use_hook(GlassOutbox::get);
    if *sent.borrow() != panels {
        *sent.borrow_mut() = panels.clone();
        outbox.put(panels);
    }
    use_drop(move || {
        if *sent.borrow() != GlassPanels::default() {
            outbox.put(GlassPanels::default());
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

    // The hole's size lives in `layout`: a style-only diff leaves it at the size it was first measured.
    fn diff(&self, other: &Rc<dyn ElementExt>) -> DiffModifies {
        let modifies = crate::style_diff(self, other);
        if modifies.is_empty() {
            modifies
        } else {
            modifies | DiffModifies::LAYOUT
        }
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

#[derive(PartialEq)]
pub struct WindowGlass {
    content: Element,
    overrides: Overrides,
}

impl WindowGlass {
    pub fn new(content: impl IntoElement) -> Self {
        Self {
            content: content.into_element(),
            overrides: Overrides::default(),
        }
    }
}

forward_overrides!(WindowGlass);

impl Component for WindowGlass {
    fn render(&self) -> impl IntoElement {
        let glass = sidebar_glass();
        let radius = CORNER_WINDOW.fitted;
        let mut area = use_state(Area::default);
        use_publish(glass, radius, *area.read());
        let root = rect()
            .expanded()
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .map(
                glass.is_none().then(crate::theme::resolved),
                |el, resolved| {
                    el.background(color(surface_background(&resolved, SurfaceKind::Toplevel)))
                },
            )
            .maybe_child(glass.map(|_| {
                let size = area.read().size;
                hole((0.0, 0.0), (size.width, size.height), radius)
            }))
            .child(self.content.clone());
        self.overrides.apply(root)
    }
}

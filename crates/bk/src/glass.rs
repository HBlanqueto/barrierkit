use std::borrow::Cow;
use std::rc::Rc;

use ctdesign::Rgba;
use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{Color4f, Paint, PaintStyle, RRect, Rect as SkRect};
use torin::prelude::{Position, Size};

use crate::platform::backdrop::{BackdropSupport, Declared, GlassLook};
use crate::styles::{
    ControlState, FullscreenBarStyle, MaterialStyle, Paints, Theme, TopGlassStyle, resolve,
};

use crate::theme::{SurfaceKind, surface_background};

pub(crate) fn glass_tint() -> Rgba {
    if crate::backdrop::tinted().is_some() {
        crate::style::Palette::current().raised_background
    } else {
        surface_background(&crate::theme::resolved(), SurfaceKind::Toplevel)
    }
}

pub(crate) fn transparency_reduced() -> bool {
    crate::theme::resolved().reduce_transparency
}

// A control whose glass the platform draws declares it; the rest of the time it paints its own fill.
pub(crate) fn native_glass() -> bool {
    freya_core::current_context::CurrentContext::try_with(|_| {
        composed_by_compositor()
            && try_consume_root_context::<crate::platform::backdrop::NativeGlass>()
                .is_some_and(|support| support.0)
    })
    .unwrap_or(false)
}

fn composed_by_compositor() -> bool {
    !transparency_reduced()
        && try_consume_root_context::<BackdropSupport>().is_some_and(|support| support.0)
}

pub(crate) fn material(tint: Rgba, solid: Rgba, blur: f32, opaque: bool) -> (Rgba, Option<f32>) {
    if opaque {
        (Rgba { a: 1.0, ..solid }, None)
    } else {
        (tint, Some(blur))
    }
}

fn solid(depth: f32, tint: Rgba) -> Element {
    rect()
        .position(Position::new_absolute())
        .width(Size::fill())
        .height(Size::px(depth))
        .background(crate::theme::color(tint))
        .into_element()
}

// A native glass the size of its child, for a control whose width comes from its content: it shows once the child is laid out.
#[derive(PartialEq)]
pub(crate) struct GlassBehind {
    pub(crate) radius: f32,
    pub(crate) look: GlassLook,
    pub(crate) width: Size,
    pub(crate) child: Element,
}

impl Component for GlassBehind {
    fn render(&self) -> impl IntoElement {
        let mut size = use_state(|| None::<(f32, f32)>);
        rect()
            .width(self.width.clone())
            .on_sized(move |event: Event<SizedEventData>| {
                size.set_if_modified(Some((event.area.width(), event.area.height())))
            })
            .maybe_child(
                (*size.read())
                    .map(|size| Declared::glass(self.radius, size, (0.0, 0.0), self.look)),
            )
            .child(self.child.clone())
    }
}

pub(crate) fn leaf(element: impl ElementExt) -> Element {
    Element::Element {
        key: DiffKey::None,
        element: Rc::new(element),
        elements: Vec::new(),
    }
}

pub(crate) fn layout(width: Size, height: Size, position: Position) -> LayoutData {
    torin::node::Node {
        width,
        height,
        position,
        ..Default::default()
    }
    .into()
}

pub(crate) fn fullscreen_backdrop(depth: f32, opacity: f32) -> Element {
    let style: FullscreenBarStyle = resolve(&Theme::tengen());
    let paints = Paints::current(ControlState::default());
    let tint = paints.color(style.tint.resolve(ControlState::default()));
    solid(depth, Rgba { a: opacity, ..tint })
}

pub struct TopGlass {
    style: Theme,
}

impl TopGlass {
    pub fn new() -> Self {
        Self {
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(TopGlassStyle) -> TopGlassStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }
}

impl Default for TopGlass {
    fn default() -> Self {
        Self::new()
    }
}

impl From<TopGlass> for Element {
    fn from(top: TopGlass) -> Self {
        let style: TopGlassStyle = resolve(&top.style);
        if composed_by_compositor() {
            return Declared::soft_top_edge(style.glass.height);
        }
        let paints = Paints::current(ControlState::default());
        let tint = paints.color(style.tint.resolve(ControlState::default()));
        solid(style.glass.height, Rgba { a: 1.0, ..tint })
    }
}

#[derive(PartialEq)]
struct MaterialElement {
    corner_radius: f32,
    tint: Rgba,
    rim: Option<(Rgba, f32)>,
    layout: LayoutData,
}

pub struct Material {
    corner_radius: f32,
    style: Theme,
}

impl Material {
    pub fn new(corner_radius: f32) -> Self {
        Self {
            corner_radius,
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(MaterialStyle) -> MaterialStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }
}

impl From<Material> for Element {
    fn from(material: Material) -> Self {
        let style: MaterialStyle = resolve(&material.style);
        if composed_by_compositor() {
            return Declared::material(style.material, material.corner_radius);
        }
        let paints = Paints::current(ControlState::default());
        let tint = paints.color(style.tint.resolve(ControlState::default()));
        leaf(MaterialElement {
            corner_radius: material.corner_radius,
            tint: Rgba { a: 1.0, ..tint },
            rim: style
                .border
                .map(|(paint, width)| (paints.color(&paint), width)),
            layout: layout(Size::fill(), Size::fill(), Position::new_absolute()),
        })
    }
}

impl ElementExt for MaterialElement {
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
        let _material = tracing::trace_span!("material").entered();
        let area = context.layout_node.visible_area();
        let bounds = SkRect::new(area.min_x(), area.min_y(), area.max_x(), area.max_y());
        let radius = self.corner_radius * context.scale_factor as f32;
        let shape = RRect::new_rect_xy(bounds, radius, radius);
        let canvas = context.canvas;
        let mut tint = Paint::default();
        tint.set_anti_alias(true);
        tint.set_color4f(rgba4f(self.tint), None);
        canvas.draw_rrect(shape, &tint);
        let Some((color, width)) = self.rim else {
            return;
        };
        let width = width * context.scale_factor as f32;
        let mut rim = Paint::default();
        rim.set_style(PaintStyle::Stroke);
        rim.set_stroke_width(width);
        rim.set_anti_alias(true);
        rim.set_color4f(rgba4f(color), None);
        let inset = width / 2.0;
        canvas.draw_rrect(
            RRect::new_rect_xy(bounds.with_inset((inset, inset)), radius, radius),
            &rim,
        );
    }
}

fn rgba4f(color: Rgba) -> Color4f {
    Color4f::new(color.r, color.g, color.b, color.a)
}

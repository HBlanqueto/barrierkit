use std::borrow::Cow;
use std::rc::Rc;

use ctdesign::Rgba;
use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{Color4f, Paint, PaintStyle, RRect, Rect as SkRect};
use torin::prelude::{Position, Size};

use crate::platform::FullscreenBar;
use crate::platform::backdrop::{BackdropSupport, Declared, GlassLook};
use crate::styles::{
    ControlState, FullscreenBarStyle, MaterialStyle, Paints, Theme, TopGlassStyle, resolve,
};

use crate::theme::{SurfaceKind, surface_background};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum GlassEdge {
    Top,
    Left,
}

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

fn along(edge: GlassEdge, depth: f32) -> (Size, Size, Position) {
    let depth = Size::px(depth);
    match edge {
        GlassEdge::Top => (Size::fill(), depth, Position::new_absolute()),
        GlassEdge::Left => (depth, Size::fill(), Position::new_absolute().left(0.0)),
    }
}

fn solid(edge: GlassEdge, depth: f32, tint: Rgba) -> Element {
    let (width, height, position) = along(edge, depth);
    rect()
        .position(position)
        .width(width)
        .height(height)
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

fn fullscreen_depth(style: &FullscreenBarStyle, bar: FullscreenBar, depth: f32) -> f32 {
    match bar {
        FullscreenBar::ProgressiveGlass => style.glass.height,
        _ => depth,
    }
}

// The glass of a full-screen bar the compositor draws, over the strip the bar sits in.
pub(crate) fn fullscreen_glass_at(
    bar: FullscreenBar,
    depth: f32,
    position: Position,
    width: Size,
) -> Option<Element> {
    let style: FullscreenBarStyle = resolve(&Theme::tengen());
    let depth = fullscreen_depth(&style, bar, depth);
    composed_by_compositor()
        .then(|| {
            Declared::bar(
                bar,
                GlassEdge::Top,
                layout(width, Size::px(depth), position),
            )
        })
        .flatten()
}

pub(crate) fn fullscreen_backdrop(
    bar: FullscreenBar,
    edge: GlassEdge,
    depth: f32,
    opacity: f32,
) -> Element {
    let style: FullscreenBarStyle = resolve(&Theme::tengen());
    let depth = fullscreen_depth(&style, bar, depth);
    let (width, height, position) = along(edge, depth);
    if composed_by_compositor()
        && let Some(glass) = Declared::bar(bar, edge, layout(width, height, position))
    {
        return glass;
    }
    let paints = Paints::current(ControlState::default(), style.effect);
    let tint = paints.color(style.tint.resolve(ControlState::default()));
    solid(edge, depth, Rgba { a: opacity, ..tint })
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
        let paints = Paints::current(ControlState::default(), style.effect);
        let tint = paints.color(style.tint.resolve(ControlState::default()));
        solid(GlassEdge::Top, style.glass.height, Rgba { a: 1.0, ..tint })
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
        let paints = Paints::current(ControlState::default(), style.effect);
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

#[cfg(test)]
mod tests {
    use ctappearance::{AppearanceConfig, resolve};
    use freya_core::integration::AppComponent;
    use freya_testing::TestingRunner;

    use super::*;

    struct Case(fn() -> Element);

    impl App for Case {
        fn render(&self) -> impl IntoElement {
            (self.0)()
        }
    }

    fn painted(compositor: bool, reduced: bool, glass: fn() -> Element) -> Vec<u8> {
        let resolved = resolve(
            &AppearanceConfig {
                reduce_transparency: reduced,
                ..Default::default()
            },
            None,
        );
        let (mut test, _) = TestingRunner::new(
            AppComponent::new(Case(glass)),
            torin::prelude::Size2D::new(200.0, 100.0),
            move |runner| {
                runner.provide_root_context(move || State::create(resolved));
                runner.provide_root_context(move || BackdropSupport(compositor));
            },
            1.0,
        );
        test.sync_and_update();
        test.render().as_bytes().to_vec()
    }

    fn bare() -> Element {
        rect().expanded().background((200, 40, 40)).into()
    }

    fn with_top_glass() -> Element {
        rect()
            .expanded()
            .background((200, 40, 40))
            .child(TopGlass::new())
            .into()
    }

    fn with_material() -> Element {
        rect()
            .expanded()
            .background((200, 40, 40))
            .child(Material::new(12.0))
            .into()
    }

    #[test]
    fn a_compositor_that_composes_the_blur_leaves_the_glass_transparent() {
        let window = painted(true, false, bare);
        assert_eq!(painted(true, false, with_top_glass), window);
        assert_eq!(painted(true, false, with_material), window);
    }

    #[test]
    fn without_the_compositor_the_glass_is_the_kits_opaque_solid() {
        let window = painted(false, false, bare);
        assert_ne!(painted(false, false, with_top_glass), window);
        assert_ne!(painted(false, false, with_material), window);
        for glass in [with_top_glass, with_material] {
            assert_eq!(painted(false, false, glass), painted(false, true, glass));
        }
    }

    #[test]
    fn a_reduced_transparency_glass_is_the_kits_solid_whoever_composes_the_blur() {
        for glass in [with_top_glass, with_material] {
            let solid = painted(false, true, glass);
            assert_eq!(painted(true, true, glass), solid);
            assert_ne!(solid, painted(false, true, bare));
        }
    }
}

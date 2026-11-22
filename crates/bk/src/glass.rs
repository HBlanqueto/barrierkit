use std::borrow::Cow;
use std::rc::Rc;

use ctdesign::{FULLSCREEN_BAR, FullscreenGlassMetrics, Rgba};
use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{
    BlendMode, Color4f, Colors, Gradient, Interpolation, Paint, PaintStyle, RRect, Rect as SkRect,
    SaveLayerRec, TileMode, blur, shaders,
};
use torin::prelude::{Position, Size};

use crate::platform::FullscreenBar;
use crate::platform::backdrop::{BackdropSupport, Declared};
use crate::styles::{
    ControlState, FullscreenBarStyle, MaterialStyle, Paints, Theme, TopGlassStyle, resolve,
};

use crate::theme::{SurfaceKind, surface_background};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum GlassEdge {
    Top,
    Left,
}

#[derive(PartialEq)]
struct Glass {
    edge: GlassEdge,
    metrics: FullscreenGlassMetrics,
    tint: Rgba,
    line: Option<Rgba>,
    opacity: f32,
    layout: LayoutData,
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

fn glass(
    edge: GlassEdge,
    metrics: FullscreenGlassMetrics,
    depth: f32,
    (tint, line): (Rgba, Option<Rgba>),
    opacity: f32,
) -> Element {
    let (width, height, position) = along(edge, depth);
    leaf(Glass {
        edge,
        metrics,
        tint,
        line,
        opacity,
        layout: layout(width, height, position),
    })
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

// An opacity layer would hold the backdrop blur, which then samples that empty layer instead of the window.
pub(crate) fn fullscreen_backdrop(
    bar: FullscreenBar,
    edge: GlassEdge,
    depth: f32,
    opacity: f32,
) -> Element {
    let style: FullscreenBarStyle = resolve(&Theme::tengen());
    let paints = Paints::current(ControlState::default(), style.effect);
    let tint = paints.color(style.tint.resolve(ControlState::default()));
    let bar = if transparency_reduced() {
        FullscreenBar::Solid
    } else {
        bar
    };
    match bar {
        FullscreenBar::Solid => solid(edge, depth, Rgba { a: opacity, ..tint }),
        FullscreenBar::Glass => glass(
            edge,
            style.band,
            depth,
            (
                tint,
                Some(paints.color(style.line.resolve(ControlState::default()))),
            ),
            opacity,
        ),
        FullscreenBar::ProgressiveGlass => {
            glass(edge, style.glass, style.glass.height, (tint, None), opacity)
        }
    }
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
        let paints = Paints::current(ControlState::default(), style.effect);
        let tint = paints.color(style.tint.resolve(ControlState::default()));
        if transparency_reduced() {
            solid(GlassEdge::Top, style.glass.height, Rgba { a: 1.0, ..tint })
        } else if composed_by_compositor() {
            Declared::soft_top_edge(style.glass.height)
        } else {
            glass(
                GlassEdge::Top,
                style.glass,
                style.glass.height,
                (tint, None),
                1.0,
            )
        }
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
fn layer_mask(mask: &[(f32, f32)], floor: f32, level: f32) -> Vec<(f32, f32)> {
    const SAMPLES: usize = 16;
    (0..=SAMPLES)
        .map(|sample| {
            let t = sample as f32 / SAMPLES as f32;
            let strength = strength_at(mask, t);
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
    };
    paint.set_shader(shaders::linear_gradient(line, &gradient, None));
    paint
}

impl ElementExt for Glass {
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
        let _glass = tracing::trace_span!("glass").entered();
        let area = context.layout_node.visible_area();
        let bounds = SkRect::new(area.min_x(), area.min_y(), area.max_x(), area.max_y());
        let canvas = context.canvas;
        let layers = self.metrics.layers;
        for layer in 0..layers {
            let level = 0.5f32.powi((layers - 1 - layer) as i32);
            let sigma = self.metrics.blur * level * context.scale_factor as f32;
            // Uncropped, so the blur samples the content past the glass instead of smearing its last row.
            let Some(filter) = blur((sigma, sigma), TileMode::Clamp, None, None) else {
                continue;
            };
            let mut mask = inwards(
                self.edge,
                bounds,
                Rgba {
                    a: self.opacity,
                    ..ctdesign::GLASS_MASK
                },
                &layer_mask(self.metrics.mask, level / 2.0, level),
            );
            mask.set_blend_mode(BlendMode::DstIn);
            // SaveLayerRec bounds are only a hint; without the clip the blurred layer covers the whole canvas.
            canvas.save();
            canvas.clip_rect(bounds, None, Some(true));
            canvas.save_layer(&SaveLayerRec::default().bounds(&bounds).backdrop(&filter));
            canvas.draw_rect(bounds, &mask);
            canvas.restore();
            canvas.restore();
        }
        canvas.draw_rect(
            bounds,
            &inwards(
                self.edge,
                bounds,
                Rgba {
                    a: self.tint.a * self.opacity,
                    ..self.tint
                },
                self.metrics.tint,
            ),
        );
        if let Some(line) = self.line {
            let width = FULLSCREEN_BAR.band_line * context.scale_factor as f32;
            let edge_line = match self.edge {
                GlassEdge::Top => SkRect::new(
                    bounds.left,
                    bounds.bottom - width,
                    bounds.right,
                    bounds.bottom,
                ),
                GlassEdge::Left => SkRect::new(
                    bounds.right - width,
                    bounds.top,
                    bounds.right,
                    bounds.bottom,
                ),
            };
            let mut paint = Paint::default();
            paint.set_color4f(
                rgba4f(Rgba {
                    a: line.a * self.opacity,
                    ..line
                }),
                None,
            );
            canvas.draw_rect(edge_line, &paint);
        }
    }
}

#[derive(PartialEq)]
struct MaterialElement {
    corner_radius: f32,
    tint: Rgba,
    blur: Option<f32>,
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
        let (tint, blur) = self::material(tint, tint, style.blur, transparency_reduced());
        leaf(MaterialElement {
            corner_radius: material.corner_radius,
            tint,
            blur,
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
        canvas.save();
        canvas.clip_rrect(shape, None, Some(true));
        if let Some(filter) = self.blur.and_then(|radius| {
            let sigma = radius * context.scale_factor as f32;
            blur((sigma, sigma), TileMode::Clamp, None, None)
        }) {
            canvas.save_layer(&SaveLayerRec::default().bounds(&bounds).backdrop(&filter));
            canvas.restore();
        }
        let mut tint = Paint::default();
        tint.set_color4f(rgba4f(self.tint), None);
        canvas.draw_rrect(shape, &tint);
        canvas.restore();
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
    fn without_the_compositor_the_glass_keeps_drawing_its_own_tint() {
        let window = painted(false, false, bare);
        assert_ne!(painted(false, false, with_top_glass), window);
        assert_ne!(painted(false, false, with_material), window);
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

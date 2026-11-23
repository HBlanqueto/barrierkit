use std::any::Any;
use std::borrow::Cow;

use ctappearance::material::Material;
use freya_core::data::LayoutData;
use freya_core::element::{Element, ElementExt, RenderContext};
use freya_core::integration::RenderPipeline;
use freya_core::layers::Layers;
use freya_core::node_id::NodeId;
use freya_core::prelude::Color;
use freya_core::tree::{DiffModifies, Tree};
use freya_engine::prelude::{Canvas, FontCollection, FontMgr};
use torin::prelude::{Area, Point2D, Position, Size, Size2D};

use super::FullscreenBar;
use super::glass::Rects;
use super::window_style::{Edge, ScrollEdgeEffectStyle, WireMaterial};
use crate::glass::GlassEdge;

#[derive(Clone, Copy)]
pub(crate) struct BackdropSupport(pub(crate) bool);

// The platform draws Liquid Glass itself: a control declares its glass and draws only what lies over it.
#[derive(Clone, Copy)]
pub(crate) struct NativeGlass(pub(crate) bool);

// The platform veils the window under a sheet itself (AppKit's `beginSheet`).
#[derive(Clone, Copy)]
pub(crate) struct NativeSheets(pub(crate) bool);

// Malaysia serves 4 requesting surfaces per toplevel and the sidebar's is the window's own.
const MOST_CHROMES: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Effect {
    ScrollEdge {
        edge: Edge,
        style: ScrollEdgeEffectStyle,
    },
    Material {
        material: Material,
        radius: u32,
    },
    Glass {
        radius: u32,
        clear: bool,
        spacing: u32,
    },
}

// How a glass looks: SwiftUI's `Glass.clear`, and how near two glasses of one container merge.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct GlassLook {
    pub(crate) clear: bool,
    pub(crate) spacing: f32,
}

#[derive(PartialEq)]
pub(crate) struct Declared {
    pub(crate) effect: Effect,
    pub(crate) layout: LayoutData,
}

impl ElementExt for Declared {
    fn changed(&self, other: &std::rc::Rc<dyn ElementExt>) -> bool {
        !crate::style_diff(self, other).is_empty()
    }

    // The size lives in `layout`: a style-only diff leaves it at the size it was first measured.
    fn diff(&self, other: &std::rc::Rc<dyn ElementExt>) -> DiffModifies {
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

    fn render(&self, _: RenderContext) {}
}

impl Declared {
    fn scroll_edge(edge: Edge, style: ScrollEdgeEffectStyle, layout: LayoutData) -> Element {
        crate::glass::leaf(Self {
            effect: Effect::ScrollEdge { edge, style },
            layout,
        })
    }

    pub(crate) fn soft_top_edge(depth: f32) -> Element {
        Self::scroll_edge(
            Edge::Top,
            ScrollEdgeEffectStyle::Soft,
            crate::glass::layout(Size::fill(), Size::px(depth), Position::new_absolute()),
        )
    }

    pub(crate) fn bar(bar: FullscreenBar, edge: GlassEdge, layout: LayoutData) -> Option<Element> {
        let style = match bar {
            FullscreenBar::Glass => ScrollEdgeEffectStyle::Hard,
            FullscreenBar::ProgressiveGlass => ScrollEdgeEffectStyle::Soft,
            FullscreenBar::Solid => return None,
        };
        let edge = match edge {
            GlassEdge::Top => Edge::Top,
            GlassEdge::Left => Edge::Leading,
        };
        Some(Self::scroll_edge(edge, style, layout))
    }

    pub(crate) fn glass(
        radius: f32,
        (width, height): (f32, f32),
        (left, top): (f32, f32),
        look: GlassLook,
    ) -> Element {
        crate::glass::leaf(Self {
            effect: Effect::Glass {
                radius: radius.round() as u32,
                clear: look.clear,
                spacing: look.spacing.round() as u32,
            },
            layout: crate::glass::layout(
                Size::px(width),
                Size::px(height),
                Position::new_absolute().top(top).left(left),
            ),
        })
    }

    pub(crate) fn material(material: Material, radius: f32) -> Element {
        crate::glass::leaf(Self {
            effect: Effect::Material {
                material,
                radius: radius.round() as u32,
            },
            layout: crate::glass::layout(Size::fill(), Size::fill(), Position::new_absolute()),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Chrome {
    pub(crate) root: NodeId,
    pub(crate) nodes: Vec<NodeId>,
    pub(crate) area: Area,
    pub(crate) effects: Vec<(Effect, Area)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Placement {
    pub(crate) origin: (i32, i32),
    pub(crate) size: (u32, u32),
}

fn subtree(tree: &Tree, root: NodeId) -> Vec<NodeId> {
    let mut nodes = vec![root];
    let mut next = 0;
    while let Some(&node) = nodes.get(next) {
        nodes.extend(tree.children.get(&node).into_iter().flatten().copied());
        next += 1;
    }
    nodes
}

pub(crate) fn collect(tree: &Tree, window: Size2D) -> Vec<Chrome> {
    let bounds = Area::new(Point2D::zero(), window);
    let mut chromes: Vec<Chrome> = Vec::new();
    let mut alone: Vec<NodeId> = Vec::new();
    for (id, element) in &tree.elements {
        let Some(declared) = (element.as_ref() as &dyn Any).downcast_ref::<Declared>() else {
            continue;
        };
        let Some(&root) = tree.parents.get(id).filter(|root| !alone.contains(root)) else {
            continue;
        };
        let (Some(leaf), Some(holder)) = (tree.layout.get(id), tree.layout.get(&root)) else {
            continue;
        };
        if leaf.hidden || holder.hidden {
            continue;
        }
        let leaf = leaf.visible_area();
        match chromes.iter_mut().find(|chrome| chrome.root == root) {
            Some(chrome) => chrome.effects.push((declared.effect, leaf)),
            None => {
                let nodes = subtree(tree, root);
                let ancestors = tree
                    .heights
                    .get(&root)
                    .map_or(0, |height| usize::from(*height));
                if nodes.len() + ancestors >= tree.elements.len() {
                    alone.push(root);
                    continue;
                }
                chromes.push(Chrome {
                    root,
                    nodes,
                    area: holder.visible_area(),
                    effects: vec![(declared.effect, leaf)],
                });
            }
        }
    }
    chromes.sort_by_key(|chrome| std::cmp::Reverse(chrome.nodes.len()));
    let mut outer: Vec<Chrome> = Vec::new();
    for chrome in chromes {
        match outer
            .iter_mut()
            .find(|other| other.nodes.contains(&chrome.root))
        {
            Some(other) => other.effects.extend(chrome.effects),
            None => outer.push(chrome),
        }
    }
    for chrome in &mut outer {
        let reached = chrome
            .effects
            .iter()
            .fold(chrome.area, |area, (_, leaf)| area.union(leaf));
        chrome.area = reached.intersection(&bounds).unwrap_or_default();
    }
    outer.retain(|chrome| !chrome.area.is_empty());
    outer.sort_by(|a, b| {
        (a.area.min_y(), a.area.min_x())
            .partial_cmp(&(b.area.min_y(), b.area.min_x()))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    outer.truncate(MOST_CHROMES);
    outer
}

fn floor(value: f32, scale: f32) -> i32 {
    (value / scale).floor() as i32
}

fn ceil(value: f32, scale: f32) -> i32 {
    (value / scale).ceil() as i32
}

impl Chrome {
    pub(crate) fn placement(&self, scale: f32) -> Placement {
        let (x, y) = (
            floor(self.area.min_x(), scale),
            floor(self.area.min_y(), scale),
        );
        Placement {
            origin: (x, y),
            size: (
                (ceil(self.area.max_x(), scale) - x).max(1) as u32,
                (ceil(self.area.max_y(), scale) - y).max(1) as u32,
            ),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Backdrop {
    pub(crate) material: Option<WireMaterial>,
    pub(crate) radius: u32,
    pub(crate) rects: Rects,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ScrollEdge {
    pub(crate) edge: Edge,
    pub(crate) style: ScrollEdgeEffectStyle,
    pub(crate) depth: u32,
}

// One backdrop and one scroll edge per surface: the protocol keeps one of each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Requests {
    pub(crate) backdrop: Backdrop,
    pub(crate) scroll_edge: Option<ScrollEdge>,
    // How near the glasses of one chrome merge; only a platform with its own container uses it.
    pub(crate) glass_spacing: u32,
}

pub(crate) fn wire(material: Material) -> WireMaterial {
    match material {
        Material::Titlebar => WireMaterial::Titlebar,
        Material::Selection => WireMaterial::Selection,
        Material::Menu => WireMaterial::Menu,
        Material::Popover => WireMaterial::Popover,
        Material::Sidebar => WireMaterial::Sidebar,
        Material::HeaderView => WireMaterial::HeaderView,
        Material::Sheet => WireMaterial::Sheet,
        Material::WindowBackground => WireMaterial::WindowBackground,
        Material::HudWindow => WireMaterial::HudWindow,
        Material::FullScreenUi => WireMaterial::FullScreenUi,
        Material::ToolTip => WireMaterial::ToolTip,
        Material::ContentBackground => WireMaterial::ContentBackground,
        Material::UnderWindowBackground => WireMaterial::UnderWindowBackground,
        Material::UnderPageBackground => WireMaterial::UnderPageBackground,
    }
}

impl Requests {
    pub(crate) fn of(chrome: &Chrome, scale: f32) -> Self {
        let placement = chrome.placement(scale);
        let (x, y) = placement.origin;
        let mut requests = Self::default();
        for (effect, area) in &chrome.effects {
            match *effect {
                Effect::ScrollEdge { edge, style } if requests.scroll_edge.is_none() => {
                    let depth = match edge {
                        Edge::Leading | Edge::Trailing => ceil(area.max_x(), scale) - x,
                        _ => ceil(area.max_y(), scale) - y,
                    };
                    requests.scroll_edge = Some(ScrollEdge {
                        edge,
                        style,
                        depth: depth.max(0) as u32,
                    });
                }
                Effect::Material { material, radius } => {
                    requests.add_backdrop(wire(material), radius, area, (x, y), scale);
                }
                Effect::Glass {
                    radius,
                    clear,
                    spacing,
                } => {
                    let material = if clear {
                        WireMaterial::GlassClear
                    } else {
                        WireMaterial::GlassRegular
                    };
                    if requests.add_backdrop(material, radius, area, (x, y), scale) {
                        requests.glass_spacing = requests.glass_spacing.max(spacing);
                    }
                }
                Effect::ScrollEdge { .. } => {}
            }
        }
        requests
    }

    fn add_backdrop(
        &mut self,
        material: WireMaterial,
        radius: u32,
        area: &Area,
        (x, y): (i32, i32),
        scale: f32,
    ) -> bool {
        let backdrop = &mut self.backdrop;
        let same = *backdrop.material.get_or_insert(material) == material
            && (backdrop.rects.is_empty() || backdrop.radius == radius);
        if same {
            backdrop.radius = radius;
            let (left, top) = (
                floor(area.min_x(), scale) - x,
                floor(area.min_y(), scale) - y,
            );
            backdrop.rects.push((
                left,
                top,
                ceil(area.max_x(), scale) - x - left,
                ceil(area.max_y(), scale) - y - top,
            ));
        }
        same
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Pass {
    Content,
    Chrome(usize),
}

pub(crate) fn render_passes(
    tree: &mut Tree,
    chromes: &[Vec<NodeId>],
    mut render: impl FnMut(Pass, &Tree),
) {
    let members: Vec<Vec<(NodeId, i16)>> = chromes
        .iter()
        .map(|nodes| {
            nodes
                .iter()
                .filter_map(|id| Some((*id, tree.layer_state.get(id)?.layer)))
                .collect()
        })
        .collect();
    let mut all = std::mem::take(&mut tree.layers);
    for (id, layer) in members.iter().flatten() {
        all.remove_node_from_layer(id, *layer);
    }
    tree.layers = all;
    render(Pass::Content, tree);
    let mut all = std::mem::take(&mut tree.layers);
    for (index, nodes) in members.iter().enumerate() {
        for (id, layer) in nodes {
            tree.layers.insert_node_in_layer(*id, *layer);
        }
        render(Pass::Chrome(index), tree);
        tree.layers = Layers::default();
    }
    for (id, layer) in members.iter().flatten() {
        all.insert_node_in_layer(*id, *layer);
    }
    tree.layers = all;
}

pub(crate) fn render_pass(
    font_collection: &mut FontCollection,
    font_manager: &FontMgr,
    tree: &Tree,
    canvas: &Canvas,
    scale: f64,
    origin: (i32, i32),
    background: Color,
) {
    canvas.save();
    canvas.translate((
        -(origin.0 as f64 * scale) as f32,
        -(origin.1 as f64 * scale) as f32,
    ));
    crate::group::render(RenderPipeline {
        font_collection,
        font_manager,
        canvas,
        tree,
        scale_factor: scale,
        background,
    });
    canvas.restore();
}

#[cfg(test)]
mod tests {
    use crate::glass::fullscreen_backdrop;
    use freya_components::cache::AssetCacher;
    use freya_core::integration::{
        AccentColor, AnimationClock, CurrentWindowId, GlobalContexts, NavigationMode, New,
        Platform, PlatformWindow, PreferredTheme, RenderingTicker, Runner, ScreenReader, UserEvent,
        default_fonts,
    };
    use freya_core::prelude::*;
    use freya_engine::prelude::{TypefaceFontProvider, raster_n32_premul};
    use ragnarok::NodesState;
    use std::rc::Rc;

    use super::Effect;
    use super::*;
    use crate::group::{CompositingGroup, GroupEffect, ShaderEffect};

    struct Scene {
        tree: Tree,
        fonts: FontCollection,
        manager: FontMgr,
        scale: f64,
        _runner: Runner,
    }

    fn scene(app: impl Fn() -> Element + 'static, size: (f32, f32), scale: f64) -> Scene {
        let contexts = GlobalContexts::register();
        contexts.insert_context(Platform::new(|_| {}));
        let mut runner = Runner::new(app);
        runner.provide_root_context(ScreenReader::new);
        let (_sender, ticker) = RenderingTicker::new();
        runner.provide_root_context(|| ticker);
        runner.provide_root_context(AnimationClock::new);
        runner.provide_root_context(AssetCacher::create);
        let mut tree = Tree::default();
        let platform = runner.run_in(|| PlatformWindow {
            id: 0,
            focused_accessibility_id: State::create(
                freya_core::accessibility::tree::ACCESSIBILITY_ROOT_ID,
            ),
            focused_accessibility_node: State::create(accesskit::Node::new(
                accesskit::Role::Window,
            )),
            root_size: State::create(Size2D::new(size.0, size.1)),
            scale_factor: State::create(scale),
            custom_scale_factor: State::create(1.0),
            navigation_mode: State::create(NavigationMode::NotKeyboard),
            preferred_theme: State::create(PreferredTheme::Light),
            is_app_focused: State::create(true),
            accent_color: State::create(AccentColor::default()),
            sender: Rc::new(|_: UserEvent| {}),
        });
        Platform::get().register_window(platform.clone());
        runner.provide_root_context(|| CurrentWindowId(platform.id));
        runner.provide_root_context(|| tree.accessibility_generator.clone());
        let mut fonts = FontCollection::new();
        let provider = TypefaceFontProvider::new();
        let manager: FontMgr = provider.into();
        fonts.set_default_font_manager(FontMgr::default(), None);
        fonts.set_dynamic_font_manager(manager.clone());
        runner.provide_root_context({
            let fonts = fonts.clone();
            move || fonts.clone()
        });
        let mutations = runner.sync_and_update();
        runner.run_in(|| tree.apply_mutations(mutations, scale as f32));
        let (events, _receiver) = futures_channel::mpsc::unbounded();
        tree.measure_layout(
            Size2D::new(size.0 * scale as f32, size.1 * scale as f32),
            &mut fonts,
            &manager,
            &events,
            &mut NodesState::default(),
            scale,
            &default_fonts(),
        );
        Scene {
            tree,
            fonts,
            manager,
            scale,
            _runner: runner,
        }
    }

    const RED: Color = Color::RED;
    const BLUE: Color = Color::BLUE;

    fn window_with_a_bar() -> Element {
        rect()
            .expanded()
            .child(rect().expanded().background(RED))
            .child(
                rect()
                    .position(Position::new_absolute().top(0.0).left(0.0))
                    .width(Size::fill())
                    .height(Size::px(40.0))
                    .background(BLUE)
                    .child(Declared::soft_top_edge(60.0))
                    .child(
                        rect()
                            .width(Size::px(10.0))
                            .height(Size::px(10.0))
                            .background(Color::GREEN),
                    ),
            )
            .into()
    }

    fn pixel(pixels: &[u8], width: usize, (x, y): (usize, usize)) -> [u8; 4] {
        let at = (y * width + x) * 4;
        [pixels[at], pixels[at + 1], pixels[at + 2], pixels[at + 3]]
    }

    fn read(surface: &mut freya_engine::prelude::Surface, size: (i32, i32)) -> Vec<u8> {
        let info = freya_engine::prelude::ImageInfo::new(
            size,
            freya_engine::prelude::ColorType::RGBA8888,
            freya_engine::prelude::AlphaType::Premul,
            None,
        );
        let mut pixels = vec![0u8; (size.0 * size.1 * 4) as usize];
        assert!(surface.read_pixels(&info, &mut pixels, (size.0 * 4) as usize, (0, 0)));
        pixels
    }

    #[test]
    fn a_declared_leaf_makes_its_container_a_chrome_sized_by_the_leaf_and_the_container() {
        let mut scene = scene(window_with_a_bar, (200.0, 100.0), 1.0);
        let chromes = collect(&scene.tree, Size2D::new(200.0, 100.0));
        assert_eq!(chromes.len(), 1);
        let chrome = &chromes[0];
        assert_eq!(chrome.nodes.len(), 3);
        assert_eq!(
            chrome.area,
            Area::new(Point2D::new(0.0, 0.0), Size2D::new(200.0, 60.0))
        );
        assert_eq!(
            chrome.effects,
            vec![(
                Effect::ScrollEdge {
                    edge: Edge::Top,
                    style: ScrollEdgeEffectStyle::Soft
                },
                Area::new(Point2D::new(0.0, 0.0), Size2D::new(200.0, 60.0))
            )]
        );
        let _ = &mut scene;
    }

    #[test]
    fn a_declared_leaf_straight_under_the_window_root_declares_nothing() {
        let scene = scene(
            || {
                rect()
                    .expanded()
                    .child(Declared::soft_top_edge(60.0))
                    .into()
            },
            (200.0, 100.0),
            1.0,
        );
        assert!(collect(&scene.tree, Size2D::new(200.0, 100.0)).is_empty());
    }

    #[test]
    fn a_chrome_outside_the_window_is_dropped_and_one_that_overflows_it_is_cut() {
        let scene = scene(window_with_a_bar, (200.0, 50.0), 1.0);
        let chromes = collect(&scene.tree, Size2D::new(200.0, 50.0));
        assert_eq!(chromes[0].area.size, Size2D::new(200.0, 50.0));
        assert!(collect(&scene.tree, Size2D::new(0.0, 0.0)).is_empty());
    }

    #[test]
    fn nested_glass_holders_merge_into_the_outer_chrome() {
        let scene = scene(
            || {
                rect()
                    .expanded()
                    .child(rect().expanded().background(RED))
                    .child(
                        rect()
                            .position(Position::new_absolute().top(0.0))
                            .width(Size::fill())
                            .height(Size::px(40.0))
                            .child(Declared::soft_top_edge(30.0))
                            .child(
                                rect()
                                    .width(Size::px(20.0))
                                    .height(Size::px(20.0))
                                    .child(Declared::material(Material::Menu, 6.0)),
                            ),
                    )
                    .into()
            },
            (200.0, 100.0),
            1.0,
        );
        let chromes = collect(&scene.tree, Size2D::new(200.0, 100.0));
        assert_eq!(chromes.len(), 1);
        assert_eq!(chromes[0].effects.len(), 2);
    }

    #[test]
    fn the_chrome_is_drawn_apart_from_the_content_and_the_layers_come_back() {
        let mut scene = scene(window_with_a_bar, (200.0, 100.0), 1.0);
        let chromes = collect(&scene.tree, Size2D::new(200.0, 100.0));
        let layers_before = scene.tree.layers.clone();
        let mut content = raster_n32_premul((200, 100)).unwrap();
        let mut strip = raster_n32_premul((200, 60)).unwrap();
        let nodes = vec![chromes[0].nodes.clone()];
        let (mut fonts, manager, scale) = (scene.fonts.clone(), scene.manager.clone(), scene.scale);
        render_passes(&mut scene.tree, &nodes, |pass, tree| match pass {
            Pass::Content => render_pass(
                &mut fonts,
                &manager,
                tree,
                content.canvas(),
                scale,
                (0, 0),
                Color::WHITE,
            ),
            Pass::Chrome(_) => render_pass(
                &mut fonts,
                &manager,
                tree,
                strip.canvas(),
                scale,
                (0, 0),
                Color::TRANSPARENT,
            ),
        });
        assert_eq!(scene.tree.layers, layers_before);
        let content = read(&mut content, (200, 100));
        let strip = read(&mut strip, (200, 60));
        assert_eq!(
            pixel(&content, 200, (100, 20)),
            [255, 0, 0, 255],
            "red under the bar"
        );
        assert_eq!(
            pixel(&content, 200, (5, 5)),
            [255, 0, 0, 255],
            "no green mark in the content"
        );
        assert_eq!(
            pixel(&strip, 200, (100, 20)),
            [0, 0, 255, 255],
            "the blue plate"
        );
        assert_eq!(
            pixel(&strip, 200, (5, 5)),
            [0, 255, 0, 255],
            "the green mark"
        );
        assert_eq!(
            pixel(&strip, 200, (100, 50)),
            [0, 0, 0, 0],
            "clear below the plate"
        );
    }

    #[test]
    fn a_chrome_canvas_shifts_the_window_by_the_origin_of_its_strip() {
        let mut scene = scene(window_with_a_bar, (200.0, 100.0), 1.0);
        let chromes = collect(&scene.tree, Size2D::new(200.0, 100.0));
        let mut strip = raster_n32_premul((100, 60)).unwrap();
        let nodes = vec![chromes[0].nodes.clone()];
        let (mut fonts, manager, scale) = (scene.fonts.clone(), scene.manager.clone(), scene.scale);
        render_passes(&mut scene.tree, &nodes, |pass, tree| {
            if let Pass::Chrome(_) = pass {
                render_pass(
                    &mut fonts,
                    &manager,
                    tree,
                    strip.canvas(),
                    scale,
                    (100, 0),
                    Color::TRANSPARENT,
                );
            }
        });
        let strip = read(&mut strip, (100, 60));
        assert_eq!(
            pixel(&strip, 100, (5, 5)),
            [0, 0, 255, 255],
            "the mark is 100 px to the left of the strip"
        );
    }

    fn chrome(effects: Vec<(Effect, Area)>, area: Area) -> Chrome {
        Chrome {
            root: NodeId::ROOT,
            nodes: Vec::new(),
            area,
            effects,
        }
    }

    fn at(x: f32, y: f32, width: f32, height: f32) -> Area {
        Area::new(Point2D::new(x, y), Size2D::new(width, height))
    }

    fn glass(radius: u32, clear: bool, spacing: u32) -> Effect {
        Effect::Glass {
            radius,
            clear,
            spacing,
        }
    }

    #[test]
    fn a_strip_covers_the_whole_logical_pixels_its_area_touches() {
        let chrome = chrome(Vec::new(), at(10.0, 15.0, 301.0, 61.0));
        assert_eq!(
            chrome.placement(2.0),
            Placement {
                origin: (5, 7),
                size: (151, 31)
            }
        );
        assert_eq!(
            chrome.placement(1.0),
            Placement {
                origin: (10, 15),
                size: (301, 61)
            }
        );
    }

    #[test]
    fn a_top_edge_reaches_from_the_top_of_the_strip_to_the_bottom_of_the_glass() {
        let effect = Effect::ScrollEdge {
            edge: Edge::Top,
            style: ScrollEdgeEffectStyle::Soft,
        };
        let chrome = chrome(
            vec![(effect, at(0.0, 40.0, 400.0, 192.0))],
            at(0.0, 40.0, 400.0, 104.0),
        );
        let requests = Requests::of(&chrome, 2.0);
        assert_eq!(
            requests.scroll_edge,
            Some(ScrollEdge {
                edge: Edge::Top,
                style: ScrollEdgeEffectStyle::Soft,
                depth: 96
            })
        );
        assert_eq!(requests.backdrop, Backdrop::default());
    }

    #[test]
    fn a_leading_edge_reaches_across_the_width_of_the_glass_and_the_first_edge_wins() {
        let hard = |edge| Effect::ScrollEdge {
            edge,
            style: ScrollEdgeEffectStyle::Hard,
        };
        let chrome = chrome(
            vec![
                (hard(Edge::Leading), at(0.0, 0.0, 220.0, 600.0)),
                (hard(Edge::Top), at(0.0, 0.0, 800.0, 52.0)),
            ],
            at(0.0, 0.0, 220.0, 600.0),
        );
        let requests = Requests::of(&chrome, 1.0);
        assert_eq!(
            requests.scroll_edge,
            Some(ScrollEdge {
                edge: Edge::Leading,
                style: ScrollEdgeEffectStyle::Hard,
                depth: 220
            })
        );
    }

    #[test]
    fn a_material_rectangle_is_relative_to_its_strip_and_one_request_holds_one_material() {
        let material = |material, radius| Effect::Material { material, radius };
        let chrome = chrome(
            vec![
                (material(Material::Menu, 6), at(20.0, 30.0, 100.0, 50.0)),
                (material(Material::Menu, 6), at(140.0, 30.0, 60.0, 50.0)),
                (material(Material::Sheet, 6), at(0.0, 0.0, 10.0, 10.0)),
                (material(Material::Menu, 12), at(0.0, 0.0, 10.0, 10.0)),
            ],
            at(10.0, 20.0, 200.0, 100.0),
        );
        assert_eq!(
            Requests::of(&chrome, 1.0).backdrop,
            Backdrop {
                material: Some(WireMaterial::Menu),
                radius: 6,
                rects: vec![(10, 10, 100, 50), (130, 10, 60, 50)],
            }
        );
    }

    #[test]
    fn a_glass_is_a_glass_regular_backdrop_of_its_radius_and_one_request_holds_one_kind() {
        let chrome = chrome(
            vec![
                (glass(19, false, 0), at(20.0, 30.0, 76.0, 38.0)),
                (glass(19, false, 0), at(120.0, 30.0, 38.0, 38.0)),
                (
                    Effect::Material {
                        material: Material::Menu,
                        radius: 19,
                    },
                    at(0.0, 0.0, 10.0, 10.0),
                ),
            ],
            at(10.0, 20.0, 200.0, 100.0),
        );
        assert_eq!(
            Requests::of(&chrome, 1.0).backdrop,
            Backdrop {
                material: Some(WireMaterial::GlassRegular),
                radius: 19,
                rects: vec![(10, 10, 76, 38), (110, 10, 38, 38)],
            }
        );
    }

    #[test]
    fn a_clear_glass_is_a_glass_clear_backdrop() {
        let chrome = chrome(
            vec![(glass(19, true, 0), at(20.0, 30.0, 76.0, 38.0))],
            at(10.0, 20.0, 200.0, 100.0),
        );
        assert_eq!(
            Requests::of(&chrome, 1.0).backdrop.material,
            Some(WireMaterial::GlassClear)
        );
    }

    #[test]
    fn the_spacing_of_a_container_reaches_the_request_only_through_the_glasses_it_keeps() {
        let chrome = chrome(
            vec![
                (glass(19, false, 20), at(20.0, 30.0, 76.0, 38.0)),
                (glass(19, true, 40), at(120.0, 30.0, 38.0, 38.0)),
            ],
            at(10.0, 20.0, 200.0, 100.0),
        );
        let requests = Requests::of(&chrome, 1.0);
        assert_eq!(requests.glass_spacing, 20);
        assert_eq!(requests.backdrop.rects.len(), 1);
    }

    #[test]
    fn every_material_the_kit_names_reaches_the_wire_under_the_same_name_and_number() {
        use Material::*;
        let in_header_order = [
            Titlebar,
            Selection,
            Menu,
            Popover,
            Sidebar,
            HeaderView,
            Sheet,
            WindowBackground,
            HudWindow,
            FullScreenUi,
            ToolTip,
            ContentBackground,
            UnderWindowBackground,
            UnderPageBackground,
        ];
        for (number, material) in in_header_order.into_iter().enumerate() {
            assert_eq!(wire(material) as usize, number, "{material:?}");
        }
    }

    #[test]
    fn without_a_chrome_the_content_pass_draws_every_node() {
        let mut scene = scene(window_with_a_bar, (200.0, 100.0), 1.0);
        let mut content = raster_n32_premul((200, 100)).unwrap();
        let (mut fonts, manager, scale) = (scene.fonts.clone(), scene.manager.clone(), scene.scale);
        let mut passes = Vec::new();
        render_passes(&mut scene.tree, &[], |pass, tree| {
            passes.push(pass);
            render_pass(
                &mut fonts,
                &manager,
                tree,
                content.canvas(),
                scale,
                (0, 0),
                Color::WHITE,
            );
        });
        assert_eq!(passes, [Pass::Content]);
        let content = read(&mut content, (200, 100));
        assert_eq!(pixel(&content, 200, (5, 5)), [0, 255, 0, 255]);
    }

    struct Built(
        Rc<std::cell::RefCell<Option<Element>>>,
        FullscreenBar,
        GlassEdge,
    );

    impl App for Built {
        fn render(&self) -> impl IntoElement {
            *self.0.borrow_mut() = Some(fullscreen_backdrop(self.1, self.2, 44.0, 1.0));
            rect()
        }
    }

    fn declared(
        compositor: bool,
        reduced: bool,
        bar: FullscreenBar,
        edge: GlassEdge,
    ) -> Option<Effect> {
        let resolved = ctappearance::resolve(
            &ctappearance::AppearanceConfig {
                reduce_transparency: reduced,
                ..Default::default()
            },
            None,
        );
        let built = Rc::<std::cell::RefCell<Option<Element>>>::default();
        let (mut test, _) = freya_testing::TestingRunner::new(
            freya_core::integration::AppComponent::new(Built(Rc::clone(&built), bar, edge)),
            Size2D::new(200.0, 100.0),
            move |runner| {
                runner.provide_root_context(move || State::create(resolved));
                runner.provide_root_context(move || BackdropSupport(compositor));
            },
            1.0,
        );
        test.sync_and_update();
        let Some(Element::Element { element, .. }) = built.borrow().clone() else {
            return None;
        };
        (element.as_ref() as &dyn std::any::Any)
            .downcast_ref::<Declared>()
            .map(|declared| declared.effect)
    }

    #[test]
    fn a_full_screen_glass_declares_its_edge_and_style_to_a_compositor_that_composes_the_blur() {
        let edge = |edge, style| Some(Effect::ScrollEdge { edge, style });
        assert_eq!(
            declared(true, false, FullscreenBar::Glass, GlassEdge::Top),
            edge(Edge::Top, ScrollEdgeEffectStyle::Hard)
        );
        assert_eq!(
            declared(true, false, FullscreenBar::ProgressiveGlass, GlassEdge::Top),
            edge(Edge::Top, ScrollEdgeEffectStyle::Soft)
        );
        assert_eq!(
            declared(true, false, FullscreenBar::Glass, GlassEdge::Left),
            edge(Edge::Leading, ScrollEdgeEffectStyle::Hard)
        );
    }

    #[test]
    fn a_full_screen_glass_without_compositor_blur_is_the_solid() {
        for bar in [FullscreenBar::Glass, FullscreenBar::ProgressiveGlass] {
            assert_eq!(declared(false, false, bar, GlassEdge::Top), None);
            assert_eq!(declared(true, true, bar, GlassEdge::Top), None);
        }
        assert_eq!(
            declared(true, false, FullscreenBar::Solid, GlassEdge::Top),
            None
        );
    }

    fn draw(scene: &Scene, kit: bool, size: (i32, i32)) -> Vec<u8> {
        let mut surface = raster_n32_premul(size).unwrap();
        let mut fonts = scene.fonts.clone();
        if kit {
            render_pass(
                &mut fonts,
                &scene.manager,
                &scene.tree,
                surface.canvas(),
                scene.scale,
                (0, 0),
                Color::WHITE,
            );
        } else {
            RenderPipeline {
                font_collection: &mut fonts,
                font_manager: &scene.manager,
                canvas: surface.canvas(),
                tree: &scene.tree,
                scale_factor: scene.scale,
                background: Color::WHITE,
            }
            .render();
        }
        read(&mut surface, size)
    }

    fn overlapping() -> Element {
        let child = |left: f32, color| {
            rect()
                .position(Position::new_absolute().left(left).top(0.0))
                .width(Size::px(60.0))
                .height(Size::px(40.0))
                .background(color)
        };
        rect()
            .width(Size::px(100.0))
            .height(Size::px(40.0))
            .child(child(0.0, RED))
            .child(child(40.0, BLUE))
            .into()
    }

    fn grouped(effect: GroupEffect) -> impl Fn() -> Element {
        move || {
            CompositingGroup::new(effect.clone())
                .child(overlapping())
                .into_element()
        }
    }

    fn near(actual: [u8; 4], expected: [u8; 4]) {
        assert!(
            actual.iter().zip(expected).all(|(a, e)| a.abs_diff(e) <= 2),
            "{actual:?} is not {expected:?}"
        );
    }

    #[test]
    fn without_a_group_the_kit_paints_every_scene_as_freya_does() {
        for app in [window_with_a_bar as fn() -> Element, overlapping] {
            let scene = scene(app, (200.0, 100.0), 1.0);
            assert!(draw(&scene, true, (200, 100)) == draw(&scene, false, (200, 100)));
        }
    }

    #[test]
    fn an_opacity_on_a_group_shows_no_seam_where_its_children_overlap() {
        let scene = scene(grouped(GroupEffect::Opacity(0.5)), (100.0, 40.0), 1.0);
        let pixels = draw(&scene, true, (100, 40));
        near(pixel(&pixels, 100, (10, 20)), [255, 128, 128, 255]);
        near(pixel(&pixels, 100, (50, 20)), [128, 128, 255, 255]);
    }

    #[test]
    fn a_blur_on_a_group_softens_its_edge_and_leaves_its_middle() {
        let scene = scene(grouped(GroupEffect::Blur(4.0)), (100.0, 40.0), 1.0);
        let pixels = draw(&scene, true, (100, 40));
        near(pixel(&pixels, 100, (30, 20)), [255, 0, 0, 255]);
        let edge = pixel(&pixels, 100, (0, 20));
        assert!(edge[1] > 20 && edge[1] < 235, "{edge:?}");
    }

    #[test]
    fn a_color_matrix_on_a_group_recolors_what_the_children_drew() {
        let mut matrix = [0.0; 20];
        matrix[5] = 1.0;
        matrix[18] = 1.0;
        let scene = scene(
            grouped(GroupEffect::ColorMatrix(matrix)),
            (100.0, 40.0),
            1.0,
        );
        let pixels = draw(&scene, true, (100, 40));
        near(pixel(&pixels, 100, (10, 20)), [0, 255, 0, 255]);
        near(pixel(&pixels, 100, (80, 20)), [0, 0, 0, 255]);
    }

    const FADE: &str = "uniform shader content; uniform float time; half4 main(float2 p) { half4 c = content.eval(p); return half4(c.rgb * half(time), c.a); }";

    #[test]
    fn a_shader_on_a_group_reads_the_time_it_is_given() {
        let at = |time: f32| {
            let effect = ShaderEffect::new(FADE, vec![]).at(time);
            let scene = scene(grouped(GroupEffect::Shader(effect)), (100.0, 40.0), 1.0);
            pixel(&draw(&scene, true, (100, 40)), 100, (10, 20))
        };
        near(at(0.0), [0, 0, 0, 255]);
        near(at(1.0), [255, 0, 0, 255]);
        let half = at(0.5);
        assert!(half[0] > 100 && half[0] < 160, "{half:?}");
    }

    #[test]
    fn groups_nest_and_an_inner_group_paints_inside_the_outer_one() {
        let scene = scene(
            || {
                CompositingGroup::new(GroupEffect::Opacity(0.5))
                    .child(CompositingGroup::new(GroupEffect::Opacity(0.5)).child(overlapping()))
                    .into_element()
            },
            (100.0, 40.0),
            1.0,
        );
        near(
            pixel(&draw(&scene, true, (100, 40)), 100, (50, 20)),
            [191, 191, 255, 255],
        );
    }

    fn flat(level: u8) -> impl Fn() -> Element {
        move || {
            CompositingGroup::new(GroupEffect::ScrollEdge {
                depth: 20.0,
                progress: 1.0,
                dark: false,
            })
            .child(
                rect()
                    .width(Size::px(100.0))
                    .height(Size::px(60.0))
                    .background(Color::from_rgb(level, level, level)),
            )
            .into_element()
        }
    }

    // The values of macOS: a checker of two grays under the pocket of a scroll view, light appearance (probe `scrolledge`).
    #[test]
    fn a_scroll_edge_tones_flat_content_as_macos_does_and_leaves_what_is_below_its_depth() {
        for (level, expected) in [(64u8, 168.4), (128, 200.0), (191, 232.2)] {
            let scene = scene(flat(level), (100.0, 60.0), 1.0);
            let pixels = draw(&scene, true, (100, 60));
            let inside = pixel(&pixels, 100, (50, 8))[0] as f32;
            assert!((inside - expected).abs() <= 2.0, "{level}: {inside}");
            near(pixel(&pixels, 100, (50, 40)), [level, level, level, 255]);
        }
    }

    #[test]
    fn a_scroll_edge_with_no_progress_shows_the_content_and_with_all_of_it_the_pocket() {
        let at = |progress| {
            let scene = scene(
                move || {
                    CompositingGroup::new(GroupEffect::ScrollEdge {
                        depth: 20.0,
                        progress,
                        dark: false,
                    })
                    .child(
                        rect()
                            .width(Size::px(100.0))
                            .height(Size::px(60.0))
                            .background(Color::from_rgb(128, 128, 128)),
                    )
                    .into_element()
                },
                (100.0, 60.0),
                1.0,
            );
            pixel(&draw(&scene, true, (100, 60)), 100, (50, 8))[0] as f32
        };
        assert!((at(0.0) - 128.0).abs() <= 1.5);
        assert!((at(0.5) - 164.0).abs() <= 2.5);
        assert!((at(1.0) - 200.0).abs() <= 1.5);
    }

    // macOS blurs the 8 px checker of the probe to a deviation of 7.9 levels (rows 56 to 88 of the pocket, light).
    #[test]
    fn a_scroll_edge_blurs_a_checker_to_the_deviation_macos_leaves() {
        let scene = scene(
            || {
                let mut cells = rect().width(Size::px(104.0)).height(Size::px(56.0));
                for row in 0..7 {
                    for col in 0..13 {
                        let level = if (row + col) % 2 == 0 { 255 } else { 0 };
                        cells = cells.child(
                            rect()
                                .position(
                                    Position::new_absolute()
                                        .left(col as f32 * 8.0)
                                        .top(row as f32 * 8.0),
                                )
                                .width(Size::px(8.0))
                                .height(Size::px(8.0))
                                .background(Color::from_rgb(level, level, level)),
                        );
                    }
                }
                CompositingGroup::new(GroupEffect::ScrollEdge {
                    depth: 56.0,
                    progress: 1.0,
                    dark: false,
                })
                .child(cells)
                .into_element()
            },
            (104.0, 56.0),
            1.0,
        );
        let pixels = draw(&scene, true, (104, 56));
        let mut deviations = Vec::new();
        for y in 16..40 {
            let row: Vec<f32> = (16..88)
                .map(|x| pixel(&pixels, 104, (x, y))[0] as f32)
                .collect();
            let mean = row.iter().sum::<f32>() / row.len() as f32;
            deviations.push(
                (row.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / row.len() as f32).sqrt(),
            );
        }
        let deviation = deviations.iter().sum::<f32>() / deviations.len() as f32;
        assert!((deviation - 7.9).abs() <= 2.0, "{deviation}");
    }

    // A headless Vulkan device (Mesa's lavapipe in the container) draws the groups with the GPU backend the Linux runner uses, and the
    // pixels match the CPU raster within 10 levels (the GPU and CPU Gaussians differ in kernel). Run with `--ignored`; it needs a Vulkan loader and an ICD (`VK_ICD_FILENAMES`).
    #[cfg(target_os = "linux")]
    mod vulkan {
        use ash::{
            Entry,
            vk::{self as ash_vk, Handle},
        };
        use freya_engine::prelude::{DirectContext, SurfaceOrigin, direct_contexts, vk};
        use skia_safe::gpu::{Budgeted, surfaces};

        use super::*;

        fn context() -> DirectContext {
            // SAFETY: libvulkan is the system loader; its initialisers have no preconditions.
            let entry = Box::leak(Box::new(unsafe { Entry::load() }.expect("libvulkan")));
            let app = ash_vk::ApplicationInfo::default().api_version(ash_vk::API_VERSION_1_1);
            // SAFETY: the create info only borrows locals that outlive the call.
            let instance = Box::leak(Box::new(
                unsafe {
                    entry.create_instance(
                        &ash_vk::InstanceCreateInfo::default().application_info(&app),
                        None,
                    )
                }
                .expect("vkCreateInstance"),
            ));
            // SAFETY: `instance` is alive.
            let (device, family) = unsafe { instance.enumerate_physical_devices() }
                .unwrap()
                .into_iter()
                .find_map(|device| {
                    // SAFETY: `device` was enumerated from `instance`.
                    let families =
                        unsafe { instance.get_physical_device_queue_family_properties(device) };
                    let family = families
                        .iter()
                        .position(|f| f.queue_flags.contains(ash_vk::QueueFlags::GRAPHICS))?;
                    Some((device, family as u32))
                })
                .expect("a Vulkan device");
            let priorities = [1.0];
            let queue_info = ash_vk::DeviceQueueCreateInfo::default()
                .queue_family_index(family)
                .queue_priorities(&priorities);
            // SAFETY: `device` comes from `instance`.
            let logical = Box::leak(Box::new(
                unsafe {
                    instance.create_device(
                        device,
                        &ash_vk::DeviceCreateInfo::default()
                            .queue_create_infos(std::slice::from_ref(&queue_info)),
                        None,
                    )
                }
                .expect("vkCreateDevice"),
            ));
            // SAFETY: the device was created with one queue in `family`.
            let queue = unsafe { logical.get_device_queue(family, 0) };
            let get_proc = |of: vk::GetProcOf| {
                // SAFETY: Skia asks only for handles it received from this instance and device.
                unsafe {
                    match of {
                        vk::GetProcOf::Instance(handle, name) => entry
                            .get_instance_proc_addr(ash_vk::Instance::from_raw(handle as _), name),
                        vk::GetProcOf::Device(handle, name) => {
                            (instance.fp_v1_0().get_device_proc_addr)(
                                ash_vk::Device::from_raw(handle as _),
                                name,
                            )
                        }
                    }
                }
                .map_or(std::ptr::null(), |f| f as _)
            };
            let builder = vk::BackendContext::new_builder(
                instance.handle().as_raw() as _,
                device.as_raw() as _,
                logical.handle().as_raw() as _,
                (queue.as_raw() as _, family as usize),
                &get_proc,
                Some(vk::Version::new(1, 1, 0)),
            );
            // SAFETY: instance, device and queue are leaked, so they outlive the Skia context.
            let backend = unsafe { builder.build() };
            direct_contexts::make_vulkan(&backend, None).expect("a Skia Vulkan context")
        }

        fn gpu_pixels(scene: &Scene, size: (i32, i32)) -> Vec<u8> {
            let mut context = context();
            let info = freya_engine::prelude::ImageInfo::new_n32_premul(size, None);
            let mut surface = surfaces::render_target(
                &mut context,
                Budgeted::No,
                &info,
                None,
                SurfaceOrigin::TopLeft,
                None,
                false,
                false,
            )
            .expect("a GPU surface");
            let mut fonts = scene.fonts.clone();
            render_pass(
                &mut fonts,
                &scene.manager,
                &scene.tree,
                surface.canvas(),
                scene.scale,
                (0, 0),
                Color::WHITE,
            );
            context.flush_and_submit();
            read(&mut surface, size)
        }

        #[test]
        #[ignore = "needs a Vulkan device"]
        fn groups_draw_on_vulkan_as_on_the_raster() {
            let effects = [
                GroupEffect::Opacity(0.5),
                GroupEffect::Shader(ShaderEffect::new(FADE, vec![]).at(1.0)),
                GroupEffect::Blur(4.0),
                GroupEffect::ScrollEdge {
                    depth: 30.0,
                    progress: 1.0,
                    dark: false,
                },
            ];
            for effect in effects {
                let scene = scene(grouped(effect.clone()), (100.0, 40.0), 1.0);
                let cpu = draw(&scene, true, (100, 40));
                let gpu = gpu_pixels(&scene, (100, 40));
                let worst = cpu
                    .iter()
                    .zip(&gpu)
                    .map(|(a, b)| a.abs_diff(*b))
                    .max()
                    .unwrap();
                assert!(worst <= 10, "{effect:?}: {worst}");
            }
        }
    }
}

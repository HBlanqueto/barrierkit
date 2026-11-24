use std::any::Any;
use std::borrow::Cow;
use std::rc::Rc;

use ctappearance::material::Material;
use ctdesign::Rgba;
use freya_core::data::LayoutData;
use freya_core::element::{Element, ElementExt, RenderContext};
use freya_core::integration::RenderPipeline;
use freya_core::layers::Layers;
use freya_core::node_id::NodeId;
use freya_core::prelude::Color;
use freya_core::tree::{DiffModifies, Tree};
use freya_engine::prelude::{Canvas, FontCollection, FontMgr};
use torin::prelude::{Area, Point2D, Position, Size, Size2D};

use super::glass::Rects;
use super::window_style::{Edge, ScrollEdgeEffectStyle, WireMaterial};
use super::{FullscreenBar, MOST_CHROMES};

#[derive(Clone, Copy)]
pub(crate) struct BackdropSupport(pub(crate) bool);

#[derive(Clone, Copy)]
pub(crate) struct NativeGlass(pub(crate) bool);

#[derive(Clone, Copy)]
pub(crate) struct NativeSheets(pub(crate) bool);

#[derive(Clone, Copy)]
pub(crate) struct NativePopovers(pub(crate) bool);

#[derive(Clone, Copy)]
pub(crate) struct SystemWindowBackground(pub(crate) bool);

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
        tint: Option<Rgba>,
    },
    // The platform's own pane glass lies under the container, so the content beside it shows through the glass.
    Pane,
    // The platform's own title bar lies under this strip: AppKit's titlebar material and separator.
    Titlebar,
    // Nothing native lies under it; the layer of its own lets the title ride with the full-screen bar.
    Layer,
}

// How a glass looks: SwiftUI's `Glass.clear` and `Glass.tint`, and how near two glasses of one container merge.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct GlassLook {
    pub(crate) clear: bool,
    pub(crate) spacing: f32,
    pub(crate) tint: Option<Rgba>,
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

    pub(crate) fn bar(bar: FullscreenBar, layout: LayoutData) -> Option<Element> {
        let style = match bar {
            FullscreenBar::Glass => ScrollEdgeEffectStyle::Hard,
            FullscreenBar::ProgressiveGlass => ScrollEdgeEffectStyle::Soft,
            FullscreenBar::Solid => return None,
        };
        Some(Self::scroll_edge(Edge::Top, style, layout))
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
                tint: look.tint,
            },
            layout: crate::glass::layout(
                Size::px(width),
                Size::px(height),
                Position::new_absolute().top(top).left(left),
            ),
        })
    }

    pub(crate) fn pane() -> Element {
        crate::glass::leaf(Self {
            effect: Effect::Pane,
            layout: crate::glass::layout(Size::fill(), Size::fill(), Position::new_absolute()),
        })
    }

    pub(crate) fn titlebar(height: f32) -> Element {
        crate::glass::leaf(Self {
            effect: Effect::Titlebar,
            layout: crate::glass::layout(Size::fill(), Size::px(height), Position::new_absolute()),
        })
    }

    pub(crate) fn layer() -> Element {
        crate::glass::leaf(Self {
            effect: Effect::Layer,
            layout: crate::glass::layout(Size::px(0.0), Size::px(0.0), Position::new_absolute()),
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
    pub(crate) nodes: Rc<[NodeId]>,
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
                    nodes: nodes.into(),
                    area: holder.visible_area(),
                    effects: vec![(declared.effect, leaf)],
                });
            }
        }
    }
    chromes.sort_by_key(|chrome| std::cmp::Reverse(chrome.nodes.len()));
    let mut outer: Vec<Chrome> = Vec::new();
    for chrome in chromes {
        let ancestors =
            std::iter::successors(tree.parents.get(&chrome.root), |id| tree.parents.get(id));
        match outer
            .iter_mut()
            .find(|other| ancestors.clone().any(|ancestor| *ancestor == other.root))
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
    // A pane's chrome goes first, so that every other chrome lies above its glass.
    let not_pane = |chrome: &Chrome| {
        !chrome
            .effects
            .iter()
            .any(|(effect, _)| *effect == Effect::Pane)
    };
    outer.sort_by(|a, b| {
        (not_pane(a), a.area.min_y(), a.area.min_x())
            .partial_cmp(&(not_pane(b), b.area.min_y(), b.area.min_x()))
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
                    ..
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
                Effect::ScrollEdge { .. } | Effect::Pane | Effect::Titlebar | Effect::Layer => {}
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
            backdrop.rects.push(strip_rect(area, (x, y), scale));
        }
        same
    }
}

pub(super) fn strip_rect(area: &Area, (x, y): (i32, i32), scale: f32) -> (i32, i32, i32, i32) {
    let (left, top) = (
        floor(area.min_x(), scale) - x,
        floor(area.min_y(), scale) - y,
    );
    (
        left,
        top,
        ceil(area.max_x(), scale) - x - left,
        ceil(area.max_y(), scale) - y - top,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Pass {
    Content,
    Chrome(usize),
}

pub(crate) fn render_passes(
    tree: &mut Tree,
    chromes: &[Rc<[NodeId]>],
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

//! P1 of `docs/research/shader-needs.md` (ADR 0103): a `CompositingGroup` draws its child subtree into one layer and runs an effect
//! over the layer: an opacity, a blur, a color matrix or an SkSL image filter. The kit calls `render` instead of Freya's
//! `RenderPipeline`, which paints one node at a time and cannot hold a subtree; a scene without a group paints exactly as before.

use std::any::Any;
use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;
use std::time::Instant;

use freya_animation::prelude::{AnimNum, OnCreation, OnFinish, use_animation};
use freya_core::data::LayoutData;
use freya_core::element::{ClipContext, ElementExt, RenderContext};
use freya_core::integration::RenderPipeline;
use freya_core::layers::Layer;
use freya_core::node_id::NodeId;
use freya_core::prelude::*;
use freya_core::tree::{DiffModifies, Tree};
use freya_engine::prelude::{
    ClipOp, Paint, Rect as SkRect, SaveLayerRec, SkMatrix, SkPoint, TileMode, blur, color_filters,
};
use torin::prelude::{Area, Position, Size};

use crate::shader::Program;

/// What a group does to its layer.
#[derive(Clone, Debug, PartialEq)]
pub enum GroupEffect {
    /// The layer at this opacity, so overlapping children show no seam.
    Opacity(f32),
    /// A Gaussian blur of this radius in points.
    Blur(f32),
    /// A 4x5 color matrix in row-major order, as Skia reads it (`translate` column in 0 to 1).
    ColorMatrix([f32; 20]),
    /// An SkSL image filter: `uniform shader content;` receives the layer; a uniform named `time` is bound to the seconds since the
    /// group appeared, added to its starting time; the group redraws every frame.
    Shader(ShaderEffect),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShaderEffect {
    source: &'static str,
    uniforms: Vec<(&'static str, Vec<f32>)>,
    time: f32,
}

impl ShaderEffect {
    pub fn new(source: &'static str, uniforms: Vec<(&'static str, Vec<f32>)>) -> Self {
        Self {
            source,
            uniforms,
            time: 0.0,
        }
    }
}

impl ShaderEffect {
    pub(crate) fn at(mut self, time: f32) -> Self {
        self.time = time;
        self
    }
}

impl GroupEffect {
    fn uses_time(&self) -> bool {
        match self {
            Self::Shader(effect) => {
                Program::get(effect.source).is_some_and(|program| program.has_uniform("time"))
            }
            _ => false,
        }
    }

    // The paint of the layer: the group's own effect and the opacity it inherits.
    fn paint(&self, inherited: f32, scale: f32) -> Paint {
        let mut paint = Paint::default();
        let mut alpha = inherited;
        match self {
            Self::Opacity(opacity) => alpha *= opacity.clamp(0.0, 1.0),
            Self::Blur(radius) => {
                let sigma = radius * scale;
                paint.set_image_filter(blur((sigma, sigma), TileMode::Decal, None, None));
            }
            Self::ColorMatrix(matrix) => {
                paint.set_color_filter(color_filters::matrix_row_major(
                    matrix,
                    color_filters::Clamp::Yes,
                ));
            }
            Self::Shader(effect) => {
                if let Some(program) = Program::get(effect.source) {
                    let mut arguments: Vec<(&str, &[f32])> = effect
                        .uniforms
                        .iter()
                        .map(|(name, values)| (*name, values.as_slice()))
                        .collect();
                    let time = [effect.time];
                    if program.has_uniform("time") {
                        arguments.push(("time", &time));
                    }
                    paint.set_image_filter(program.filter(&arguments, "content"));
                }
            }
        }
        paint.set_alpha_f(alpha);
        paint
    }

    // How far the effect can reach outside the children, in points.
    fn outset(&self) -> f32 {
        match self {
            Self::Blur(radius) => radius * 3.0,
            _ => 0.0,
        }
    }
}

// The element that marks its parent as a group root. It draws nothing and takes no room.
#[derive(Clone, PartialEq)]
struct Marker {
    effect: GroupEffect,
    layout: LayoutData,
}

impl ElementExt for Marker {
    fn changed(&self, other: &Rc<dyn ElementExt>) -> bool {
        !crate::style_diff(self, other).is_empty()
    }

    fn diff(&self, other: &Rc<dyn ElementExt>) -> DiffModifies {
        crate::style_diff(self, other)
    }

    fn layout(&'_ self) -> Cow<'_, LayoutData> {
        Cow::Borrowed(&self.layout)
    }

    fn render(&self, _: RenderContext) {}
}

fn marker(effect: GroupEffect) -> Element {
    crate::glass::leaf(Marker {
        effect,
        layout: crate::glass::layout(Size::px(0.0), Size::px(0.0), Position::new_absolute()),
    })
}

/// SwiftUI's `compositingGroup()` with an effect: the children are drawn into one layer and the effect runs over it.
#[derive(PartialEq)]
pub struct CompositingGroup {
    effect: GroupEffect,
    children: Vec<Element>,
}

impl CompositingGroup {
    pub fn new(effect: GroupEffect) -> Self {
        Self {
            effect,
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_element());
        self
    }
}

impl Component for CompositingGroup {
    fn render(&self) -> impl IntoElement {
        let started = use_hook(Instant::now);
        let moving = self.effect.uses_time();
        let clock = use_animation(move |config| {
            config.on_creation(if moving {
                OnCreation::Run
            } else {
                OnCreation::Nothing
            });
            config.on_finish(OnFinish::restart());
            AnimNum::new(0.0, 1.0).time(1000)
        });
        let _ = clock.get().value();
        let mut effect = self.effect.clone();
        if let GroupEffect::Shader(shader) = &mut effect {
            *shader = shader
                .clone()
                .at(shader.time + started.elapsed().as_secs_f32());
        }
        rect().child(marker(effect)).children(self.children.clone())
    }
}

// The nodes under `root`, root first, that a group paints itself: not an overlay and what hangs from it.
fn members<N: Copy + Eq + Hash>(
    root: N,
    children: &dyn Fn(N) -> Vec<N>,
    outside: &dyn Fn(N) -> bool,
) -> Vec<N> {
    let mut found = vec![root];
    let mut next = 0;
    while next < found.len() {
        for child in children(found[next]) {
            if !outside(child) {
                found.push(child);
            }
        }
        next += 1;
    }
    found
}

// What a node applies on top of what its group already applied: the entries of an inherited list after the group's own.
fn beyond<T>(list: &[T], applied: usize) -> &[T] {
    list.get(applied..).unwrap_or(&[])
}

// How much of each inherited list the enclosing groups already applied to the canvas.
#[derive(Clone, Copy, Default)]
struct Applied {
    clips: usize,
    rotations: usize,
    scales: usize,
    opacities: usize,
}

struct Groups {
    effects: HashMap<NodeId, GroupEffect>,
    owner: HashMap<NodeId, NodeId>,
}

impl Groups {
    fn find(tree: &Tree) -> Self {
        let mut effects = HashMap::default();
        for (id, element) in &tree.elements {
            if let Some(marker) = (element.as_ref() as &dyn Any).downcast_ref::<Marker>()
                && let Some(parent) = tree.parents.get(id)
            {
                effects.insert(*parent, marker.effect.clone());
            }
        }
        let mut roots: Vec<NodeId> = effects.keys().copied().collect();
        roots.sort_by_key(|root| tree.heights.get(root).copied().unwrap_or(0));
        let children = |id: NodeId| tree.children.get(&id).cloned().unwrap_or_default();
        let outside = |id: NodeId| {
            tree.elements
                .get(&id)
                .is_some_and(|element| !matches!(element.layer(), Layer::Relative(_)))
        };
        let mut owner = HashMap::default();
        for root in roots {
            for member in members(root, &children, &outside) {
                owner.insert(member, root);
            }
        }
        Self { effects, owner }
    }
}

struct Painter<'a, 'b> {
    pipeline: &'a mut RenderPipeline<'b>,
    groups: Groups,
    order: Vec<NodeId>,
    clip_areas: HashMap<NodeId, Area>,
}

impl Painter<'_, '_> {
    fn scale_transformed_area(&self, mut area: Area, scale_node_ids: &[NodeId]) -> Area {
        for node_id in scale_node_ids {
            let layout_node = self.pipeline.tree.layout.get(node_id).unwrap();
            let effect = self.pipeline.tree.effect_state.get(node_id).unwrap();
            let node_area = layout_node.visible_area();
            let origin = effect.transform_origin.origin(&node_area);
            let scale = effect.scale.unwrap();
            area = area.translate(-origin.to_vector());
            area = area.scale(scale.x, scale.y);
            area = area.translate(origin.to_vector());
        }
        area
    }

    fn clip_area(&mut self, clip_node_id: NodeId) -> Area {
        if let Some(area) = self.clip_areas.get(&clip_node_id) {
            return *area;
        }
        let tree = self.pipeline.tree;
        let layout = tree.layout.get(&clip_node_id).unwrap();
        let effect = tree.effect_state.get(&clip_node_id).unwrap();
        let area = self.scale_transformed_area(layout.visible_area(), &effect.scales);
        self.clip_areas.insert(clip_node_id, area);
        area
    }

    // The clips, rotations and scales of a node beyond what `applied` covers, as Freya's loop applies them.
    fn apply_transforms(&mut self, node_id: NodeId, applied: Applied) {
        let tree = self.pipeline.tree;
        let Some(effect_state) = tree.effect_state.get(&node_id) else {
            return;
        };
        for clip_node_id in beyond(&effect_state.clips, applied.clips) {
            let area = self.clip_area(*clip_node_id);
            let clip_element = tree.elements.get(clip_node_id).unwrap();
            clip_element.clip(ClipContext {
                canvas: self.pipeline.canvas,
                visible_area: &area,
                scale_factor: self.pipeline.scale_factor,
            });
        }
        for id in beyond(&effect_state.rotations, applied.rotations) {
            let layout_node = tree.layout.get(id).unwrap();
            let effect = tree.effect_state.get(id).unwrap();
            let origin = effect.transform_origin.origin(&layout_node.visible_area());
            let mut matrix = SkMatrix::new_identity();
            matrix.set_rotate(
                effect.rotation.unwrap(),
                Some(SkPoint {
                    x: origin.x,
                    y: origin.y,
                }),
            );
            self.pipeline.canvas.concat(&matrix);
        }
    }

    fn apply_scales(&self, node_id: NodeId, applied: Applied) {
        let tree = self.pipeline.tree;
        let Some(effect_state) = tree.effect_state.get(&node_id) else {
            return;
        };
        for id in beyond(&effect_state.scales, applied.scales) {
            let layout_node = tree.layout.get(id).unwrap();
            let effect = tree.effect_state.get(id).unwrap();
            let origin = effect.transform_origin.origin(&layout_node.visible_area());
            let scale = effect.scale.unwrap();
            self.pipeline.canvas.translate((origin.x, origin.y));
            self.pipeline.canvas.scale((scale.x, scale.y));
            self.pipeline.canvas.translate((-origin.x, -origin.y));
        }
    }

    fn culled(&mut self, node_id: NodeId) -> bool {
        let tree = self.pipeline.tree;
        let Some(layout_node) = tree.layout.get(&node_id) else {
            return true;
        };
        if layout_node.hidden {
            return true;
        }
        let Some(effect_state) = tree.effect_state.get(&node_id) else {
            return false;
        };
        let visible_area =
            self.scale_transformed_area(layout_node.visible_area(), &effect_state.scales);
        for clip_node_id in effect_state.clips.iter() {
            if !visible_area.intersects(&self.clip_area(*clip_node_id)) {
                return true;
            }
        }
        false
    }

    fn paint_node(&mut self, node_id: NodeId, applied: Applied) {
        if self.culled(node_id) {
            return;
        }
        let tree = self.pipeline.tree;
        let canvas = self.pipeline.canvas;
        let layout_node = tree.layout.get(&node_id).unwrap();
        let effect_state = tree.effect_state.get(&node_id);
        let visible_area = effect_state.map_or(layout_node.visible_area(), |effect| {
            self.scale_transformed_area(layout_node.visible_area(), &effect.scales)
        });
        let saved = canvas.save();
        let element = tree.elements.get(&node_id).unwrap();
        let text_style_state = tree.text_style_state.get(&node_id).unwrap();
        if let Some(effect_state) = effect_state {
            self.apply_transforms(node_id, applied);
            let scale_factor = self.pipeline.scale_factor as f32;
            let render_rect = element.render_rect(&visible_area, scale_factor);
            // The shadows' reach is private to Freya, so a node with one takes an unbounded layer.
            let layer_bounds = element
                .style()
                .shadows
                .is_empty()
                .then(|| *render_rect.rect());
            if let Some(blur_radius) = effect_state.blur {
                let style = element.style();
                let image_filter = blur(
                    (blur_radius * scale_factor, blur_radius * scale_factor),
                    None,
                    None,
                    render_rect.rect(),
                );
                if let Some(image_filter) = image_filter {
                    let rec = SaveLayerRec::default()
                        .bounds(render_rect.rect())
                        .backdrop(&image_filter);
                    let blur_layer = canvas.save();
                    if style.corner_radius.is_round() {
                        canvas.clip_rrect(render_rect, ClipOp::Intersect, true);
                    }
                    canvas.save_layer(&rec);
                    canvas.restore_to_count(blur_layer);
                }
            }
            let opacity = beyond(&effect_state.opacities, applied.opacities)
                .iter()
                .fold(1., |acc, opacity| acc * opacity);
            if opacity < 1. {
                canvas.save_layer_alpha_f(layer_bounds, opacity);
            }
            self.apply_scales(node_id, applied);
        }
        element.render(RenderContext {
            font_collection: self.pipeline.font_collection,
            canvas,
            layout_node,
            text_style_state,
            scale_factor: self.pipeline.scale_factor,
        });
        canvas.restore_to_count(saved);
    }

    fn paint_group(&mut self, root: NodeId, applied: Applied) {
        let Some(effect) = self.groups.effects.get(&root).cloned() else {
            return;
        };
        if self.culled(root) {
            return;
        }
        let tree = self.pipeline.tree;
        let canvas = self.pipeline.canvas;
        let state = tree.effect_state.get(&root);
        let inside = Applied {
            clips: state.map_or(0, |state| state.clips.len()),
            rotations: state.map_or(0, |state| state.rotations.len()),
            scales: state.map_or(0, |state| state.scales.len()),
            opacities: state.map_or(0, |state| state.opacities.len()),
        };
        let members: Vec<NodeId> = self
            .order
            .iter()
            .copied()
            .filter(|id| self.groups.owner.get(id) == Some(&root) || self.is_inner_root(*id, root))
            .collect();
        let saved = canvas.save();
        self.apply_transforms(root, applied);
        self.apply_scales(root, applied);
        let scale_factor = self.pipeline.scale_factor as f32;
        let inherited = state.map_or(1., |state| {
            beyond(&state.opacities, applied.opacities)
                .iter()
                .fold(1., |acc, opacity| acc * opacity)
        });
        let paint = effect.paint(inherited, scale_factor);
        let reach = effect.outset() * scale_factor;
        let bounds = (inside.rotations == applied.rotations && inside.scales == applied.scales)
            .then(|| self.union(&members, reach))
            .flatten();
        let layer = match &bounds {
            Some(bounds) => SaveLayerRec::default().bounds(bounds).paint(&paint),
            None => SaveLayerRec::default().paint(&paint),
        };
        canvas.save_layer(&layer);
        for member in members {
            if member == root {
                self.paint_node(root, inside);
            } else if self.groups.effects.contains_key(&member) {
                self.paint_group(member, inside);
            } else {
                self.paint_node(member, inside);
            }
        }
        canvas.restore_to_count(saved);
    }

    fn is_inner_root(&self, id: NodeId, outer: NodeId) -> bool {
        self.groups.effects.contains_key(&id)
            && id != outer
            && self
                .pipeline
                .tree
                .parents
                .get(&id)
                .and_then(|parent| self.groups.owner.get(parent))
                == Some(&outer)
    }

    fn union(&self, members: &[NodeId], reach: f32) -> Option<SkRect> {
        let tree = self.pipeline.tree;
        let mut areas = members
            .iter()
            .filter_map(|id| tree.layout.get(id))
            .map(|node| node.visible_area());
        let first = areas.next()?;
        let all = areas.fold(first, |union, area| union.union(&area));
        Some(SkRect::new(
            all.min_x() - reach,
            all.min_y() - reach,
            all.max_x() + reach,
            all.max_y() + reach,
        ))
    }
}

/// Freya's `RenderPipeline::render` with the groups of this module: every node paints as there, and a group paints its subtree into
/// one layer at the place of its root.
pub(crate) fn render(mut pipeline: RenderPipeline<'_>) {
    pipeline.canvas.clear(pipeline.background);
    let groups = Groups::find(pipeline.tree);
    let mut keys: Vec<i16> = pipeline.tree.layers.keys().copied().collect();
    keys.sort_unstable();
    let order: Vec<NodeId> = keys
        .iter()
        .flat_map(|key| pipeline.tree.layers.get(key).unwrap().iter().copied())
        .collect();
    let mut painter = Painter {
        pipeline: &mut pipeline,
        groups,
        order: order.clone(),
        clip_areas: HashMap::default(),
    };
    for node_id in order {
        match painter.groups.owner.get(&node_id) {
            None => painter.paint_node(node_id, Applied::default()),
            Some(root) if *root == node_id => {
                if !painter.is_nested(node_id) {
                    painter.paint_group(node_id, Applied::default());
                }
            }
            Some(_) => {}
        }
    }
}

impl Painter<'_, '_> {
    // A group root that hangs from another group paints with it.
    fn is_nested(&self, root: NodeId) -> bool {
        self.pipeline
            .tree
            .parents
            .get(&root)
            .is_some_and(|parent| self.groups.owner.contains_key(parent))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(id: u32) -> Vec<u32> {
        match id {
            1 => vec![2, 3],
            2 => vec![4],
            3 => vec![5],
            _ => vec![],
        }
    }

    #[test]
    fn a_group_holds_its_root_and_every_descendant_in_order() {
        let found = members(1, &|id| tree(id), &|_| false);
        assert_eq!(found, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn an_overlay_and_what_hangs_from_it_stay_out_of_the_group() {
        let found = members(1, &|id| tree(id), &|id| id == 3);
        assert_eq!(found, [1, 2, 4]);
    }

    #[test]
    fn a_node_applies_only_what_its_group_did_not() {
        assert_eq!(beyond(&[1, 2, 3, 4], 2), [3, 4]);
        assert!(beyond(&[1, 2], 2).is_empty());
        assert!(beyond(&[1, 2], 5).is_empty());
    }

    #[test]
    fn the_layer_carries_the_inherited_opacity_and_the_opacity_of_its_effect() {
        let paint = GroupEffect::Opacity(0.5).paint(0.5, 1.0);
        assert!((paint.alpha_f() - 0.25).abs() < 1e-6);
        assert!((GroupEffect::Blur(4.0).paint(1.0, 2.0).alpha_f() - 1.0).abs() < 1e-6);
        assert!(
            GroupEffect::Blur(4.0)
                .paint(1.0, 2.0)
                .image_filter()
                .is_some()
        );
    }
}

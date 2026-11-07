use std::borrow::Cow;
use std::rc::Rc;
use std::time::Duration;

use ctdesign::{DURATION_BASE, MEDIA, Rgba};
use freya_animation::prelude::use_animation_transition;
use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{
    BlendMode, Data, Paint, Rect as SkRect, RuntimeEffect, SaveLayerRec, TileMode, blur,
};
use torin::prelude::{Area, Direction, Position, Size};

use crate::animation::Spring;
use crate::glass::{layout, leaf, material, transparency_reduced};

pub const MAX_SHAPES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub radius: f32,
}

impl Frame {
    fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn lerp(&self, to: &Frame, t: f32) -> Frame {
        let mix = |a: f32, b: f32| a + (b - a) * t;
        Frame {
            x: mix(self.x, to.x),
            y: mix(self.y, to.y),
            width: mix(self.width, to.width).max(0.0),
            height: mix(self.height, to.height).max(0.0),
            radius: mix(self.radius, to.radius).max(0.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Morph {
    pub id: &'static str,
    pub from: Frame,
    pub to: Frame,
}

fn nearest(frames: &[(&'static str, Frame)], of: &Frame) -> Option<Frame> {
    let (cx, cy) = of.center();
    frames.iter().map(|(_, frame)| *frame).min_by(|a, b| {
        let distance = |f: &Frame| {
            let (x, y) = f.center();
            (x - cx).powi(2) + (y - cy).powi(2)
        };
        distance(a).total_cmp(&distance(b))
    })
}

pub fn plan(shown: &[(&'static str, Frame)], target: &[(&'static str, Frame)]) -> Vec<Morph> {
    let mut morphs: Vec<Morph> = target
        .iter()
        .map(|(id, to)| {
            let from = shown
                .iter()
                .find(|(shown_id, _)| shown_id == id)
                .map(|(_, frame)| *frame)
                .or_else(|| nearest(shown, to))
                .unwrap_or(*to);
            Morph { id, from, to: *to }
        })
        .collect();
    for (id, from) in shown {
        if target.iter().all(|(target_id, _)| target_id != id)
            && let Some(to) = nearest(target, from)
        {
            morphs.push(Morph {
                id,
                from: *from,
                to,
            });
        }
    }
    morphs
}

// Smooth union of rounded-box distances (Inigo Quilez, https://iquilezles.org/articles/smin/).
const SKSL: &str = r"
uniform float4 frames[8];
uniform float4 radii[2];
uniform float count;
uniform float k;
uniform float pixel;
uniform float mode;
uniform half4 tint;
uniform half4 rim;

float radius_at(int i) {
    float4 r = i < 4 ? radii[0] : radii[1];
    int j = i < 4 ? i : i - 4;
    return j == 0 ? r.x : j == 1 ? r.y : j == 2 ? r.z : r.w;
}

float rounded_box(float2 p, float4 f, float r) {
    float2 half_size = f.zw * 0.5;
    float2 q = abs(p - (f.xy + half_size)) - half_size + r;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}

float smooth_min(float a, float b) {
    float h = max(k - abs(a - b), 0.0) / k;
    return min(a, b) - h * h * k * 0.25;
}

half4 main(float2 p) {
    float d = 100000.0;
    for (int i = 0; i < 8; i++) {
        if (float(i) >= count) { break; }
        d = smooth_min(d, rounded_box(p, frames[i], radius_at(i)));
    }
    float inside = clamp(0.5 - d / pixel, 0.0, 1.0);
    if (mode < 0.5) { return half4(inside); }
    float edge = clamp(1.0 - abs(d + pixel * 0.5) / pixel, 0.0, 1.0);
    return tint * half(inside) + rim * half(edge);
}
";

thread_local! {
    static EFFECT: Option<RuntimeEffect> = RuntimeEffect::make_for_shader(SKSL, None)
        .inspect_err(|error| tracing::warn!(%error, "BarrierKit: the glass shader did not compile"))
        .ok();
}

#[derive(PartialEq)]
struct Blob {
    frames: Vec<Frame>,
    spacing: f32,
    origin: (f32, f32),
    tint: Rgba,
    blur: Option<f32>,
    rim: Rgba,
    layout: LayoutData,
}

fn uniforms(blob: &Blob, scale: f32, mode: f32) -> Vec<f32> {
    let mut values = Vec::with_capacity(48);
    let frames = &blob.frames[..blob.frames.len().min(MAX_SHAPES)];
    for index in 0..MAX_SHAPES {
        values.extend(match frames.get(index) {
            Some(f) => [
                (blob.origin.0 + f.x) * scale,
                (blob.origin.1 + f.y) * scale,
                f.width * scale,
                f.height * scale,
            ],
            None => [0.0; 4],
        });
    }
    for index in 0..MAX_SHAPES {
        values.push(frames.get(index).map_or(0.0, |f| f.radius * scale));
    }
    let color = |c: Rgba| [c.r * c.a, c.g * c.a, c.b * c.a, c.a];
    values.extend([
        frames.len() as f32,
        (blob.spacing * scale).max(0.001),
        scale,
        mode,
    ]);
    values.extend(color(blob.tint));
    values.extend(color(blob.rim));
    values
}

impl Blob {
    fn paint(&self, effect: &RuntimeEffect, scale: f32, mode: f32) -> Option<Paint> {
        let bytes: Vec<u8> = uniforms(self, scale, mode)
            .iter()
            .flat_map(|value| value.to_ne_bytes())
            .collect();
        let shader = effect.make_shader(Data::new_copy(&bytes), &[], None)?;
        let mut paint = Paint::default();
        paint.set_shader(shader);
        paint.set_anti_alias(true);
        Some(paint)
    }

    fn bounds(&self, scale: f32) -> SkRect {
        let grow = self.spacing;
        let (left, top, right, bottom) = self.frames.iter().fold(
            (f32::MAX, f32::MAX, f32::MIN, f32::MIN),
            |(l, t, r, b), f| {
                (
                    l.min(f.x),
                    t.min(f.y),
                    r.max(f.x + f.width),
                    b.max(f.y + f.height),
                )
            },
        );
        SkRect::new(
            (self.origin.0 + left - grow) * scale,
            (self.origin.1 + top - grow) * scale,
            (self.origin.0 + right + grow) * scale,
            (self.origin.1 + bottom + grow) * scale,
        )
    }
}

impl ElementExt for Blob {
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
        if self.frames.is_empty() {
            return;
        }
        let scale = context.scale_factor as f32;
        let bounds = self.bounds(scale);
        let canvas = context.canvas;
        EFFECT.with(|effect| {
            let Some(effect) = effect else {
                return;
            };
            let (Some(mut mask), Some(overlay)) = (
                self.paint(effect, scale, 0.0),
                self.paint(effect, scale, 1.0),
            ) else {
                return;
            };
            let filter = self.blur.and_then(|radius| {
                let sigma = radius * scale;
                blur((sigma, sigma), TileMode::Clamp, None, None)
            });
            let layer = SaveLayerRec::default().bounds(&bounds);
            canvas.save_layer(&match &filter {
                Some(filter) => layer.backdrop(filter),
                None => layer,
            });
            mask.set_blend_mode(BlendMode::DstIn);
            canvas.draw_rect(bounds, &mask);
            canvas.draw_rect(bounds, &overlay);
            canvas.restore();
        });
    }
}

type Measured = Vec<(&'static str, Frame)>;

#[derive(PartialEq)]
pub struct GlassContainer {
    spacing: f32,
    gap: f32,
    items: Vec<(&'static str, f32, Element)>,
}

impl GlassContainer {
    pub fn new(spacing: f32) -> Self {
        Self {
            spacing,
            gap: spacing,
            items: Vec::new(),
        }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn item(mut self, id: &'static str, radius: f32, content: impl IntoElement) -> Self {
        self.items.push((id, radius, content.into_element()));
        self
    }
}

impl Component for GlassContainer {
    fn render(&self) -> impl IntoElement {
        let mut origin = use_state(Area::default);
        let measured = use_state(Measured::new);
        let mut morphs = use_state(Vec::<Morph>::new);
        let reduced_motion = use_reactive(&crate::theme::resolved().reduced_motion);
        let mut generation = use_state(|| 0u64);
        let progress = use_animation_transition(generation, move |_: u64, _: u64| {
            let time = if *reduced_motion.peek() {
                0
            } else {
                DURATION_BASE
            };
            Spring::new(0.0, 1.0, Duration::from_millis(time), 0.0)
        });
        let t = progress.get().current();
        let shown: Measured = morphs
            .read()
            .iter()
            .map(|morph| (morph.id, morph.from.lerp(&morph.to, t)))
            .collect();

        use_side_effect(move || {
            let base = origin.read().origin;
            let target: Measured = measured
                .read()
                .iter()
                .map(|(id, frame)| {
                    (
                        *id,
                        Frame {
                            x: frame.x - base.x,
                            y: frame.y - base.y,
                            ..*frame
                        },
                    )
                })
                .collect();
            let current = morphs.peek();
            let settled: Vec<(&'static str, Frame)> =
                current.iter().map(|morph| (morph.id, morph.to)).collect();
            if settled == target {
                return;
            }
            let t = progress.peek().current();
            let from: Measured = current
                .iter()
                .filter(|morph| target.iter().any(|(id, _)| *id == morph.id) || t < 1.0)
                .map(|morph| (morph.id, morph.from.lerp(&morph.to, t)))
                .collect();
            drop(current);
            let first = from.is_empty();
            morphs.set(plan(&from, &target));
            if !first {
                let next = *generation.peek() + 1;
                generation.set(next);
            }
        });

        let items = self.items.iter().map(|(id, radius, content)| {
            GlassItem {
                id,
                radius: *radius,
                measured,
                content: content.clone(),
            }
            .into_element()
        });
        let shown_origin = *origin.read();
        let (tint, blur) = material(
            MEDIA.round_glass,
            MEDIA.material_tint,
            MEDIA.material_blur,
            transparency_reduced(),
        );
        rect()
            .direction(Direction::Horizontal)
            .spacing(self.gap)
            .on_sized(move |event: Event<SizedEventData>| origin.set_if_modified(event.area))
            .child(leaf(Blob {
                frames: shown.into_iter().map(|(_, frame)| frame).collect(),
                spacing: self.spacing,
                origin: (shown_origin.origin.x, shown_origin.origin.y),
                tint,
                blur,
                rim: MEDIA.round_rim,
                layout: layout(Size::fill(), Size::fill(), Position::new_absolute()),
            }))
            .children(items)
    }
}

#[derive(PartialEq)]
struct GlassItem {
    id: &'static str,
    radius: f32,
    measured: State<Measured>,
    content: Element,
}

impl Component for GlassItem {
    fn render(&self) -> impl IntoElement {
        let id = self.id;
        let radius = self.radius;
        let mut measured = self.measured;
        use_drop(move || measured.write().retain(|(shown, _)| *shown != id));
        rect()
            .on_sized(move |event: Event<SizedEventData>| {
                let frame = Frame {
                    x: event.area.origin.x,
                    y: event.area.origin.y,
                    width: event.area.width(),
                    height: event.area.height(),
                    radius: radius.min(event.area.height() / 2.0),
                };
                let mut list = measured.peek().clone();
                match list.iter_mut().find(|(shown, _)| *shown == id) {
                    Some((_, current)) if *current == frame => return,
                    Some((_, current)) => *current = frame,
                    None => list.push((id, frame)),
                }
                measured.set(list);
            })
            .child(self.content.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(x: f32) -> Frame {
        Frame {
            x,
            y: 0.0,
            width: 40.0,
            height: 40.0,
            radius: 20.0,
        }
    }

    #[test]
    fn a_new_shape_grows_out_of_its_nearest_neighbour_and_a_removed_one_melts_into_it() {
        let appear = plan(
            &[("a", frame(0.0))],
            &[("a", frame(0.0)), ("b", frame(60.0))],
        );
        assert_eq!(appear[1].from, frame(0.0));
        assert_eq!(appear[1].to, frame(60.0));
        let vanish = plan(
            &[("a", frame(0.0)), ("b", frame(60.0))],
            &[("a", frame(0.0))],
        );
        assert_eq!(vanish.len(), 2);
        assert_eq!((vanish[1].id, vanish[1].to), ("b", frame(0.0)));
    }

    #[test]
    fn the_shader_compiles_and_takes_every_uniform() {
        let effect = RuntimeEffect::make_for_shader(SKSL, None).unwrap();
        let blob = Blob {
            frames: vec![frame(0.0)],
            spacing: 12.0,
            origin: (0.0, 0.0),
            tint: MEDIA.round_glass,
            blur: Some(MEDIA.material_blur),
            rim: MEDIA.round_rim,
            layout: layout(Size::fill(), Size::fill(), Position::new_absolute()),
        };
        assert_eq!(effect.uniform_size(), uniforms(&blob, 1.0, 0.0).len() * 4);
    }
}

use std::borrow::Cow;
use std::rc::Rc;

use ctdesign::Rgba;
use freya_animation::prelude::{AnimNum, use_animation_transition};
use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{BlendMode, Paint, Rect as SkRect, SaveLayerRec, TileMode, blur};
use torin::prelude::{Area, Direction, Position, Size};

use crate::elements::{Overrides, forward_overrides};
use crate::glass::{layout, leaf, material, transparency_reduced};
use crate::platform::backdrop::{Declared, GlassLook};
use crate::shader::Program;
use crate::styles::{
    ControlState, GlassContainerStyle, Paints, ShapeStyle, StateValue, Theme, resolve,
};

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
uniform float4 starts[8];
uniform float4 frames[8];
uniform float4 radii_from[2];
uniform float4 radii[2];
uniform float t;
uniform float count;
uniform float k;
uniform float pixel;
uniform float mode;
uniform half4 tint;
uniform half4 rim;

float radius_at(int i, float4 radii[2]) {
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
        float4 frame = mix(starts[i], frames[i], t);
        float radius = mix(radius_at(i, radii_from), radius_at(i, radii), t);
        d = smooth_min(d, rounded_box(p, frame, radius));
    }
    float inside = clamp(0.5 - d / pixel, 0.0, 1.0);
    if (mode < 0.5) { return half4(inside); }
    float edge = clamp(1.0 - abs(d + pixel * 0.5) / pixel, 0.0, 1.0);
    return tint * half(inside) + rim * half(edge);
}
";

#[derive(PartialEq)]
struct Blob {
    from: Vec<Frame>,
    to: Vec<Frame>,
    t: f32,
    spacing: f32,
    origin: (f32, f32),
    tint: Rgba,
    blur: Option<f32>,
    rim: Rgba,
    layout: LayoutData,
}

impl Blob {
    fn boxes(&self, frames: &[Frame], scale: f32) -> ([f32; 4 * MAX_SHAPES], [f32; MAX_SHAPES]) {
        let mut boxes = [0.0f32; 4 * MAX_SHAPES];
        let mut radii = [0.0f32; MAX_SHAPES];
        for (index, f) in frames.iter().take(MAX_SHAPES).enumerate() {
            boxes[index * 4..index * 4 + 4].copy_from_slice(&[
                (self.origin.0 + f.x) * scale,
                (self.origin.1 + f.y) * scale,
                f.width * scale,
                f.height * scale,
            ]);
            radii[index] = f.radius * scale;
        }
        (boxes, radii)
    }

    fn paint(&self, program: &Program, scale: f32, mode: f32) -> Option<Paint> {
        let (starts, radii_from) = self.boxes(&self.from, scale);
        let (boxes, radii) = self.boxes(&self.to, scale);
        let count = self.to.len().min(MAX_SHAPES);
        let color = |c: Rgba| [c.r * c.a, c.g * c.a, c.b * c.a, c.a];
        let shader = program.shader(&[
            ("starts", &starts),
            ("frames", &boxes),
            ("radii_from", &radii_from),
            ("radii", &radii),
            ("t", &[self.t]),
            ("count", &[count as f32]),
            ("k", &[(self.spacing * scale).max(0.001)]),
            ("pixel", &[scale]),
            ("mode", &[mode]),
            ("tint", &color(self.tint)),
            ("rim", &color(self.rim)),
        ])?;
        let mut paint = Paint::default();
        paint.set_shader(shader);
        paint.set_anti_alias(true);
        Some(paint)
    }

    fn bounds(&self, scale: f32) -> SkRect {
        let grow = self.spacing;
        let (left, top, right, bottom) = self.from.iter().chain(&self.to).fold(
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
        if self.to.is_empty() {
            return;
        }
        let scale = context.scale_factor as f32;
        let bounds = self.bounds(scale);
        let canvas = context.canvas;
        let Some(program) = Program::get(SKSL) else {
            return;
        };
        let (Some(mut mask), Some(overlay)) = (
            self.paint(&program, scale, 0.0),
            self.paint(&program, scale, 1.0),
        ) else {
            return;
        };
        let filter = self.blur.and_then(|radius| {
            let sigma = radius * scale;
            blur((sigma, sigma), TileMode::Clamp, None, None)
        });
        let layer = SaveLayerRec::default().bounds(&bounds);
        // SaveLayerRec bounds are only a hint; without the clip the blurred layer covers the whole canvas.
        canvas.save();
        canvas.clip_rect(bounds, None, Some(true));
        canvas.save_layer(&match &filter {
            Some(filter) => layer.backdrop(filter),
            None => layer,
        });
        mask.set_blend_mode(BlendMode::DstIn);
        canvas.draw_rect(bounds, &mask);
        canvas.draw_rect(bounds, &overlay);
        canvas.restore();
        canvas.restore();
    }
}

type Measured = Vec<(&'static str, Frame)>;

#[derive(PartialEq)]
pub struct GlassContainer {
    spacing: f32,
    gap: f32,
    items: Vec<(&'static str, f32, Element)>,
    style: Theme,
    overrides: Overrides,
}

impl GlassContainer {
    pub fn new(spacing: f32) -> Self {
        Self {
            spacing,
            gap: spacing,
            items: Vec::new(),
            style: Theme::tengen(),
            overrides: Overrides::default(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(GlassContainerStyle) -> GlassContainerStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
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

forward_overrides!(GlassContainer);

impl Component for GlassContainer {
    fn render(&self) -> impl IntoElement {
        let style: GlassContainerStyle = resolve(&self.style);
        let animation = style.animation;
        let mut origin = use_state(Area::default);
        let measured = use_state(Measured::new);
        let mut morphs = use_state(Vec::<Morph>::new);
        let duration_ms = use_reactive(&animation.duration_ms());
        let mut generation = use_state(|| 0u64);
        let progress = use_animation_transition(generation, move |_: u64, _: u64| {
            AnimNum::new(0.0, 1.0).time(*duration_ms.peek())
        });
        let t = animation.progress(progress.get().value());
        let shown: Vec<Morph> = morphs
            .read()
            .iter()
            .filter(|morph| t < 1.0 || measured.peek().iter().any(|(id, _)| *id == morph.id))
            .cloned()
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
            let t = animation.progress(progress.peek().value());
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
        let paints = Paints::current(ControlState::default(), style.effect);
        let ink =
            |paint: &StateValue<ShapeStyle>| paints.color(paint.resolve(ControlState::default()));
        let (tint, blur) = material(
            ink(&style.tint),
            ink(&style.solid),
            style.blur,
            transparency_reduced(),
        );
        if crate::glass::native_glass() {
            let look = GlassLook {
                clear: style.clear,
                spacing: self.spacing,
            };
            let glasses: Vec<Element> = measured
                .read()
                .iter()
                .map(|(_, frame)| {
                    Declared::glass(
                        frame.radius,
                        (frame.width, frame.height),
                        (
                            frame.x - shown_origin.origin.x,
                            frame.y - shown_origin.origin.y,
                        ),
                        look,
                    )
                })
                .collect();
            let root = rect()
                .direction(Direction::Horizontal)
                .spacing(self.gap)
                .on_sized(move |event: Event<SizedEventData>| origin.set_if_modified(event.area))
                .children(glasses)
                .children(items);
            return self.overrides.apply(root);
        }
        let root = rect()
            .direction(Direction::Horizontal)
            .spacing(self.gap)
            .on_sized(move |event: Event<SizedEventData>| origin.set_if_modified(event.area))
            .child(leaf(Blob {
                from: shown.iter().map(|morph| morph.from).collect(),
                to: shown.iter().map(|morph| morph.to).collect(),
                t,
                spacing: self.spacing,
                origin: (shown_origin.origin.x, shown_origin.origin.y),
                tint,
                blur,
                rim: ink(&style.rim),
                layout: layout(Size::fill(), Size::fill(), Position::new_absolute()),
            }))
            .children(items);
        self.overrides.apply(root)
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
    use ctdesign::MEDIA;

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
        let program = Program::get(SKSL).unwrap();
        let blob = Blob {
            from: vec![frame(0.0)],
            to: vec![frame(30.0)],
            t: 0.5,
            spacing: 12.0,
            origin: (0.0, 0.0),
            tint: MEDIA.round_glass,
            blur: Some(MEDIA.material_blur),
            rim: MEDIA.round_rim,
            layout: layout(Size::fill(), Size::fill(), Position::new_absolute()),
        };
        assert!(blob.paint(&program, 1.0, 0.0).is_some());
        assert!(blob.paint(&program, 2.0, 1.0).is_some());
    }

    // The shader mixes the frames by `t`, which is the animation's progress: the picture is the one of the frames mixed on the CPU.
    #[test]
    fn the_shader_morphs_by_the_time_it_is_given_as_the_cpu_mix_of_the_frames_does() {
        use freya_engine::prelude::{AlphaType, ColorType, ImageInfo, raster_n32_premul};
        let program = Program::get(SKSL).unwrap();
        let (a, b) = (
            frame(0.0),
            Frame {
                x: 40.0,
                y: 10.0,
                width: 30.0,
                height: 50.0,
                radius: 6.0,
            },
        );
        let blob = |from: Frame, to: Frame, t: f32| Blob {
            from: vec![from],
            to: vec![to],
            t,
            spacing: 12.0,
            origin: (0.0, 0.0),
            tint: MEDIA.round_glass,
            blur: None,
            rim: MEDIA.round_rim,
            layout: layout(Size::fill(), Size::fill(), Position::new_absolute()),
        };
        let pixels = |blob: Blob| {
            let mut surface = raster_n32_premul((90, 70)).unwrap();
            let paint = blob.paint(&program, 1.0, 1.0).unwrap();
            surface
                .canvas()
                .draw_rect(SkRect::new(0.0, 0.0, 90.0, 70.0), &paint);
            let info = ImageInfo::new((90, 70), ColorType::RGBA8888, AlphaType::Premul, None);
            let mut out = vec![0u8; 90 * 70 * 4];
            assert!(surface.read_pixels(&info, &mut out, 90 * 4, (0, 0)));
            out
        };
        for t in [0.0, 0.3, 1.0] {
            let gpu = pixels(blob(a, b, t));
            let cpu = pixels(blob(a.lerp(&b, t), a.lerp(&b, t), 0.0));
            let worst = gpu
                .iter()
                .zip(&cpu)
                .map(|(x, y)| x.abs_diff(*y))
                .max()
                .unwrap();
            assert!(worst <= 1, "t {t}: {worst}");
        }
    }
}

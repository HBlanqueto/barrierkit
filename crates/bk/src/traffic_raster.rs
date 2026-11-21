use std::cell::RefCell;
use std::collections::HashMap;

use ctappearance::traffic::{self, ButtonVisual, Glyph};
use ctdesign::{Rgba, TRAFFIC_GLYPHS, TRAFFIC_MARGIN, TrafficHalo};
use freya_core::elements::image::ImageHandle;
use freya_core::prelude::{Bytes, Platform};
use freya_engine::prelude::{
    BlendMode, BlurStyle, Canvas, ClipOp, Color4f, MaskFilter, Paint, PaintCap, PaintStyle, Path,
    PathBuilder, raster_n32_premul,
};

use crate::style::linear;

const CAPACITY: usize = 96;

thread_local! {
    static LIGHTS: RefCell<HashMap<Vec<u32>, Option<ImageHandle>>> = RefCell::new(HashMap::new());
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Layers {
    All,
    Disc,
    GlyphOnly,
}

/// A light of `diameter` points, drawn as macOS draws it: a vertical gradient disc, a ring along its edge, a halo
/// outside it and, when hovered, its `glyph`. A pressed light is enlarged and brightened as it sits on `backdrop`.
/// `layers` keeps the disc or the glyph apart so that a glyph can cross-fade into another.
/// The image is `diameter` plus `TRAFFIC_MARGIN` on every side, rasterized once per look at the window's scale.
pub(crate) fn light(
    visual: &ButtonVisual,
    halo: TrafficHalo,
    backdrop: Rgba,
    ring_width: f32,
    diameter: f32,
    glyph: Glyph,
    layers: Layers,
) -> Option<ImageHandle> {
    let scale = *Platform::get().current_window().scale_factor.read() as f32;
    let colors = visual
        .fill
        .iter()
        .chain(&visual.ring)
        .chain(visual.glyph.iter())
        .chain([&halo.color, &backdrop]);
    let key: Vec<u32> = colors
        .flat_map(|c| [c.r, c.g, c.b, c.a])
        .chain([
            visual.scale,
            visual.dodge,
            halo.blur,
            halo.dy,
            ring_width,
            diameter,
            scale,
        ])
        .map(f32::to_bits)
        .chain([
            u32::from(visual.glyph.is_some()),
            glyph as u32,
            layers as u32,
        ])
        .collect();
    LIGHTS.with_borrow_mut(|cache| {
        if let Some(cached) = cache.get(&key) {
            return cached.clone();
        }
        let image = draw(
            visual,
            halo,
            backdrop,
            ring_width,
            diameter,
            (glyph, layers),
            scale,
        );
        if cache.len() >= CAPACITY {
            cache.clear();
        }
        cache.insert(key, image.clone());
        image
    })
}

fn paint(color: Rgba) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color4f(Color4f::new(color.r, color.g, color.b, color.a), None);
    paint
}

fn draw(
    visual: &ButtonVisual,
    halo: TrafficHalo,
    backdrop: Rgba,
    ring_width: f32,
    diameter: f32,
    (glyph, layers): (Glyph, Layers),
    scale: f32,
) -> Option<ImageHandle> {
    let side = (diameter + 2.0 * TRAFFIC_MARGIN) * scale;
    let mut surface = raster_n32_premul((side.round() as i32, side.round() as i32))?;
    let canvas = surface.canvas();
    let radius = diameter / 2.0;
    let center = radius + TRAFFIC_MARGIN;
    canvas.scale((scale, scale));
    canvas.translate((center, center));
    canvas.scale((visual.scale, visual.scale));
    canvas.translate((-center, -center));
    let disc = Path::circle((center, center), radius, None);

    if layers != Layers::GlyphOnly && halo.color.a > 0.0 {
        let mut shadow = paint(halo.color);
        shadow.set_mask_filter(MaskFilter::blur(BlurStyle::Normal, halo.blur, true));
        canvas.save();
        canvas.clip_path(&disc, ClipOp::Difference, true);
        canvas.draw_circle((center, center + halo.dy), radius, &shadow);
        canvas.restore();
    }

    if layers != Layers::GlyphOnly && visual.dodge > 0.0 {
        canvas.draw_path(&disc, &paint(backdrop));
    }

    let (top, bottom) = (center - radius, center + radius);
    let stops = |colors: &[Rgba]| -> Vec<(f32, Rgba)> {
        let last = (colors.len() - 1) as f32;
        colors
            .iter()
            .enumerate()
            .map(|(index, &color)| (index as f32 / last, color))
            .collect()
    };
    if layers != Layers::GlyphOnly {
        let mut fill = Paint::default();
        fill.set_anti_alias(true);
        fill.set_shader(linear((0.0, top), (0.0, bottom), &stops(&visual.fill)));
        canvas.draw_path(&disc, &fill);

        let mut ring = Paint::default();
        ring.set_anti_alias(true);
        ring.set_style(PaintStyle::Stroke);
        ring.set_stroke_width(ring_width);
        ring.set_shader(linear((0.0, top), (0.0, bottom), &stops(&visual.ring)));
        canvas.draw_circle((center, center), radius - ring_width / 2.0, &ring);
    }

    if let Some(color) = visual.glyph.filter(|_| layers != Layers::Disc) {
        canvas.save();
        canvas.translate((TRAFFIC_MARGIN, TRAFFIC_MARGIN));
        draw_glyph(canvas, glyph, paint(color));
        canvas.restore();
    }

    if layers != Layers::GlyphOnly && visual.dodge > 0.0 {
        let mut dodge = paint(Rgba {
            r: visual.dodge,
            g: visual.dodge,
            b: visual.dodge,
            a: 1.0,
        });
        dodge.set_blend_mode(BlendMode::ColorDodge);
        canvas.draw_path(&disc, &dodge);
    }
    Some(ImageHandle::new(surface.image_snapshot(), Bytes::new()))
}

fn draw_glyph(canvas: &Canvas, glyph: Glyph, mut ink: Paint) {
    let strokes = |canvas: &Canvas, ink: &mut Paint, cap, segments: &[ctdesign::Segment]| {
        ink.set_style(PaintStyle::Stroke);
        ink.set_stroke_width(TRAFFIC_GLYPHS.stroke);
        ink.set_stroke_cap(cap);
        for segment in segments {
            canvas.draw_line(segment.a, segment.b, ink);
        }
    };
    let triangles = match glyph {
        Glyph::Close => return strokes(canvas, &mut ink, PaintCap::Round, TRAFFIC_GLYPHS.close),
        Glyph::Minimize => {
            return strokes(canvas, &mut ink, PaintCap::Butt, TRAFFIC_GLYPHS.minimize);
        }
        Glyph::Plus => return strokes(canvas, &mut ink, PaintCap::Butt, TRAFFIC_GLYPHS.plus),
        Glyph::ZoomEnter => TRAFFIC_GLYPHS.zoom,
        Glyph::ZoomExit => TRAFFIC_GLYPHS.zoom_exit,
    };
    for triangle in triangles {
        let [first, second, third] = traffic::rounded(triangle);
        let mut outline = PathBuilder::new();
        outline.move_to(first.exit);
        for corner in [second, third, first] {
            outline
                .line_to(corner.enter)
                .quad_to(corner.vertex, corner.exit);
        }
        outline.close();
        canvas.draw_path(&outline.detach(), &ink);
    }
}

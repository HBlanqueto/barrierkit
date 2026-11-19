use std::cell::RefCell;
use std::collections::HashMap;

use ctappearance::icon_plate::{Logo, Plate, logo_matrix, plate};
use ctappearance::icon_style::IconStyle;
use ctdesign::{ICON_PLATE, Rgba};
use freya_core::elements::image::{ImageHandle, SamplingMode};
use freya_core::prelude::{Bytes, Platform};
use freya_engine::prelude::{
    AlphaType, BlendMode, Canvas, ClipOp, Color, ColorType, Data, ImageInfo, Paint, PaintStyle,
    Path, PathBuilder, RRect, Rect as SkRect, RuntimeEffect, SkImage, color_filters,
    raster_n32_premul,
};

use crate::style::linear;

// The Settings icon's body on the 1024 grid.
const GRID: f32 = 1024.0;
const BODY_FROM: f32 = 100.0;
const BODY_TO: f32 = 924.0;
const CORNER_TO: f32 = 300.0;
const TILE_PLATE_CAPACITY: usize = 64;
const OPAQUE: u8 = 128;
const VISIBLE: u8 = 25;

fn squircle(scale: f32) -> Path {
    let p = |x: f32, y: f32| (x * scale, y * scale);
    let mut body = PathBuilder::new();
    body.move_to(p(100.0, 724.0))
        .line_to(p(100.0, 300.0))
        .cubic_to(p(100.0, 185.0), p(170.0, 100.0), p(300.0, 100.0))
        .line_to(p(724.0, 100.0))
        .cubic_to(p(854.0, 100.0), p(924.0, 185.0), p(924.0, 300.0))
        .line_to(p(924.0, 724.0))
        .cubic_to(p(924.0, 839.0), p(854.0, 924.0), p(724.0, 924.0))
        .line_to(p(300.0, 924.0))
        .cubic_to(p(170.0, 924.0), p(100.0, 839.0), p(100.0, 724.0))
        .close();
    body.detach()
}

struct Pixels {
    opaque: Vec<Rgba>,
    center: (f32, f32),
    extent: f32,
}

fn read(raster: &SkImage) -> Option<Pixels> {
    let (width, height) = (raster.width(), raster.height());
    let mut surface = raster_n32_premul((width, height))?;
    surface.canvas().draw_image(raster, (0, 0), None);
    let info = ImageInfo::new(
        (width, height),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let mut bytes = vec![0; width as usize * height as usize * 4];
    surface
        .read_pixels(&info, &mut bytes, width as usize * 4, (0, 0))
        .then_some(())?;
    let (mut left, mut top, mut right, mut bottom) = (width, height, 0, 0);
    let mut opaque = Vec::new();
    for (index, pixel) in bytes.as_chunks::<4>().0.iter().enumerate() {
        let (x, y) = (index as i32 % width, index as i32 / width);
        if pixel[3] > VISIBLE {
            (left, top, right, bottom) =
                (left.min(x), top.min(y), right.max(x + 1), bottom.max(y + 1));
        }
        if pixel[3] >= OPAQUE {
            opaque.push(Rgba::rgb8(pixel[0], pixel[1], pixel[2]));
        }
    }
    (right > left && !opaque.is_empty()).then_some(Pixels {
        opaque,
        center: ((left + right) as f32 / 2.0, (top + bottom) as f32 / 2.0),
        extent: (right - left).max(bottom - top) as f32,
    })
}

fn color(rgba: Rgba) -> Color {
    Color::from_argb(
        (rgba.a * 255.0).round() as u8,
        (rgba.r * 255.0).round() as u8,
        (rgba.g * 255.0).round() as u8,
        (rgba.b * 255.0).round() as u8,
    )
}

/// `raster` is an app icon; the result is the same icon on a plate, on the 1024 grid, in the user's icon style. With `glass_plate` the compositor draws the plate, so only the logo is drawn.
pub(crate) fn plated(
    raster: &SkImage,
    side: u32,
    style: IconStyle,
    dark: bool,
    tint: Rgba,
    glass_plate: bool,
) -> Option<ImageHandle> {
    let pixels = read(raster)?;
    let plate = plate(style, dark, tint, &Logo::from_colors(&pixels.opaque));
    let side = side as f32;
    let scale = side / GRID;
    let body = squircle(scale);
    let mut surface = raster_n32_premul((side as i32, side as i32))?;
    let canvas = surface.canvas();
    let glass = if plate.glass {
        ICON_PLATE.glass_shadow_factor
    } else {
        1.0
    };
    if !glass_plate {
        let mut shadow = Paint::default();
        shadow.set_anti_alias(true);
        for (offset, opacity) in [
            (ICON_PLATE.shadow_far_offset, ICON_PLATE.shadow_far_opacity),
            (
                ICON_PLATE.shadow_near_offset,
                ICON_PLATE.shadow_near_opacity,
            ),
        ] {
            shadow.set_color(color(Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: opacity * glass,
            }));
            canvas.save();
            canvas.translate((0.0, offset * side));
            canvas.draw_path(&body, &shadow);
            canvas.restore();
        }
        let mut fill = Paint::default();
        fill.set_anti_alias(true);
        fill.set_shader(linear(
            (0.0, BODY_FROM * scale),
            (0.0, BODY_TO * scale),
            &[(0.0, plate.top), (1.0, plate.bottom)],
        ));
        canvas.draw_path(&body, &fill);
        edge(
            canvas,
            &body,
            (BODY_FROM * scale, BODY_FROM * scale),
            (BODY_TO - BODY_FROM) * scale,
            (CORNER_TO - BODY_FROM) * scale,
            &plate,
        );
    }

    let fit = ICON_PLATE.logo_scale * ICON_PLATE.body * side / pixels.extent;
    let sampling = SamplingMode::Mitchell.sampling_options();
    let draw = |paint: &Paint, drop: f32| {
        canvas.save();
        canvas.translate((
            side / 2.0 - pixels.center.0 * fit,
            side / 2.0 - pixels.center.1 * fit + drop,
        ));
        canvas.scale((fit, fit));
        canvas.draw_image_with_sampling_options(raster, (0, 0), sampling, Some(paint));
        canvas.restore();
    };
    let mut cast = Paint::default();
    cast.set_anti_alias(true);
    cast.set_color_filter(color_filters::blend(Color::BLACK, BlendMode::SrcIn));
    cast.set_alpha_f(ICON_PLATE.logo_shadow_opacity * glass);
    draw(&cast, ICON_PLATE.logo_shadow_offset * side);
    let mut logo = Paint::default();
    logo.set_anti_alias(true);
    if let Some(matrix) = logo_matrix(style, dark, tint) {
        logo.set_color_filter(color_filters::matrix_row_major(
            &matrix,
            color_filters::Clamp::Yes,
        ));
    }
    draw(&logo, 0.0);
    Some(ImageHandle::new(surface.image_snapshot(), Bytes::new()))
}

// shape = (light reach, sharpness, outline on the top and bottom, 1 for the outline); ramp = the
// angles over which the outline grows to whole; side_reach = degrees per px past a corner.
const EDGE: &str = "
uniform float4 frame;
uniform float4 shape;
uniform half4 top;
uniform half4 bottom;
uniform float2 ramp;
uniform float corner;
uniform float side_reach;

half4 main(float2 p) {
    float2 local = p - frame.xy;
    float2 near = min(local, frame.zw - local);
    float angle = near.x >= corner ? 0.0
        : near.y >= corner ? 90.0 + (near.y - corner) * side_reach
        : degrees(atan(corner - near.x, corner - near.y));
    float strength = shape.w < 0.5
        ? exp(-pow(angle / shape.x, shape.y))
        : shape.z + (1.0 - shape.z) * clamp((min(angle, 90.0) - ramp.x) / (ramp.y - ramp.x), 0.0, 1.0);
    return (local.y < frame.w * 0.5 ? top : bottom) * half(strength);
}
";

thread_local! {
    static EDGE_EFFECT: Option<RuntimeEffect> = RuntimeEffect::make_for_shader(EDGE, None)
        .inspect_err(|error| tracing::warn!(%error, "BarrierKit: the plate edge shader did not compile"))
        .ok();
}

fn edge_paint(
    frame: [f32; 4],
    corner: f32,
    outline: bool,
    (top, bottom): (Rgba, Rgba),
    width: f32,
) -> Option<Paint> {
    let premultiplied = |c: Rgba| [c.r * c.a, c.g * c.a, c.b * c.a, c.a];
    let mut values = frame.to_vec();
    values.extend([
        ICON_PLATE.light_reach,
        ICON_PLATE.light_sharpness,
        ICON_PLATE.outline_ends,
        f32::from(u8::from(outline)),
    ]);
    values.extend(premultiplied(top));
    values.extend(premultiplied(bottom));
    values.extend([
        ICON_PLATE.outline_from,
        ICON_PLATE.outline_to,
        corner,
        ICON_PLATE.light_side / frame[3],
    ]);
    let bytes: Vec<u8> = values
        .iter()
        .flat_map(|value| value.to_ne_bytes())
        .collect();
    let shader = EDGE_EFFECT.with(|effect| {
        effect
            .as_ref()?
            .make_shader(Data::new_copy(&bytes), &[], None)
    })?;
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(PaintStyle::Stroke);
    paint.set_stroke_width(width);
    paint.set_shader(shader);
    Some(paint)
}

/// A plate's light along its top and bottom edges, which turns each corner and ends in a point
/// down the side, and the closed outline of clear and tinted plates; both inside `body`, whose
/// square frame starts at `origin`, measures `side` and turns its corners on radius `corner`.
pub(crate) fn edge(
    canvas: &Canvas,
    body: &Path,
    origin: (f32, f32),
    side: f32,
    corner: f32,
    plate: &Plate,
) {
    let frame = [origin.0, origin.1, side, side];
    let width = ICON_PLATE.edge_width * side;
    let outlined = plate.outline.a > 0.0;
    canvas.save();
    canvas.clip_path(body, ClipOp::Intersect, true);
    // The light sits just inside the outline, so with one it is drawn twice as deep.
    let light = if outlined { 4.0 } else { 2.0 } * width;
    if let Some(paint) = edge_paint(frame, corner, false, plate.light, light) {
        canvas.draw_path(body, &paint);
    }
    if outlined
        && let Some(paint) = edge_paint(
            frame,
            corner,
            true,
            (plate.outline, plate.outline),
            2.0 * width,
        )
    {
        canvas.draw_path(body, &paint);
    }
    canvas.restore();
}

type TilePlateKey = (u32, u32, [[u32; 4]; 5]);

thread_local! {
    static TILE_PLATES: RefCell<HashMap<TilePlateKey, Option<ImageHandle>>> =
        RefCell::new(HashMap::new());
}

/// A symbol tile's plate, `size` points square with corners of `radius`, rasterized once per look.
pub(crate) fn tile_plate(plate: &Plate, size: f32, radius: f32) -> Option<ImageHandle> {
    let scale = *Platform::get().current_window().scale_factor.read() as f32;
    let side = (size * scale).round().max(1.0);
    let radius = radius * scale;
    let bits = |c: Rgba| [c.r, c.g, c.b, c.a].map(f32::to_bits);
    let colors = [
        plate.top,
        plate.bottom,
        plate.light.0,
        plate.light.1,
        plate.outline,
    ];
    let key = (side.to_bits(), radius.to_bits(), colors.map(bits));
    TILE_PLATES.with_borrow_mut(|cache| {
        if let Some(handle) = cache.get(&key) {
            return handle.clone();
        }
        let handle = draw_tile_plate(plate, side, radius);
        if cache.len() >= TILE_PLATE_CAPACITY {
            cache.clear();
        }
        cache.insert(key, handle.clone());
        handle
    })
}

fn draw_tile_plate(plate: &Plate, side: f32, radius: f32) -> Option<ImageHandle> {
    let mut surface = raster_n32_premul((side as i32, side as i32))?;
    let canvas = surface.canvas();
    let body = Path::rrect(
        RRect::new_rect_xy(SkRect::from_wh(side, side), radius, radius),
        None,
    );
    let mut fill = Paint::default();
    fill.set_anti_alias(true);
    fill.set_shader(linear(
        (0.0, 0.0),
        (0.0, side),
        &[(0.0, plate.top), (1.0, plate.bottom)],
    ));
    canvas.draw_path(&body, &fill);
    edge(canvas, &body, (0.0, 0.0), side, radius, plate);
    Some(ImageHandle::new(surface.image_snapshot(), Bytes::new()))
}

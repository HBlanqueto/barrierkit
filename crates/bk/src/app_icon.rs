use ctappearance::ColorScheme;
use ctappearance::icon_plate::{Logo, logo_matrix, plate};
use ctappearance::icon_style::IconStyle;
use ctdesign::{ICON_PLATE, Rgba};
use freya_core::elements::image::{ImageHandle, SamplingMode};
use freya_core::prelude::Bytes;
use freya_engine::prelude::{
    AlphaType, BlendMode, ClipOp, Color, ColorType, ImageInfo, Paint, PaintStyle, Path,
    PathBuilder, SkImage, color_filters, raster_n32_premul,
};

use crate::style::linear;

// The Settings icon's body on the 1024 grid.
const GRID: f32 = 1024.0;
const BODY_FROM: f32 = 100.0;
const BODY_TO: f32 = 924.0;
const RIM_FADE: [(f32, f32); 5] = [
    (0.0, 1.0),
    (0.3, 0.25),
    (0.5, 0.0),
    (0.75, 0.12),
    (1.0, 0.7),
];
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

/// `raster` is an app icon; the result is the same icon on a plate, on the 1024 grid, in the user's icon style.
pub(crate) fn plated(
    raster: &SkImage,
    side: u32,
    style: IconStyle,
    scheme: ColorScheme,
    accent: Rgba,
) -> Option<ImageHandle> {
    let pixels = read(raster)?;
    let plate = plate(style, scheme, accent, &Logo::from_colors(&pixels.opaque));
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
    let mut outline = Paint::default();
    outline.set_anti_alias(true);
    outline.set_style(PaintStyle::Stroke);
    outline.set_stroke_width(ICON_PLATE.outline_width * side);
    outline.set_color(color(Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: ICON_PLATE.outline_opacity * glass,
    }));
    canvas.draw_path(&body, &outline);
    let rim: Vec<_> = RIM_FADE
        .iter()
        .map(|&(at, fade)| {
            (
                at,
                Rgba {
                    a: plate.rim.a * fade,
                    ..plate.rim
                },
            )
        })
        .collect();
    let mut edge = Paint::default();
    edge.set_anti_alias(true);
    edge.set_style(PaintStyle::Stroke);
    edge.set_stroke_width(ICON_PLATE.rim_width * side);
    edge.set_shader(linear(
        (BODY_FROM * scale, BODY_FROM * scale),
        (BODY_TO * scale, BODY_TO * scale),
        &rim,
    ));
    canvas.save();
    canvas.clip_path(&body, ClipOp::Intersect, true);
    canvas.draw_path(&body, &edge);
    canvas.restore();

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
    if let Some(matrix) = logo_matrix(style, scheme, accent) {
        logo.set_color_filter(color_filters::matrix_row_major(
            &matrix,
            color_filters::Clamp::Yes,
        ));
    }
    draw(&logo, 0.0);
    Some(ImageHandle::new(surface.image_snapshot(), Bytes::new()))
}

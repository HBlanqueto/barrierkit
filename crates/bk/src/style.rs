use std::borrow::Cow;

use ctdesign::{Font, Rgba, WINDOW_CORNER};
use freya_core::prelude::*;
use torin::prelude::{Position, Size};

use crate::theme::color;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub label: Rgba,
    pub secondary_label: Rgba,
    pub tertiary_label: Rgba,
    pub separator: Rgba,
    pub control_border: Rgba,
    pub window_background: Rgba,
    pub content_background: Rgba,
    pub control_background: Rgba,
    pub control_hover: Rgba,
    pub accent: Rgba,
    pub accent_inactive: Rgba,
    pub on_accent: Rgba,
    pub destructive: Rgba,
    pub raised_background: Rgba,
    pub sunken_background: Rgba,
    pub scrim: Rgba,
    pub selected_control: Rgba,
    pub card_border: Rgba,
}

impl Palette {
    pub fn current() -> Self {
        let palette = try_consume_context::<State<Palette>>()
            .map_or_else(Self::system, |state| *state.read());
        if crate::platform::window_activated() {
            palette
        } else {
            Self {
                accent: palette.accent_inactive,
                on_accent: ctappearance::backdrop::ink_on(palette.accent_inactive),
                ..palette
            }
        }
    }

    pub fn system() -> Self {
        let resolved = crate::theme::resolved();
        let base = ctappearance::palette(resolved.scheme);
        // HIG «Increase Contrast»: each ink steps up one level and control edges reach 3:1.
        let (secondary_label, tertiary_label, separator, control_border) = if resolved.contrast {
            (
                base.ink_strong,
                base.ink_muted,
                base.ink_muted,
                base.ink_muted,
            )
        } else {
            (base.ink_muted, base.ink_faint, base.line, base.line_strong)
        };
        Self {
            label: base.ink_strong,
            secondary_label,
            tertiary_label,
            separator,
            control_border,
            window_background: resolved.surfaces.window,
            content_background: resolved.surfaces.surface,
            control_background: resolved.surfaces.control,
            control_hover: resolved.surfaces.hover,
            accent: resolved.accent,
            accent_inactive: base.accent_inactive,
            on_accent: resolved.on_accent,
            destructive: base.danger,
            raised_background: resolved.surfaces.raised,
            sunken_background: resolved.surfaces.sunken,
            scrim: base.scrim,
            selected_control: base.selected_control,
            card_border: base.card_border,
        }
    }
}

pub fn text(content: impl Into<Cow<'static, str>>, face: Font, ink: Rgba) -> Label {
    with_font(label().text(content), face)
        .max_lines(1)
        .color(color(ink))
}

pub(crate) fn with_font(label: Label, face: Font) -> Label {
    label
        .font_family(face.family)
        .font_size(face.size as f32)
        .font_weight(face.weight as i32)
        .line_height(face.line_height as f32 / face.size as f32)
}

pub fn continuous_corner(radius: f32) -> CornerRadius {
    continuous(CornerRadius::new_all(radius))
}

// Freya's smoothing s runs the curve to (1 + s) R, Apple's 1.5287 R (malaysia ADR 0059).
pub(crate) fn continuous(radius: CornerRadius) -> CornerRadius {
    radius.with_smoothing(WINDOW_CORNER.expansion - 1.0)
}

/// A card filled with `fill` and framed like a panel window: in dark the glass inner line
/// (brighter on the top and bottom edges, with a faint inward glow), otherwise the window inner
/// line, and the window outer line. Same colours as the compositor's panel frames (malaysia ADR 0068).
pub fn panel_edges(body: Rect, radius: f32, fill: Rgba) -> Rect {
    let palette = ctappearance::palette(crate::theme::resolved().scheme);
    let corner = continuous_corner(radius);
    let body = body.corner_radius(corner);
    // The compositor's frame lines are one physical pixel at every scale.
    let pixel = 1.0 / *Platform::get().current_window().scale_factor.read() as f32;
    let body = match (palette.panel_inner_line, palette.inner_line) {
        (Some(line), _) => body.background(Fill::Callback(RenderCallback::new(move |context| {
            glass_line(context, fill, line, corner)
        }))),
        (None, Some(inner)) => body.background(color(fill)).border(
            Border::new()
                .fill(color(inner))
                .width(pixel)
                .alignment(BorderAlignment::Inner),
        ),
        (None, None) => body.background(color(fill)),
    };
    rect()
        .corner_radius(corner)
        .border(
            Border::new()
                .fill(color(palette.outer_border))
                .width(pixel)
                .alignment(BorderAlignment::Outer),
        )
        .child(body)
}

/// The Dock's frame: the light of its rim along its inner edge, as colours (with their opacity)
/// read from the backdrop, and the `outline` that darkens its sides. `top` and `bottom` run left
/// to right along those edges; `left` and `right` are the sides.
#[derive(Clone, Debug, PartialEq)]
pub struct GlassRim {
    pub top: Vec<Rgba>,
    pub bottom: Vec<Rgba>,
    pub left: Rgba,
    pub right: Rgba,
    pub outline: Rgba,
}

/// A glass of `width` by `height` filled with `fill` and framed as macOS frames the Dock: a dark
/// outline one pixel outside it, strongest on the sides, and a light rim one pixel inside it with a
/// glow that dies away inwards, strongest on top and bottom. Around the corners the outline hands
/// over to the rim (`ctdesign::dock_rim_dark_share`).
pub fn glass_edge((width, height): (f32, f32), radius: f32, fill: Rgba, rim: GlassRim) -> Rect {
    let corner = continuous_corner(radius);
    let pixel = 1.0 / *Platform::get().current_window().scale_factor.read() as f32;
    rect()
        .width(Size::px(width))
        .height(Size::px(height))
        .child(
            rect()
                .position(Position::new_absolute().left(-pixel).top(-pixel))
                .width(Size::px(width + 2.0 * pixel))
                .height(Size::px(height + 2.0 * pixel))
                .background(Fill::Callback(RenderCallback::new(move |context| {
                    glass_frame(context, fill, &rim, corner)
                }))),
        )
}

fn glass_frame(context: &mut FillRenderContext, fill: Rgba, rim: &GlassRim, corner: CornerRadius) {
    use freya_engine::prelude::{
        BlendMode, ClipOp, Paint, PaintStyle, PathBuilder, RRect, Rect as SkRect, shaders,
    };
    let RenderTarget::Background { canvas, .. } = &context.target else {
        return;
    };
    let pixel = 1.0 / context.scale_factor as f32;
    let (full_width, full_height) = (context.size.width, context.size.height);
    let (width, height) = (full_width - 2.0 * pixel, full_height - 2.0 * pixel);
    let radius = corner.top_left();
    let glass = SkRect::from_xywh(pixel, pixel, width, height);
    // The glass shape, `inset` logical pixels in (negative: out).
    let path = |inset: f32| {
        let reach = (radius - inset).max(0.0);
        let mut outline = PathBuilder::new();
        outline
            .add_path(
                &continuous_corner(reach).smoothed_path(RRect::new_rect_xy(
                    glass.with_inset((inset, inset)),
                    reach,
                    reach,
                )),
                None,
            )
            .close();
        outline.detach()
    };
    let down = |colour: Rgba, share: &dyn Fn(f32) -> f32| -> Vec<(f32, Rgba)> {
        (0..=20)
            .map(|step| {
                let at = step as f32 / 20.0;
                (
                    at,
                    Rgba {
                        a: colour.a * share(at),
                        ..colour
                    },
                )
            })
            .collect()
    };
    let line = |shader: freya_engine::prelude::Shader, depth: f32| {
        let mut paint = Paint::default();
        paint.set_shader(shader);
        paint.set_style(PaintStyle::Stroke);
        paint.set_anti_alias(true);
        paint.set_stroke_width(pixel);
        (paint, path(depth))
    };

    canvas.save();
    canvas.clip_path(&path(0.0), ClipOp::Intersect, true);
    let mut paint = Paint::default();
    paint.set_color(color(fill));
    canvas.draw_paint(&paint);
    let reach = (radius / width).min(0.5);
    let strengths = std::iter::once(1.0).chain(ctdesign::DOCK_RIM_GLOW);
    for (depth, strength) in strengths.enumerate() {
        let fade = |at: f32| strength * (1.0 - ctdesign::dock_rim_dark_share(at));
        let Some(mask) = linear((0.0, pixel), (0.0, pixel + height), &down(WHITE, &fade)) else {
            continue;
        };
        for (edge, from, to) in [
            (&rim.top, 0.0, full_height / 2.0),
            (&rim.bottom, full_height / 2.0, full_height),
        ] {
            let Some(along) = linear((pixel, 0.0), (pixel + width, 0.0), &along(edge, rim, reach))
            else {
                continue;
            };
            let (paint, outline) = line(
                shaders::blend(BlendMode::Modulate, along, mask.clone()),
                (depth as f32 + 0.5) * pixel,
            );
            canvas.save();
            canvas.clip_rect(
                SkRect::new(0.0, from, full_width, to),
                ClipOp::Intersect,
                false,
            );
            canvas.draw_path(&outline, &paint);
            canvas.restore();
        }
    }
    canvas.restore();

    canvas.save();
    canvas.clip_path(&path(0.0), ClipOp::Difference, true);
    if let Some(shader) = linear(
        (0.0, pixel),
        (0.0, pixel + height),
        &down(rim.outline, &ctdesign::dock_rim_dark_share),
    ) {
        let (paint, outline) = line(shader, -0.5 * pixel);
        canvas.draw_path(&outline, &paint);
    }
    canvas.restore();
}

const WHITE: Rgba = Rgba {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

// The edge's colours spread over the bar, with the sides at the ends and where each corner starts.
fn along(edge: &[Rgba], rim: &GlassRim, reach: f32) -> Vec<(f32, Rgba)> {
    let step = (1.0 - 2.0 * reach) / edge.len().max(1) as f32;
    std::iter::once((0.0, rim.left))
        .chain(
            edge.iter()
                .enumerate()
                .map(|(index, colour)| (reach + (index as f32 + 0.5) * step, *colour)),
        )
        .chain(std::iter::once((1.0, rim.right)))
        .collect()
}

fn glass_line(
    context: &mut FillRenderContext,
    fill: Rgba,
    line: ctdesign::PanelLine,
    corner: CornerRadius,
) {
    use freya_engine::prelude::Rect as SkRect;
    context.set_color(color(fill));
    let RenderTarget::Background { canvas, .. } = &context.target else {
        return;
    };
    let (width, height) = (context.size.width, context.size.height);
    let bounds = SkRect::from_wh(width, height);
    let radius = corner.top_left();
    let curve = (radius * WINDOW_CORNER.expansion / height).min(0.5);
    let depth = (ctdesign::PANEL_GLOW_DEPTH / height).min(0.5);
    let clear = Rgba {
        a: 0.0,
        ..line.glow
    };
    canvas.draw_rect(
        bounds,
        &vertical(
            height,
            &[
                (0.0, line.glow),
                (depth, clear),
                (1.0 - depth, clear),
                (1.0, line.glow),
            ],
        ),
    );
    let paint = vertical(
        height,
        &[
            (0.0, line.edges),
            (curve, line.sides),
            (1.0 - curve, line.sides),
            (1.0, line.edges),
        ],
    );
    stroke_edge(context, corner, paint);
}

fn stroke_edge(
    context: &FillRenderContext,
    corner: CornerRadius,
    mut paint: freya_engine::prelude::Paint,
) {
    use freya_engine::prelude::{PaintStyle, PathBuilder, RRect, Rect as SkRect};
    let RenderTarget::Background { canvas, .. } = &context.target else {
        return;
    };
    let bounds = SkRect::from_wh(context.size.width, context.size.height);
    let radius = corner.top_left();
    paint.set_style(PaintStyle::Stroke);
    paint.set_anti_alias(true);
    // One physical pixel on each side of the edge; the clip keeps the inner half, as the compositor's line.
    paint.set_stroke_width(2.0 / context.scale_factor as f32);
    // Freya's smoothed path leaves its last contour open: a fill closes it, a stroke does not.
    let mut outline = PathBuilder::new();
    outline
        .add_path(
            &corner.smoothed_path(RRect::new_rect_xy(bounds, radius, radius)),
            None,
        )
        .close();
    canvas.draw_path(&outline.detach(), &paint);
}

fn vertical(height: f32, stops: &[(f32, Rgba)]) -> freya_engine::prelude::Paint {
    let mut paint = freya_engine::prelude::Paint::default();
    paint.set_dither(true);
    paint.set_shader(linear((0.0, 0.0), (0.0, height), stops));
    paint
}

pub(crate) fn linear(
    from: (f32, f32),
    to: (f32, f32),
    stops: &[(f32, Rgba)],
) -> Option<freya_engine::prelude::Shader> {
    use freya_engine::prelude::{Color4f, Colors, Gradient, Interpolation, TileMode, shaders};
    let (colors, offsets): (Vec<Color4f>, Vec<f32>) = stops
        .iter()
        .map(|&(offset, c)| (Color4f::new(c.r, c.g, c.b, c.a), offset))
        .unzip();
    let gradient = Gradient::new(
        Colors::new(&colors, Some(&offsets), TileMode::Clamp, None),
        Interpolation::default(),
    );
    shaders::linear_gradient((from, to), &gradient, None)
}

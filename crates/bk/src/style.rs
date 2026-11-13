use std::borrow::Cow;

use ctdesign::{Font, Rgba, WINDOW_CORNER};
use freya_core::prelude::*;

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

/// The Dock's rim: the light along the inner edge of its glass, as colours (with their opacity)
/// read from the backdrop. `top` and `bottom` run left to right along those edges; `left` and
/// `right` are the sides.
#[derive(Clone, Debug, PartialEq)]
pub struct GlassRim {
    pub top: Vec<Rgba>,
    pub bottom: Vec<Rgba>,
    pub left: Rgba,
    pub right: Rgba,
}

/// A card filled with `fill` whose only frame is `rim`: a soft line, full on top, thinning down the
/// sides, gone around the lower corners and back along the bottom (`ctdesign::dock_rim_fade`).
pub fn glass_edge(body: Rect, radius: f32, fill: Rgba, rim: GlassRim) -> Rect {
    let corner = continuous_corner(radius);
    body.corner_radius(corner)
        .background(Fill::Callback(RenderCallback::new(move |context| {
            glass_rim(context, fill, &rim, corner)
        })))
}

fn glass_rim(context: &mut FillRenderContext, fill: Rgba, rim: &GlassRim, corner: CornerRadius) {
    use freya_engine::prelude::{
        BlendMode, ClipOp, Paint, Rect as SkRect, SkBlurStyle, SkMaskFilter, shaders,
    };
    context.set_color(color(fill));
    let RenderTarget::Background { canvas, .. } = &context.target else {
        return;
    };
    let (width, height) = (context.size.width, context.size.height);
    let scale = context.scale_factor as f32;
    let radius = corner.top_left();
    let white = |at: f32, visible: f32| {
        (
            at,
            Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: visible,
            },
        )
    };
    let corner_share = (radius * WINDOW_CORNER.expansion / height).min(0.5);
    let fade: Vec<_> = ctdesign::dock_rim_fade(corner_share)
        .into_iter()
        .map(|(at, visible)| white(at, visible))
        .collect();
    let Some(fade) = linear((0.0, 0.0), (0.0, height), &fade) else {
        return;
    };
    let reach = (radius / width).min(0.5);
    for (edge, from, to) in [
        (&rim.top, 0.0, height / 2.0),
        (&rim.bottom, height / 2.0, height),
    ] {
        let Some(along) = linear((0.0, 0.0), (width, 0.0), &along(edge, rim, reach)) else {
            continue;
        };
        let mut paint = Paint::default();
        paint.set_shader(shaders::blend(BlendMode::Modulate, along, fade.clone()));
        paint.set_mask_filter(SkMaskFilter::blur(
            SkBlurStyle::Normal,
            ctdesign::DOCK_RIM_BLUR / scale,
            false,
        ));
        canvas.save();
        canvas.clip_rect(SkRect::new(0.0, from, width, to), ClipOp::Intersect, false);
        stroke_edge(context, corner, paint);
        canvas.restore();
    }
}

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

fn linear(
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

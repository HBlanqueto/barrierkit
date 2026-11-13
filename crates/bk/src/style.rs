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
/// to right along those edges; `left` and `right` are the colours at the sides, which the corners
/// take. `corner` is `DockSurface::corner`.
#[derive(Clone, Debug, PartialEq)]
pub struct GlassRim {
    pub top: Vec<Rgba>,
    pub bottom: Vec<Rgba>,
    pub left: Rgba,
    pub right: Rgba,
    pub outline: Rgba,
    pub corner: [f32; 3],
}

/// A glass of `width` by `height` filled with `fill` and framed as macOS frames the Dock: a dark
/// outline one pixel outside it, strongest on the sides, and a light rim one pixel inside it with a
/// glow that dies away inwards, strongest on top and bottom. Around each corner the rim dies away
/// and the outline takes over, by the angle of the frame (`ctdesign::dock_rim_light_share`).
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

// The corners of a frame: where each one's square sits, the centre its angle is measured from, the
// sweep it covers (degrees from the +x axis, clockwise), whether the angle from vertical
// (0 on top and bottom, 90 on the sides) runs against the sweep, and which side's colour it takes.
struct Quarter {
    square: freya_engine::prelude::Rect,
    centre: (f32, f32),
    start: f32,
    against: bool,
    left: bool,
}

fn glass_frame(context: &mut FillRenderContext, fill: Rgba, rim: &GlassRim, corner: CornerRadius) {
    use freya_engine::prelude::{
        ClipOp, Paint, PaintStyle, PathBuilder, RRect, Rect as SkRect, Shader,
    };
    let RenderTarget::Background { canvas, .. } = &context.target else {
        return;
    };
    let pixel = 1.0 / context.scale_factor as f32;
    let (full_width, full_height) = (context.size.width, context.size.height);
    let (width, height) = (full_width - 2.0 * pixel, full_height - 2.0 * pixel);
    let radius = corner.top_left();
    // How far a continuous corner reaches along each edge.
    let reach = radius * WINDOW_CORNER.expansion;
    let (left, top, right, bottom) = (pixel, pixel, pixel + width, pixel + height);
    let glass = SkRect::from_xywh(left, top, width, height);
    // The glass shape, `inset` logical pixels in (negative: out).
    let path = |inset: f32| {
        let rounding = (radius - inset).max(0.0);
        let mut outline = PathBuilder::new();
        outline
            .add_path(
                &continuous_corner(rounding).smoothed_path(RRect::new_rect_xy(
                    glass.with_inset((inset, inset)),
                    rounding,
                    rounding,
                )),
                None,
            )
            .close();
        outline.detach()
    };
    let stroke = |clip: SkRect, shader: Option<Shader>, inset: f32| {
        let Some(shader) = shader else {
            return;
        };
        let mut paint = Paint::default();
        paint.set_shader(shader);
        paint.set_style(PaintStyle::Stroke);
        paint.set_anti_alias(true);
        paint.set_stroke_width(pixel);
        canvas.save();
        canvas.clip_rect(clip, ClipOp::Intersect, false);
        canvas.draw_path(&path(inset), &paint);
        canvas.restore();
    };
    let quarters = [
        Quarter {
            square: SkRect::new(0.0, 0.0, left + reach, top + reach),
            centre: (left + reach, top + reach),
            start: 180.0,
            against: true,
            left: true,
        },
        Quarter {
            square: SkRect::new(right - reach, 0.0, full_width, top + reach),
            centre: (right - reach, top + reach),
            start: 270.0,
            against: false,
            left: false,
        },
        Quarter {
            square: SkRect::new(right - reach, bottom - reach, full_width, full_height),
            centre: (right - reach, bottom - reach),
            start: 0.0,
            against: true,
            left: false,
        },
        Quarter {
            square: SkRect::new(0.0, bottom - reach, left + reach, full_height),
            centre: (left + reach, bottom - reach),
            start: 90.0,
            against: false,
            left: true,
        },
    ];
    let quarter = |at: &Quarter, colour: Rgba, share: &dyn Fn(f32) -> f32| {
        let stops: Vec<(f32, Rgba)> = (0..=18)
            .map(|step| {
                let along = step as f32 / 18.0;
                let angle = 90.0 * if at.against { 1.0 - along } else { along };
                (
                    along,
                    Rgba {
                        a: colour.a * share(angle),
                        ..colour
                    },
                )
            })
            .collect();
        sweep(at.centre, (at.start, at.start + 90.0), &stops)
    };

    canvas.save();
    canvas.clip_path(&path(0.0), ClipOp::Intersect, true);
    let mut paint = Paint::default();
    paint.set_color(color(fill));
    canvas.draw_paint(&paint);
    let strengths = std::iter::once(1.0).chain(ctdesign::DOCK_RIM_GLOW);
    for (depth, strength) in strengths.enumerate() {
        let inset = (depth as f32 + 0.5) * pixel;
        let reach_share = (reach / width).min(0.5);
        let ends = (rim.left, rim.right);
        stroke(
            SkRect::new(left + reach, 0.0, right - reach, full_height / 2.0),
            linear(
                (left, 0.0),
                (right, 0.0),
                &along(&rim.top, ends, reach_share, strength),
            ),
            inset,
        );
        stroke(
            SkRect::new(left + reach, full_height / 2.0, right - reach, full_height),
            linear(
                (left, 0.0),
                (right, 0.0),
                &along(&rim.bottom, ends, reach_share, strength),
            ),
            inset,
        );
        for at in &quarters {
            let colour = if at.left { rim.left } else { rim.right };
            let lit = |angle| strength * ctdesign::dock_rim_light_share(rim.corner, angle);
            stroke(at.square, quarter(at, colour, &lit), inset);
        }
    }
    canvas.restore();

    canvas.save();
    canvas.clip_path(&path(0.0), ClipOp::Difference, true);
    let outset = -0.5 * pixel;
    stroke(
        SkRect::new(0.0, top + reach, left + reach, bottom - reach),
        linear(
            (0.0, 0.0),
            (1.0, 0.0),
            &[(0.0, rim.outline), (1.0, rim.outline)],
        ),
        outset,
    );
    stroke(
        SkRect::new(right - reach, top + reach, full_width, bottom - reach),
        linear(
            (0.0, 0.0),
            (1.0, 0.0),
            &[(0.0, rim.outline), (1.0, rim.outline)],
        ),
        outset,
    );
    for at in &quarters {
        let dark = |angle| ctdesign::dock_rim_dark_share(rim.corner, angle);
        stroke(at.square, quarter(at, rim.outline, &dark), outset);
    }
    canvas.restore();
}

// The edge's colours spread over the bar, with the sides at the ends and where each corner starts,
// all `strength` as opaque.
fn along(
    edge: &[Rgba],
    (left, right): (Rgba, Rgba),
    reach: f32,
    strength: f32,
) -> Vec<(f32, Rgba)> {
    let step = (1.0 - 2.0 * reach) / edge.len().max(1) as f32;
    let weaker = |colour: Rgba| Rgba {
        a: colour.a * strength,
        ..colour
    };
    std::iter::once((0.0, left))
        .chain(
            edge.iter()
                .enumerate()
                .map(|(index, colour)| (reach + (index as f32 + 0.5) * step, *colour)),
        )
        .chain(std::iter::once((1.0, right)))
        .map(|(at, colour)| (at, weaker(colour)))
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
    use freya_engine::prelude::shaders;
    gradient(stops, |gradient| {
        shaders::linear_gradient((from, to), gradient, None)
    })
}

// `angles` are degrees from the +x axis, clockwise; the stops run from the first to the second.
fn sweep(
    centre: (f32, f32),
    angles: (f32, f32),
    stops: &[(f32, Rgba)],
) -> Option<freya_engine::prelude::Shader> {
    use freya_engine::prelude::shaders;
    gradient(stops, |gradient| {
        shaders::sweep_gradient(centre, angles, gradient, None)
    })
}

fn gradient(
    stops: &[(f32, Rgba)],
    shader: impl FnOnce(&freya_engine::prelude::Gradient) -> Option<freya_engine::prelude::Shader>,
) -> Option<freya_engine::prelude::Shader> {
    use freya_engine::prelude::{Color4f, Colors, Gradient, Interpolation, TileMode};
    let (colors, offsets): (Vec<Color4f>, Vec<f32>) = stops
        .iter()
        .map(|&(offset, c)| (Color4f::new(c.r, c.g, c.b, c.a), offset))
        .unzip();
    let gradient = Gradient::new(
        Colors::new(&colors, Some(&offsets), TileMode::Clamp, None),
        Interpolation::default(),
    );
    shader(&gradient)
}

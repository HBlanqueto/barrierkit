use std::borrow::Cow;

use ctdesign::{CHART, Rgba};
use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{Paint, PaintStyle, PathBuilder, Rect as SkRect};

use super::scale::{Linear, Window, monotone_controls, stack};
use super::{Interpolation, MarkKind, Series};

#[derive(PartialEq)]
pub(super) struct Plot {
    pub kind: MarkKind,
    pub series: Vec<Series>,
    pub colors: Vec<Rgba>,
    pub stacked: bool,
    pub interpolation: Interpolation,
    pub y_domain: (f64, f64),
    pub ticks: Vec<f64>,
    pub rules: Vec<(f64, Rgba)>,
    pub grid: Rgba,
    pub hover: Option<usize>,
    pub hover_ink: Rgba,
    pub halo: Rgba,
    pub window: Window,
    pub shift: f64,
    pub horizontal: bool,
    pub layout: LayoutData,
}

pub(super) fn position(kind: MarkKind, index: usize) -> f64 {
    match kind {
        MarkKind::Bar => index as f64 + 0.5,
        _ => index as f64,
    }
}

pub(super) fn paint(color: Rgba, style: PaintStyle, width: f32) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(style);
    paint.set_stroke_width(width);
    paint.set_color(crate::theme::color(color));
    paint
}

pub(super) fn mix(a: Rgba, b: Rgba, t: f32) -> Rgba {
    Rgba {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

pub(super) fn shade(color: Rgba, light: f32) -> Rgba {
    Rgba {
        r: color.r * light,
        g: color.g * light,
        b: color.b * light,
        ..color
    }
}

pub(super) fn normalized(values: &[f64]) -> (f64, f64) {
    let min = values.iter().copied().fold(f64::MAX, f64::min);
    let max = values.iter().copied().fold(f64::MIN, f64::max);
    (min, (max - min).max(f64::EPSILON))
}

impl Plot {
    fn bands(&self) -> Vec<Vec<(f64, f64)>> {
        let values: Vec<Vec<f64>> = self.series.iter().map(|s| s.values.clone()).collect();
        if self.stacked {
            stack(&values)
        } else {
            values
                .into_iter()
                .map(|v| {
                    v.into_iter()
                        .map(|y| (self.y_domain.0.max(0.0).min(y), y))
                        .collect()
                })
                .collect()
        }
    }

    fn count(&self) -> usize {
        self.series
            .iter()
            .map(|s| s.values.len())
            .max()
            .unwrap_or(0)
    }
}

impl ElementExt for Plot {
    fn changed(&self, other: &std::rc::Rc<dyn ElementExt>) -> bool {
        !crate::style_diff(self, other).is_empty()
    }

    fn diff(&self, other: &std::rc::Rc<dyn ElementExt>) -> DiffModifies {
        crate::style_diff(self, other)
    }

    fn layout(&'_ self) -> Cow<'_, LayoutData> {
        Cow::Borrowed(&self.layout)
    }

    fn render(&self, context: RenderContext) {
        let area = context.layout_node.visible_area();
        let scale = context.scale_factor as f32;
        let canvas = context.canvas;
        let (left, top, right, bottom) = (area.min_x(), area.min_y(), area.max_x(), area.max_y());
        let y = Linear {
            domain: self.y_domain,
            range: (f64::from(bottom), f64::from(top)),
        };
        let count = self.count();
        canvas.save();
        canvas.clip_rect(SkRect::new(left, top, right, bottom), None, Some(true));
        if self.horizontal {
            self.horizontal_bars(canvas, (left, top, right, bottom), scale);
            canvas.restore();
            return;
        }
        let width = right - left;
        let band = width / self.window.span as f32;
        let x = |i: usize| {
            left + self.window.fraction(position(self.kind, i) + self.shift) as f32 * width
        };

        let grid = paint(self.grid, PaintStyle::Stroke, CHART.grid_width * scale);
        for tick in &self.ticks {
            let ty = y.map(*tick) as f32;
            canvas.draw_line((left, ty), (right, ty), &grid);
        }

        let bands = self.bands();
        let groups = if self.stacked { 1 } else { bands.len().max(1) };
        for (s, values) in bands.iter().enumerate() {
            let color = self.colors[s % self.colors.len()];
            let points: Vec<(f64, f64)> = values
                .iter()
                .enumerate()
                .map(|(i, (_, upper))| (f64::from(x(i)), y.map(*upper)))
                .collect();
            match self.kind {
                MarkKind::Bar => {
                    let slot = band * (1.0 - CHART.bar_gap);
                    let width = slot / groups as f32;
                    let offset = if self.stacked { 0.0 } else { width * s as f32 };
                    let fill = paint(color, PaintStyle::Fill, 0.0);
                    for (i, (lower, upper)) in values.iter().enumerate() {
                        let x0 = x(i) - slot / 2.0 + offset;
                        let (y0, y1) = (y.map(*upper) as f32, y.map(*lower) as f32);
                        let rect = SkRect::new(x0, y0.min(y1), x0 + width, y0.max(y1));
                        let radius = (CHART.bar_radius * scale).min(width / 2.0);
                        canvas.draw_round_rect(rect, radius, radius, &fill);
                    }
                }
                MarkKind::Point => {
                    let fill = paint(color, PaintStyle::Fill, 0.0);
                    for (px, py) in &points {
                        canvas.draw_circle(
                            (*px as f32, *py as f32),
                            CHART.point_size * scale / 2.0,
                            &fill,
                        );
                    }
                }
                MarkKind::Line | MarkKind::Area => {
                    let line = self.line(&points);
                    if self.kind == MarkKind::Area {
                        let lowers: Vec<(f64, f64)> = values
                            .iter()
                            .enumerate()
                            .map(|(i, (lower, _))| (f64::from(x(i)), y.map(*lower)))
                            .collect();
                        let mut area = line.clone();
                        for (lx, ly) in lowers.iter().rev() {
                            area.line_to((*lx as f32, *ly as f32));
                        }
                        area.close();
                        let fill = paint(
                            Rgba {
                                a: color.a * CHART.area_opacity,
                                ..color
                            },
                            PaintStyle::Fill,
                            0.0,
                        );
                        canvas.draw_path(&area.detach(), &fill);
                    }
                    let mut stroke = paint(color, PaintStyle::Stroke, CHART.line_width * scale);
                    stroke.set_stroke_join(skia_safe::paint::Join::Round);
                    stroke.set_stroke_cap(skia_safe::paint::Cap::Round);
                    canvas.draw_path(&line.snapshot(), &stroke);
                }
            }
        }

        for (value, color) in &self.rules {
            let ry = y.map(*value) as f32;
            let rule = paint(*color, PaintStyle::Stroke, CHART.rule_width * scale);
            canvas.draw_line((left, ry), (right, ry), &rule);
        }

        if let Some(i) = self.hover.filter(|i| *i < count) {
            let hx = x(i);
            let rule = paint(self.hover_ink, PaintStyle::Stroke, CHART.rule_width * scale);
            canvas.draw_line((hx, top), (hx, bottom), &rule);
            if self.kind != MarkKind::Bar {
                for (s, values) in bands.iter().enumerate() {
                    let Some((_, upper)) = values.get(i) else {
                        continue;
                    };
                    let color = self.colors[s % self.colors.len()];
                    let center = (hx, y.map(*upper) as f32);
                    let radius = CHART.point_size * scale / 2.0;
                    canvas.draw_circle(
                        center,
                        radius + CHART.rule_width * scale,
                        &paint(self.halo, PaintStyle::Fill, 0.0),
                    );
                    canvas.draw_circle(center, radius, &paint(color, PaintStyle::Fill, 0.0));
                }
            }
        }
        canvas.restore();
    }
}

impl Plot {
    fn horizontal_bars(
        &self,
        canvas: &skia_safe::Canvas,
        (left, top, right, bottom): (f32, f32, f32, f32),
        scale: f32,
    ) {
        let x = Linear {
            domain: self.y_domain,
            range: (f64::from(left), f64::from(right)),
        };
        let grid = paint(self.grid, PaintStyle::Stroke, CHART.grid_width * scale);
        for tick in &self.ticks {
            let tx = x.map(*tick) as f32;
            canvas.draw_line((tx, top), (tx, bottom), &grid);
        }
        let bands = self.bands();
        let count = self.count();
        let band = (bottom - top) / count.max(1) as f32;
        let groups = if self.stacked { 1 } else { bands.len().max(1) };
        let slot = band * (1.0 - CHART.bar_gap);
        let thickness = slot / groups as f32;
        for (s, values) in bands.iter().enumerate() {
            let fill = paint(self.colors[s % self.colors.len()], PaintStyle::Fill, 0.0);
            let offset = if self.stacked {
                0.0
            } else {
                thickness * s as f32
            };
            for (i, (lower, upper)) in values.iter().enumerate() {
                let y0 = top + band * (i as f32 + 0.5) - slot / 2.0 + offset;
                let (x0, x1) = (x.map(*lower) as f32, x.map(*upper) as f32);
                let rect = SkRect::new(x0.min(x1), y0, x0.max(x1), y0 + thickness);
                let radius = (CHART.bar_radius * scale).min(thickness / 2.0);
                canvas.draw_round_rect(rect, radius, radius, &fill);
            }
        }
        for (value, color) in &self.rules {
            let rx = x.map(*value) as f32;
            canvas.draw_line(
                (rx, top),
                (rx, bottom),
                &paint(*color, PaintStyle::Stroke, CHART.rule_width * scale),
            );
        }
        if let Some(i) = self.hover.filter(|i| *i < count) {
            let mut highlight = paint(self.hover_ink, PaintStyle::Fill, 0.0);
            highlight.set_alpha_f(0.12);
            canvas.draw_rect(
                SkRect::new(
                    left,
                    top + band * i as f32,
                    right,
                    top + band * (i + 1) as f32,
                ),
                &highlight,
            );
        }
    }

    fn line(&self, points: &[(f64, f64)]) -> PathBuilder {
        let mut path = PathBuilder::new();
        let Some(first) = points.first() else {
            return path;
        };
        path.move_to((first.0 as f32, first.1 as f32));
        match self.interpolation {
            Interpolation::Linear => {
                for (px, py) in &points[1..] {
                    path.line_to((*px as f32, *py as f32));
                }
            }
            Interpolation::Step => {
                for pair in points.windows(2) {
                    path.line_to((pair[1].0 as f32, pair[0].1 as f32));
                    path.line_to((pair[1].0 as f32, pair[1].1 as f32));
                }
            }
            Interpolation::Monotone => {
                for (controls, end) in monotone_controls(points).iter().zip(&points[1..]) {
                    path.cubic_to(
                        (controls[0].0 as f32, controls[0].1 as f32),
                        (controls[1].0 as f32, controls[1].1 as f32),
                        (end.0 as f32, end.1 as f32),
                    );
                }
            }
        }
        path
    }
}

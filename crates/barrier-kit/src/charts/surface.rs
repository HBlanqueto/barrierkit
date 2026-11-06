use std::borrow::Cow;

use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{PaintStyle, PathBuilder, Rect as SkRect};
use torin::prelude::{Position, Size};
use vow_design::{CHART, Rgba};

use super::plot::paint;
use super::series_color;
use super::space::{Pose, back_to_front, project};
use crate::style::Palette;

fn mix(a: Rgba, b: Rgba, t: f32) -> Rgba {
    Rgba {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

fn shade(color: Rgba, light: f32) -> Rgba {
    Rgba {
        r: color.r * light,
        g: color.g * light,
        b: color.b * light,
        ..color
    }
}

fn normalized(values: &[f64]) -> (f64, f64) {
    let min = values.iter().copied().fold(f64::MAX, f64::min);
    let max = values.iter().copied().fold(f64::MIN, f64::max);
    (min, (max - min).max(f64::EPSILON))
}

#[derive(PartialEq)]
struct Heat {
    rows: Vec<Vec<f64>>,
    color: Rgba,
    empty: Rgba,
    layout: LayoutData,
}

impl ElementExt for Heat {
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
        let values: Vec<f64> = self.rows.iter().flatten().copied().collect();
        let (min, span) = normalized(&values);
        let rows = self.rows.len().max(1) as f32;
        let cols = self.rows.iter().map(Vec::len).max().unwrap_or(1).max(1) as f32;
        let gap = CHART.heat_gap * scale;
        let (w, h) = (
            (area.width() - gap * (cols - 1.0)) / cols,
            (area.height() - gap * (rows - 1.0)) / rows,
        );
        let radius = (CHART.heat_radius * scale).min(w.min(h) / 2.0);
        for (r, row) in self.rows.iter().enumerate() {
            for (c, value) in row.iter().enumerate() {
                let t = ((value - min) / span) as f32;
                let x = area.min_x() + c as f32 * (w + gap);
                let y = area.min_y() + r as f32 * (h + gap);
                let fill = paint(mix(self.empty, self.color, t), PaintStyle::Fill, 0.0);
                context.canvas.draw_round_rect(
                    SkRect::new(x, y, x + w, y + h),
                    radius,
                    radius,
                    &fill,
                );
            }
        }
    }
}

#[derive(PartialEq)]
pub struct HeatMap {
    label: &'static str,
    rows: Vec<Vec<f64>>,
    height: f32,
}

impl HeatMap {
    pub fn new(label: &'static str, rows: Vec<Vec<f64>>) -> Self {
        Self {
            label,
            rows,
            height: CHART.height,
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl Component for HeatMap {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        rect()
            .width(Size::fill())
            .height(Size::px(self.height))
            .a11y_role(AccessibilityRole::Figure)
            .a11y_alt(self.label)
            .child(crate::glass::leaf(Heat {
                rows: self.rows.clone(),
                color: palette.accent,
                empty: palette.control_background,
                layout: crate::glass::layout(Size::fill(), Size::fill(), Position::default()),
            }))
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Plot3D {
    Surface(Vec<Vec<f64>>),
    Points(Vec<[f64; 3]>),
}

#[derive(PartialEq)]
struct Space {
    plot: Plot3D,
    pose: Pose,
    low: Rgba,
    high: Rgba,
    frame: Rgba,
    layout: LayoutData,
}

const BOX_EDGES: [(usize, usize); 12] = [
    (0, 1),
    (1, 3),
    (3, 2),
    (2, 0),
    (4, 5),
    (5, 7),
    (7, 6),
    (6, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];

impl ElementExt for Space {
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
        let (cx, cy) = (area.center().x, area.center().y);
        let unit = area.width().min(area.height()) * 0.32;
        let screen = |p: [f64; 3]| -> ([f32; 2], f64) {
            let [x, y, z] = project(p, self.pose);
            ([cx + x as f32 * unit, cy - y as f32 * unit], z)
        };

        let corners: Vec<[f32; 2]> = (0..8)
            .map(|i| {
                let corner = [
                    if i & 1 == 0 { -1.0 } else { 1.0 },
                    if i & 4 == 0 { -1.0 } else { 1.0 },
                    if i & 2 == 0 { -1.0 } else { 1.0 },
                ];
                screen(corner).0
            })
            .collect();
        let frame = paint(self.frame, PaintStyle::Stroke, CHART.grid_width * scale);
        for (a, b) in BOX_EDGES {
            canvas.draw_line(
                (corners[a][0], corners[a][1]),
                (corners[b][0], corners[b][1]),
                &frame,
            );
        }

        match &self.plot {
            Plot3D::Surface(grid) => {
                let values: Vec<f64> = grid.iter().flatten().copied().collect();
                let (min, span) = normalized(&values);
                let rows = grid.len();
                let cols = grid.iter().map(Vec::len).min().unwrap_or(0);
                if rows < 2 || cols < 2 {
                    return;
                }
                let vertex = |r: usize, c: usize| {
                    [
                        c as f64 / (cols - 1) as f64 * 2.0 - 1.0,
                        (grid[r][c] - min) / span * 2.0 - 1.0,
                        r as f64 / (rows - 1) as f64 * 2.0 - 1.0,
                    ]
                };
                let mut faces = Vec::with_capacity((rows - 1) * (cols - 1));
                for r in 0..rows - 1 {
                    for c in 0..cols - 1 {
                        let quad = [
                            vertex(r, c),
                            vertex(r, c + 1),
                            vertex(r + 1, c + 1),
                            vertex(r + 1, c),
                        ];
                        let projected: Vec<([f32; 2], f64)> =
                            quad.iter().map(|p| screen(*p)).collect();
                        let depth = projected.iter().map(|(_, z)| z).sum::<f64>() / 4.0;
                        let height = quad.iter().map(|p| p[1]).sum::<f64>() / 4.0;
                        let (u, v) = (sub(quad[1], quad[0]), sub(quad[3], quad[0]));
                        let normal = project(cross(v, u), self.pose);
                        let facing = (normal[2] / length(normal).max(f64::EPSILON)).abs();
                        faces.push((projected, depth, height, facing));
                    }
                }
                let depths: Vec<f64> = faces.iter().map(|f| f.1).collect();
                for i in back_to_front(&depths) {
                    let (points, _, height, facing) = &faces[i];
                    let t = ((height + 1.0) / 2.0) as f32;
                    let light = 0.55 + 0.45 * *facing as f32;
                    let color = shade(mix(self.low, self.high, t), light);
                    let mut path = PathBuilder::new();
                    let pts: Vec<_> = points.iter().map(|(p, _)| (p[0], p[1]).into()).collect();
                    path.add_polygon(&pts, true);
                    let path = path.detach();
                    canvas.draw_path(&path, &paint(color, PaintStyle::Fill, 0.0));
                    canvas.draw_path(
                        &path,
                        &paint(
                            shade(color, 0.85),
                            PaintStyle::Stroke,
                            CHART.grid_width * scale,
                        ),
                    );
                }
            }
            Plot3D::Points(points) => {
                let ys: Vec<f64> = points.iter().map(|p| p[1]).collect();
                let (min, span) = normalized(&ys);
                let projected: Vec<([f32; 2], f64, f32)> = points
                    .iter()
                    .map(|p| {
                        let (xy, z) = screen(*p);
                        (xy, z, ((p[1] - min) / span) as f32)
                    })
                    .collect();
                let depths: Vec<f64> = projected.iter().map(|p| p.1).collect();
                for i in back_to_front(&depths) {
                    let (xy, z, t) = projected[i];
                    let radius = CHART.point_size * scale / 2.0 * (1.0 + 0.2 * z as f32);
                    canvas.draw_circle(
                        (xy[0], xy[1]),
                        radius,
                        &paint(mix(self.low, self.high, t), PaintStyle::Fill, 0.0),
                    );
                }
            }
        }
    }
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(a: [f64; 3]) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

#[derive(PartialEq)]
pub struct Chart3D {
    label: &'static str,
    plot: Plot3D,
    height: f32,
}

impl Chart3D {
    pub fn surface(label: &'static str, grid: Vec<Vec<f64>>) -> Self {
        Self {
            label,
            plot: Plot3D::Surface(grid),
            height: CHART.height * 1.5,
        }
    }

    pub fn points(label: &'static str, points: Vec<[f64; 3]>) -> Self {
        Self {
            label,
            plot: Plot3D::Points(points),
            height: CHART.height * 1.5,
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl Component for Chart3D {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let mut pose = use_state(|| Pose {
            azimuth: f64::from(CHART.azimuth),
            inclination: f64::from(CHART.inclination),
        });
        let mut drag = use_state(|| None::<(f64, f64, Pose)>);
        rect()
            .width(Size::fill())
            .height(Size::px(self.height))
            .a11y_role(AccessibilityRole::Figure)
            .a11y_alt(self.label)
            .on_pointer_down(move |event: Event<PointerEventData>| {
                let at = event.global_location();
                drag.set(Some((at.x, at.y, *pose.peek())));
            })
            .on_global_pointer_move(move |event: Event<PointerEventData>| {
                let Some((x, y, start)) = *drag.peek() else {
                    return;
                };
                let at = event.global_location();
                let degrees = f64::from(CHART.drag_degrees);
                pose.set(Pose {
                    azimuth: start.azimuth + (at.x - x) * degrees,
                    inclination: (start.inclination + (at.y - y) * degrees).clamp(-90.0, 90.0),
                });
            })
            .on_global_pointer_press(move |_| drag.set_if_modified(None))
            .child(crate::glass::leaf(Space {
                plot: self.plot.clone(),
                pose: *pose.read(),
                low: series_color(0),
                high: series_color(2),
                frame: palette.separator,
                layout: crate::glass::layout(Size::fill(), Size::fill(), Position::default()),
            }))
    }
}

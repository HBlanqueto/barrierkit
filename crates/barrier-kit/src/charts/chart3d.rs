use std::borrow::Cow;

use ct_design::{CALLOUT, CHART, Rgba, SPACE_2};
use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{BlendMode, Paint, PaintStyle, Point};
use skia_safe::Vertices;
use skia_safe::vertices::VertexMode;
use torin::prelude::{Alignment, Direction, Position, Size};

use super::plot::{mix, normalized, paint, shade};
use super::series_color;
use super::space::{
    Camera, Pose, View, back_to_front, fit, from_unit, project, surface_triangles, to_unit,
};
use crate::controls::{Button, ControlSize, SegmentedControl};
use crate::i18n::text;
use crate::level::LevelSlider;
use crate::style::Palette;

#[derive(Clone, Debug, PartialEq)]
enum Plot3D {
    Mesh {
        vertices: Vec<[f64; 3]>,
        triangles: Vec<[u32; 3]>,
        closed: bool,
    },
    Points(Vec<[f64; 3]>),
}

#[derive(PartialEq)]
struct Space {
    plot: Plot3D,
    camera: Camera,
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

impl Space {
    fn mesh(&self, canvas: &skia_safe::Canvas, screen: impl Fn([f64; 3]) -> Point) {
        let Plot3D::Mesh {
            vertices,
            triangles,
            closed,
        } = &self.plot
        else {
            return;
        };
        let projected: Vec<[f64; 3]> = vertices
            .iter()
            .map(|v| project(*v, self.camera.pose))
            .collect();
        let mut normals = vec![[0.0; 3]; vertices.len()];
        let mut faces = Vec::with_capacity(triangles.len());
        for tri in triangles {
            let [a, b, c] = tri.map(|i| vertices[i as usize]);
            let normal = cross(sub(b, a), sub(c, a));
            for corner in tri {
                let n = &mut normals[*corner as usize];
                *n = [n[0] + normal[0], n[1] + normal[1], n[2] + normal[2]];
            }
            let facing = project(normal, self.camera.pose)[2];
            if *closed && facing <= 0.0 {
                continue;
            }
            let depth = tri.iter().map(|i| projected[*i as usize][2]).sum::<f64>() / 3.0;
            faces.push((tri, depth));
        }
        let lit: Vec<skia_safe::Color> = vertices
            .iter()
            .zip(&normals)
            .map(|(vertex, normal)| {
                let toward = project(*normal, self.camera.pose);
                let facing = (toward[2] / length(toward).max(f64::EPSILON)).abs() as f32;
                let height = ((vertex[1] + 1.0) / 2.0) as f32;
                let Rgba { r, g, b, a } =
                    shade(mix(self.low, self.high, height), 0.35 + 0.65 * facing);
                let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                skia_safe::Color::from_argb(byte(a), byte(r), byte(g), byte(b))
            })
            .collect();
        let depths: Vec<f64> = faces.iter().map(|f| f.1).collect();
        let order = back_to_front(&depths);
        let mut positions = Vec::with_capacity(order.len() * 3);
        let mut colors = Vec::with_capacity(order.len() * 3);
        for i in order {
            for corner in faces[i].0 {
                positions.push(screen(projected[*corner as usize]));
                colors.push(lit[*corner as usize]);
            }
        }
        if positions.is_empty() {
            return;
        }
        // One indexed-free triangle list: Skia hands the whole mesh to the GPU in a single draw.
        let mesh = Vertices::new_copy(VertexMode::Triangles, &positions, &positions, &colors, None);
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        canvas.draw_vertices(&mesh, BlendMode::Dst, &paint);
    }
}

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
        let (cx, cy) = (
            area.center().x + self.camera.pan.0 as f32 * scale,
            area.center().y + self.camera.pan.1 as f32 * scale,
        );
        let unit = f64::from(area.width().min(area.height())) * 0.32 * self.camera.zoom;
        let screen =
            |[x, y, _]: [f64; 3]| Point::new(cx + (x * unit) as f32, cy - (y * unit) as f32);

        canvas.save();
        canvas.clip_rect(
            freya_engine::prelude::Rect::new(
                area.min_x(),
                area.min_y(),
                area.max_x(),
                area.max_y(),
            ),
            None,
            Some(true),
        );
        let frame = paint(self.frame, PaintStyle::Stroke, CHART.grid_width * scale);
        let corners: Vec<Point> = (0..8)
            .map(|i| {
                let corner = [
                    if i & 1 == 0 { -1.0 } else { 1.0 },
                    if i & 4 == 0 { -1.0 } else { 1.0 },
                    if i & 2 == 0 { -1.0 } else { 1.0 },
                ];
                screen(project(corner, self.camera.pose))
            })
            .collect();
        for (a, b) in BOX_EDGES {
            canvas.draw_line(corners[a], corners[b], &frame);
        }

        match &self.plot {
            Plot3D::Mesh { .. } => self.mesh(canvas, screen),
            Plot3D::Points(points) => {
                let ys: Vec<f64> = points.iter().map(|p| p[1]).collect();
                let (min, span) = normalized(&ys);
                let projected: Vec<([f64; 3], f32)> = points
                    .iter()
                    .map(|p| (project(*p, self.camera.pose), ((p[1] - min) / span) as f32))
                    .collect();
                let depths: Vec<f64> = projected.iter().map(|p| p.0[2]).collect();
                for i in back_to_front(&depths) {
                    let (point, t) = projected[i];
                    let radius = CHART.point_size * scale / 2.0 * (1.0 + 0.2 * point[2] as f32);
                    canvas.draw_circle(
                        screen(point),
                        radius,
                        &paint(mix(self.low, self.high, t), PaintStyle::Fill, 0.0),
                    );
                }
            }
        }
        canvas.restore();
    }
}

#[derive(PartialEq)]
pub struct Chart3D {
    label: &'static str,
    plot: Plot3D,
    height: f32,
}

impl Chart3D {
    pub fn surface(label: &'static str, grid: Vec<Vec<f64>>) -> Self {
        let (vertices, triangles) = surface_triangles(&grid);
        Self::new(
            label,
            Plot3D::Mesh {
                vertices,
                triangles,
                closed: false,
            },
        )
    }

    pub fn mesh(label: &'static str, vertices: &[[f64; 3]], triangles: Vec<[u32; 3]>) -> Self {
        Self::new(
            label,
            Plot3D::Mesh {
                vertices: fit(vertices),
                triangles,
                closed: true,
            },
        )
    }

    pub fn points(label: &'static str, points: Vec<[f64; 3]>) -> Self {
        Self::new(label, Plot3D::Points(points))
    }

    fn new(label: &'static str, plot: Plot3D) -> Self {
        Self {
            label,
            plot,
            height: CHART.height * 1.5,
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

fn view_title(view: View) -> &'static str {
    match view {
        View::Free => text!("view-free"),
        View::Custom => text!("view-custom"),
        View::Front => text!("view-front"),
        View::Top => text!("view-top"),
        View::Side => text!("view-side"),
        View::Isometric => text!("view-isometric"),
    }
}

const AZIMUTH: (f64, f64) = (-180.0, 180.0);
const INCLINATION: (f64, f64) = (-90.0, 90.0);

fn zoom_range() -> (f64, f64) {
    (f64::from(CHART.zoom_min), f64::from(CHART.zoom_max))
}

fn coordinate(label: &'static str, value: f32, on_change: impl Fn(f32) + 'static) -> Element {
    let palette = Palette::current();
    rect()
        .width(Size::fill())
        .direction(Direction::Horizontal)
        .cross_align(Alignment::Center)
        .spacing(SPACE_2)
        .child(
            rect()
                .width(Size::px(CHART.y_axis_width * 2.0))
                .child(crate::style::text(label, CALLOUT, palette.secondary_label)),
        )
        .child(
            rect().width(Size::flex(1.0)).child(
                LevelSlider::new(label)
                    .value(value)
                    .on_change(move |fraction: f32| on_change(fraction)),
            ),
        )
        .into_element()
}

impl Component for Chart3D {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let start = Camera::at(Pose {
            azimuth: f64::from(CHART.azimuth),
            inclination: f64::from(CHART.inclination),
        });
        let mut free = use_state(|| start);
        let mut custom = use_state(|| start);
        let mut view = use_state(|| View::Free);
        let mut drag = use_state(|| None::<(f64, f64, Camera, bool)>);
        let mut shift = use_state(|| false);
        let current = *view.read();
        let camera = match current {
            View::Free => *free.read(),
            View::Custom => *custom.read(),
            fixed => Camera::at(fixed.pose().unwrap_or(start.pose)),
        };
        let selected = View::ALL.iter().position(|v| *v == current).unwrap_or(0);
        let views = View::ALL
            .into_iter()
            .fold(SegmentedControl::new(text!("view")), |control, v| {
                control.segment(view_title(v))
            })
            .selected(selected)
            .on_change(move |index| view.set(View::ALL[index]));
        let readout = text!(
            "camera-readout",
            azimuth = format!("{:.0}", camera.pose.azimuth),
            inclination = format!("{:.0}", camera.pose.inclination),
            zoom = format!("{:.1}", camera.zoom)
        );
        let degrees = f64::from(CHART.drag_degrees);
        let coordinates = (current == View::Custom).then(|| {
            let set = move |change: fn(Camera, f32) -> Camera| {
                move |fraction: f32| {
                    let mut custom = custom;
                    let next = change(*custom.peek(), fraction);
                    custom.set(next);
                }
            };
            rect()
                .width(Size::fill())
                .direction(Direction::Vertical)
                .spacing(SPACE_2)
                .child(coordinate(
                    text!("camera-azimuth"),
                    to_unit(camera.pose.azimuth, AZIMUTH),
                    set(|c, f| Camera {
                        pose: Pose {
                            azimuth: from_unit(f, AZIMUTH),
                            ..c.pose
                        },
                        ..c
                    }),
                ))
                .child(coordinate(
                    text!("camera-inclination"),
                    to_unit(camera.pose.inclination, INCLINATION),
                    set(|c, f| Camera {
                        pose: Pose {
                            inclination: from_unit(f, INCLINATION),
                            ..c.pose
                        },
                        ..c
                    }),
                ))
                .child(coordinate(
                    text!("camera-zoom"),
                    to_unit(camera.zoom, zoom_range()),
                    set(|c, f| Camera {
                        zoom: from_unit(f, zoom_range()),
                        ..c
                    }),
                ))
        });
        rect()
            .width(Size::fill())
            .direction(Direction::Vertical)
            .spacing(SPACE_2)
            .a11y_role(AccessibilityRole::Figure)
            .a11y_alt(self.label)
            .on_global_key_down(move |event: Event<KeyboardEventData>| {
                if event.key == Key::Named(NamedKey::Shift) {
                    shift.set_if_modified(true);
                }
            })
            .on_global_key_up(move |event: Event<KeyboardEventData>| {
                if event.key == Key::Named(NamedKey::Shift) {
                    shift.set_if_modified(false);
                }
            })
            .child(views)
            .maybe_child(coordinates)
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::px(self.height))
                    .on_mouse_down(move |event: Event<MouseEventData>| {
                        let pan = *shift.peek() || event.button == Some(MouseButton::Right);
                        let at = event.global_location;
                        if current != View::Custom {
                            free.set(camera);
                            view.set_if_modified(View::Free);
                        }
                        drag.set(Some((at.x, at.y, camera, pan)));
                        Cursor::set(CursorIcon::Grabbing);
                    })
                    .on_global_pointer_move(move |event: Event<PointerEventData>| {
                        let Some((x, y, from, pan)) = *drag.peek() else {
                            return;
                        };
                        let at = event.global_location();
                        let (dx, dy) = (at.x - x, at.y - y);
                        let next = if pan {
                            from.pan_by(dx, dy)
                        } else {
                            from.orbit(dx, dy, degrees)
                        };
                        if *view.peek() == View::Custom {
                            custom.set(next);
                        } else {
                            free.set(next);
                        }
                    })
                    .on_global_pointer_press(move |_| {
                        if drag.peek().is_some() {
                            drag.set(None);
                            Cursor::set(CursorIcon::default());
                        }
                    })
                    .on_wheel(move |event: Event<WheelEventData>| {
                        let next = camera.zoom_by(
                            event.delta_y,
                            f64::from(CHART.wheel_zoom),
                            zoom_range(),
                        );
                        if current == View::Custom {
                            custom.set(next);
                        } else {
                            free.set(next);
                            view.set_if_modified(View::Free);
                        }
                    })
                    .child(crate::glass::leaf(Space {
                        plot: self.plot.clone(),
                        camera,
                        low: series_color(0),
                        high: series_color(2),
                        frame: palette.separator,
                        layout: crate::glass::layout(
                            Size::fill(),
                            Size::fill(),
                            Position::default(),
                        ),
                    })),
            )
            .child(
                rect()
                    .width(Size::fill())
                    .direction(Direction::Horizontal)
                    .cross_align(Alignment::Center)
                    .spacing(SPACE_2)
                    .child(
                        rect()
                            .a11y_role(AccessibilityRole::Label)
                            .a11y_alt(readout.clone())
                            .child(crate::style::text(
                                readout,
                                CALLOUT,
                                palette.secondary_label,
                            )),
                    )
                    .maybe(current != View::Custom, |el| {
                        el.child(
                            Button::new(text!("camera-keep"))
                                .size(ControlSize::Small)
                                .on_press(move |_| {
                                    custom.set(camera);
                                    view.set(View::Custom);
                                }),
                        )
                    }),
            )
    }
}

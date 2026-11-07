use std::borrow::Cow;
use std::f64::consts::TAU;

use ct_design::{CHART, Rgba, SPACE_1};
use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{PaintStyle, PathBuilder};
use torin::prelude::{Alignment, Direction, Position, Size};

use super::plot::paint;
use super::scale::{monotone_samples, stream_bands};
use super::{Series, format_value, legend_entry, series_color};
use crate::style::Palette;

const SAMPLES_PER_STEP: usize = 8;
const GUIDE_RINGS: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Geometry {
    center: (f64, f64),
    inner: f64,
    outer: f64,
    width: f64,
    height: f64,
    polar: bool,
}

impl Geometry {
    fn new(width: f64, height: f64, polar: bool) -> Self {
        let outer = width.min(height) / 2.0;
        Self {
            center: (width / 2.0, height / 2.0),
            inner: outer * f64::from(CHART.stream_hole),
            outer,
            width,
            height,
            polar,
        }
    }

    fn place(&self, t: f64, fraction: f64) -> (f32, f32) {
        if self.polar {
            let angle = t * TAU - TAU / 4.0;
            let radius = self.inner + fraction * (self.outer - self.inner);
            (
                (self.center.0 + radius * angle.cos()) as f32,
                (self.center.1 + radius * angle.sin()) as f32,
            )
        } else {
            (
                (t * self.width) as f32,
                ((1.0 - fraction) * self.height) as f32,
            )
        }
    }

    fn time_at(&self, x: f64, y: f64) -> f64 {
        if self.polar {
            let angle = (y - self.center.1).atan2(x - self.center.0) + TAU / 4.0;
            angle.rem_euclid(TAU) / TAU
        } else {
            (x / self.width.max(1.0)).clamp(0.0, 1.0)
        }
    }
}

// The wiggle baseline is not periodic, so the clock keeps one step open at twelve instead of joining the ends.
fn time_span(count: usize, polar: bool) -> f64 {
    if polar {
        count.max(1) as f64
    } else {
        count.saturating_sub(1).max(1) as f64
    }
}

#[derive(PartialEq)]
struct Stream {
    bands: Vec<Vec<(f64, f64)>>,
    colors: Vec<Rgba>,
    polar: bool,
    hover: Option<usize>,
    guide: Rgba,
    layout: LayoutData,
}

impl ElementExt for Stream {
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
        let geometry = Geometry::new(
            f64::from(area.width()),
            f64::from(area.height()),
            self.polar,
        );
        let offset = (area.min_x(), area.min_y());
        let at = |(x, y): (f32, f32)| (x + offset.0, y + offset.1);
        let (min, max) = self
            .bands
            .iter()
            .flatten()
            .fold((f64::MAX, f64::MIN), |(lo, hi), (a, b)| {
                (lo.min(*a), hi.max(*b))
            });
        let span = (max - min).max(f64::EPSILON);
        let count = self.bands.first().map_or(0, Vec::len);
        if count < 2 {
            return;
        }
        let steps = time_span(count, self.polar);

        let guide = paint(self.guide, PaintStyle::Stroke, CHART.grid_width * scale);
        if self.polar {
            for ring in 0..=GUIDE_RINGS {
                let fraction = ring as f64 / GUIDE_RINGS as f64;
                let radius = (geometry.inner + fraction * (geometry.outer - geometry.inner)) as f32;
                canvas.draw_circle(
                    at((geometry.center.0 as f32, geometry.center.1 as f32)),
                    radius,
                    &guide,
                );
            }
        }

        for (band, color) in self.bands.iter().zip(&self.colors) {
            let edge = |pick: fn(&(f64, f64)) -> f64| {
                let points: Vec<(f64, f64)> = band
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (i as f64, pick(v)))
                    .collect();
                monotone_samples(&points, SAMPLES_PER_STEP)
            };
            let (upper, lower) = (edge(|v| v.1), edge(|v| v.0));
            let mut path = PathBuilder::new();
            for (i, (t, value)) in upper.iter().chain(lower.iter().rev()).enumerate() {
                let point = at(geometry.place(t / steps, (value - min) / span));
                if i == 0 {
                    path.move_to(point);
                } else {
                    path.line_to(point);
                }
            }
            path.close();
            let path = path.detach();
            canvas.draw_path(&path, &paint(*color, PaintStyle::Fill, 0.0));
            canvas.draw_path(
                &path,
                &paint(self.guide, PaintStyle::Stroke, CHART.grid_width * scale),
            );
        }

        if let Some(i) = self.hover.filter(|i| *i < count) {
            let t = i as f64 / steps;
            let ray = paint(self.guide, PaintStyle::Stroke, CHART.rule_width * scale);
            canvas.draw_line(at(geometry.place(t, 0.0)), at(geometry.place(t, 1.0)), &ray);
        }
    }
}

#[derive(PartialEq)]
pub struct StreamChart {
    label: &'static str,
    series: Vec<Series>,
    polar: bool,
    unit: &'static str,
    size: f32,
}

impl StreamChart {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            series: Vec::new(),
            polar: false,
            unit: "",
            size: CHART.height,
        }
    }

    pub fn series(mut self, series: Series) -> Self {
        self.series.push(series);
        self
    }

    pub fn polar(mut self, polar: bool) -> Self {
        self.polar = polar;
        self
    }

    pub fn unit(mut self, unit: &'static str) -> Self {
        self.unit = unit;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl Component for StreamChart {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let mut hover = use_state(|| None::<usize>);
        let mut area = use_state(|| (0.0f64, 0.0f64));
        let values: Vec<Vec<f64>> = self.series.iter().map(|s| s.values.clone()).collect();
        let (order, bands) = stream_bands(&values);
        let colors: Vec<Rgba> = order.iter().map(|i| series_color(*i)).collect();
        let count = bands.first().map_or(0, Vec::len);
        let polar = self.polar;
        let at = *hover.read();
        let legend = self.series.iter().enumerate().map(|(i, series)| {
            let value = at
                .or(count.checked_sub(1))
                .and_then(|j| series.values.get(j))
                .map(|v| format_value(*v, self.unit))
                .unwrap_or_default();
            legend_entry(
                series_color(i),
                format!("{} {value}", series.name),
                &palette,
            )
        });
        let (width, height) = if polar {
            (Size::px(self.size), Size::px(self.size))
        } else {
            (Size::fill(), Size::px(self.size))
        };
        rect()
            .width(Size::fill())
            .direction(Direction::Vertical)
            .spacing(SPACE_1)
            .a11y_role(AccessibilityRole::Figure)
            .a11y_alt(self.label)
            .child(
                rect()
                    .width(Size::fill())
                    .main_align(Alignment::Center)
                    .direction(Direction::Horizontal)
                    .child(
                        rect()
                            .width(width)
                            .height(height)
                            .on_sized(move |event: Event<SizedEventData>| {
                                let size = event.area.size;
                                area.set_if_modified((
                                    f64::from(size.width),
                                    f64::from(size.height),
                                ));
                            })
                            .on_pointer_move(move |event: Event<PointerEventData>| {
                                let (w, h) = *area.peek();
                                let point = event.element_location();
                                let t = Geometry::new(w, h, polar).time_at(point.x, point.y);
                                let index = (t * time_span(count, polar)).round() as usize;
                                hover.set_if_modified(Some(index.min(count.saturating_sub(1))));
                            })
                            .on_pointer_leave(move |_| hover.set_if_modified(None))
                            .child(crate::glass::leaf(Stream {
                                bands,
                                colors,
                                polar,
                                hover: at,
                                guide: palette.separator,
                                layout: crate::glass::layout(
                                    Size::fill(),
                                    Size::fill(),
                                    Position::default(),
                                ),
                            })),
                    ),
            )
            .child(
                rect()
                    .width(Size::fill())
                    .direction(Direction::Horizontal)
                    .spacing(CHART.legend_gap)
                    .children(legend),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_flow_clock_starts_at_twelve_and_runs_clockwise() {
        let clock = Geometry::new(200.0, 200.0, true);
        let (x, y) = clock.place(0.0, 1.0);
        assert!((x - 100.0).abs() < 1e-3 && y.abs() < 1e-3);
        let (x, _) = clock.place(0.25, 1.0);
        assert!((x - 200.0).abs() < 1e-3);
    }

    #[test]
    fn the_last_sample_stops_one_step_short_of_twelve() {
        assert_eq!(11.0 / time_span(12, true), 11.0 / 12.0);
        assert_eq!(time_span(12, false), 11.0);
    }

    #[test]
    fn pointer_angle_maps_back_to_the_same_time() {
        let clock = Geometry::new(200.0, 200.0, true);
        for t in [0.0, 0.2, 0.5, 0.85] {
            let (x, y) = clock.place(t, 0.5);
            assert!((clock.time_at(f64::from(x), f64::from(y)) - t).abs() < 1e-4);
        }
    }
}

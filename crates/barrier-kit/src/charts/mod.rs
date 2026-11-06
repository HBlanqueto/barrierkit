mod plot;
mod scale;
mod sector;
mod space;
mod stream;
mod surface;

use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Gaps, Position, Size};
use vow_design::{CAPTION, CHART, CHART_SERIES, SPACE_1};

use crate::style::{Palette, text};
use scale::nice_ticks;

pub use sector::SectorChart;
pub use stream::StreamChart;
pub use surface::{Chart3D, HeatMap};

#[derive(Clone, Debug, PartialEq)]
pub struct Series {
    pub name: String,
    pub values: Vec<f64>,
}

impl Series {
    pub fn new(name: impl Into<String>, values: Vec<f64>) -> Self {
        Self {
            name: name.into(),
            values,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkKind {
    Line,
    Area,
    Bar,
    Point,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Interpolation {
    #[default]
    Linear,
    Monotone,
    Step,
}

#[derive(PartialEq)]
pub struct Chart {
    label: &'static str,
    kind: MarkKind,
    series: Vec<Series>,
    stacked: bool,
    interpolation: Interpolation,
    y_domain: Option<(f64, f64)>,
    unit: &'static str,
    x_labels: Vec<String>,
    rules: Vec<f64>,
    height: f32,
}

impl Chart {
    pub fn new(label: &'static str, kind: MarkKind) -> Self {
        Self {
            label,
            kind,
            series: Vec::new(),
            stacked: false,
            interpolation: Interpolation::default(),
            y_domain: None,
            unit: "",
            x_labels: Vec::new(),
            rules: Vec::new(),
            height: CHART.height,
        }
    }

    pub fn series(mut self, series: Series) -> Self {
        self.series.push(series);
        self
    }

    pub fn stacked(mut self, stacked: bool) -> Self {
        self.stacked = stacked;
        self
    }

    pub fn interpolation(mut self, interpolation: Interpolation) -> Self {
        self.interpolation = interpolation;
        self
    }

    pub fn y_domain(mut self, min: f64, max: f64) -> Self {
        self.y_domain = Some((min, max));
        self
    }

    pub fn unit(mut self, unit: &'static str) -> Self {
        self.unit = unit;
        self
    }

    pub fn x_labels(mut self, labels: Vec<String>) -> Self {
        self.x_labels = labels;
        self
    }

    pub fn rule(mut self, value: f64) -> Self {
        self.rules.push(value);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    fn extent(&self) -> (f64, f64) {
        let values: Vec<f64> = if self.stacked {
            scale::stack(
                &self
                    .series
                    .iter()
                    .map(|s| s.values.clone())
                    .collect::<Vec<_>>(),
            )
            .last()
            .map(|band| band.iter().map(|(_, upper)| *upper).collect())
            .unwrap_or_default()
        } else {
            self.series.iter().flat_map(|s| s.values.clone()).collect()
        };
        let max = values.iter().copied().fold(0.0, f64::max);
        let min = values.iter().copied().fold(0.0, f64::min);
        (min, if max > min { max } else { min + 1.0 })
    }

    fn count(&self) -> usize {
        self.series
            .iter()
            .map(|s| s.values.len())
            .max()
            .unwrap_or(0)
    }
}

pub fn format_value(value: f64, unit: &str) -> String {
    let digits = if value.fract() == 0.0 || value.abs() >= 100.0 {
        0
    } else if value.abs() >= 10.0 {
        1
    } else {
        2
    };
    format!("{value:.digits$}{unit}")
}

pub(crate) fn series_color(index: usize) -> vow_design::Rgba {
    CHART_SERIES[index % CHART_SERIES.len()]
}

impl Component for Chart {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let mut hover = use_state(|| None::<usize>);
        let mut width = use_state(|| 0.0f32);
        let (lo, hi) = self.y_domain.unwrap_or_else(|| self.extent());
        let ticks: Vec<f64> = nice_ticks(lo, hi, CHART.ticks as usize)
            .into_iter()
            .filter(|tick| self.y_domain.is_none_or(|(a, b)| (a..=b).contains(tick)))
            .collect();
        let domain = self.y_domain.unwrap_or((
            ticks.first().copied().unwrap_or(lo),
            ticks.last().copied().unwrap_or(hi),
        ));
        let count = self.count();
        let kind = self.kind;
        let index_at = move |x: f64, width: f32| -> usize {
            let fraction = (x / f64::from(width.max(1.0))).clamp(0.0, 1.0);
            match kind {
                MarkKind::Bar => ((fraction * count as f64) as usize).min(count.saturating_sub(1)),
                _ => (fraction * count.saturating_sub(1) as f64).round() as usize,
            }
        };

        let colors: Vec<_> = (0..self.series.len()).map(series_color).collect();
        let plot = plot::Plot {
            kind,
            series: self.series.clone(),
            colors: colors.clone(),
            stacked: self.stacked,
            interpolation: self.interpolation,
            y_domain: domain,
            ticks: ticks.clone(),
            rules: self
                .rules
                .iter()
                .map(|v| (*v, palette.secondary_label))
                .collect(),
            grid: palette.separator,
            hover: *hover.read(),
            hover_ink: palette.tertiary_label,
            halo: palette.content_background,
            layout: crate::glass::layout(Size::fill(), Size::fill(), Position::default()),
        };

        let y = scale::Linear {
            domain,
            range: (f64::from(self.height), 0.0),
        };
        let y_labels = ticks.iter().map(|tick| {
            let line = CAPTION.line_height as f32;
            let top = (y.map(*tick) as f32 - line / 2.0).clamp(0.0, self.height - line);
            rect()
                .position(Position::new_absolute().top(top).right(CHART.axis_gap))
                .child(text(
                    format_value(*tick, self.unit),
                    CAPTION,
                    palette.secondary_label,
                ))
                .into_element()
        });

        let at = *hover.read();
        let readout = self.series.iter().enumerate().map(|(s, series)| {
            let value = at
                .or(series.values.len().checked_sub(1))
                .and_then(|i| series.values.get(i))
                .map(|v| format_value(*v, self.unit))
                .unwrap_or_default();
            legend_entry(colors[s], format!("{} {value}", series.name), &palette)
        });

        let plot_width = *width.read();
        let x_labels = self.x_labels.iter().enumerate().filter_map(|(i, label)| {
            let step = (self.x_labels.len() / 6).max(1);
            if i % step != 0 || plot_width <= 0.0 {
                return None;
            }
            let center = match kind {
                MarkKind::Bar => plot_width * (i as f32 + 0.5) / count.max(1) as f32,
                _ => plot_width * i as f32 / count.saturating_sub(1).max(1) as f32,
            };
            Some(
                rect()
                    .position(
                        Position::new_absolute()
                            .left(center - CHART.y_axis_width / 2.0)
                            .top(0.0),
                    )
                    .width(Size::px(CHART.y_axis_width))
                    .main_align(Alignment::Center)
                    .direction(Direction::Horizontal)
                    .child(text(label.clone(), CAPTION, palette.secondary_label))
                    .into_element(),
            )
        });

        rect()
            .width(Size::fill())
            .direction(Direction::Vertical)
            .spacing(SPACE_1)
            .a11y_role(AccessibilityRole::Figure)
            .a11y_alt(self.label)
            .child(
                rect()
                    .width(Size::fill())
                    .direction(Direction::Horizontal)
                    .spacing(CHART.legend_gap)
                    .children(readout),
            )
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::px(self.height))
                    .direction(Direction::Horizontal)
                    .child(
                        rect()
                            .width(Size::px(CHART.y_axis_width))
                            .height(Size::fill())
                            .children(y_labels),
                    )
                    .child(
                        rect()
                            .width(Size::flex(1.0))
                            .height(Size::fill())
                            .on_sized(move |event: Event<SizedEventData>| {
                                width.set_if_modified(event.area.size.width)
                            })
                            .on_pointer_move(move |event: Event<PointerEventData>| {
                                let index = index_at(event.element_location().x, *width.peek());
                                hover.set_if_modified(Some(index));
                            })
                            .on_pointer_leave(move |_| hover.set_if_modified(None))
                            .child(crate::glass::leaf(plot)),
                    ),
            )
            .maybe(!self.x_labels.is_empty(), |el| {
                el.child(
                    rect()
                        .width(Size::fill())
                        .height(Size::px(CAPTION.line_height as f32))
                        .padding(Gaps::new(0.0, 0.0, 0.0, CHART.y_axis_width))
                        .children(x_labels),
                )
            })
    }
}

pub(crate) fn legend_entry(color: vow_design::Rgba, label: String, palette: &Palette) -> Element {
    rect()
        .direction(Direction::Horizontal)
        .cross_align(Alignment::Center)
        .spacing(SPACE_1)
        .child(
            rect()
                .width(Size::px(CHART.legend_swatch))
                .height(Size::px(CHART.legend_swatch))
                .corner_radius(CHART.legend_swatch / 2.0)
                .background(crate::theme::color(color)),
        )
        .child(text(label, CAPTION, palette.label))
        .into_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_chart_spans_zero_to_one() {
        let chart = Chart::new("vacío", MarkKind::Line).series(Series::new("a", vec![0.0, 0.0]));
        assert_eq!(chart.extent(), (0.0, 1.0));
    }

    #[test]
    fn values_keep_two_digits_below_ten_one_below_a_hundred_none_above() {
        assert_eq!(format_value(4.256, "%"), "4.26%");
        assert_eq!(format_value(42.25, ""), "42.2");
        assert_eq!(format_value(512.7, " MB"), "513 MB");
        assert_eq!(format_value(20.0, ""), "20");
    }
}

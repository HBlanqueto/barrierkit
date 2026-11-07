mod chart3d;
mod heat;
mod plot;
mod scale;
mod sector;
mod space;
mod stream;

use ctdesign::{CAPTION, CHART, CHART_SERIES, DURATION_BASE, HAIRLINE, SPACE_1, TOOLTIP};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};

use crate::i18n::text as tr_kit;
use crate::style::{Palette, text};
use crate::theme::{SurfaceKind, color, popover_palette};
use scale::{Window, format_age, nice_ticks, time_ticks};

pub use chart3d::Chart3D;
pub use heat::HeatMap;
pub use sector::SectorChart;
pub use stream::StreamChart;

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
    rules: Vec<(f64, String)>,
    height: f32,
    time_step: Option<f64>,
    scrollable: bool,
    horizontal: bool,
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
            time_step: None,
            scrollable: false,
            horizontal: false,
        }
    }

    pub fn time_axis(mut self, seconds_per_sample: f64) -> Self {
        self.time_step = Some(seconds_per_sample);
        self
    }

    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }

    pub fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
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

    pub fn rule(mut self, value: f64, label: impl Into<String>) -> Self {
        self.rules.push((value, label.into()));
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

    fn category(&self, index: usize) -> String {
        match self.time_step {
            Some(step) => format_age(
                (self.count().saturating_sub(1) - index) as f64 * step,
                tr_kit!("chart-now"),
            ),
            None => self.x_labels.get(index).cloned().unwrap_or_default(),
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

pub fn lerp_series(from: &[Series], to: &[Series], t: f64) -> Vec<Series> {
    to.iter()
        .enumerate()
        .map(|(s, target)| Series {
            name: target.name.clone(),
            values: target
                .values
                .iter()
                .enumerate()
                .map(|(i, end)| {
                    let start = from
                        .get(s)
                        .and_then(|f| f.values.get(i))
                        .copied()
                        .unwrap_or(0.0);
                    start + (end - start) * t
                })
                .collect(),
        })
        .collect()
}

pub(crate) fn series_color(index: usize) -> ctdesign::Rgba {
    CHART_SERIES[index % CHART_SERIES.len()]
}

fn axis_label(label: String, palette: &Palette) -> Element {
    text(label, CAPTION, palette.secondary_label).into_element()
}

fn placed(position: Position, width: f32, label: Element) -> Element {
    rect()
        .position(position)
        .width(Size::px(width))
        .main_align(Alignment::Center)
        .direction(Direction::Horizontal)
        .child(label)
        .into_element()
}

impl Component for Chart {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let mut hover = use_state(|| None::<usize>);
        let mut selected = use_state(|| None::<usize>);
        let mut pressed = use_state(|| None::<(usize, Window)>);
        let mut size = use_state(|| (0.0f32, 0.0f32));
        let mut window = use_state(|| None::<Window>);
        let mut drag = use_state(|| None::<(f64, Window)>);
        let mut shown = use_state(|| self.series.clone());
        let mut previous = use_state(|| self.series.clone());
        let mut flip = use_state(|| false);
        use_side_effect_with_deps(&self.series, move |series| {
            if *shown.peek() != *series {
                previous.set(shown.peek().clone());
                shown.set(series.clone());
                flip.toggle();
            }
        });
        let switch = crate::animation::use_switch_progress(flip, DURATION_BASE);
        let progress = f64::from(if *flip.read() { switch } else { 1.0 - switch });
        let (target, before) = (shown.read().clone(), previous.read().clone());
        let slid = self.time_step.is_some()
            && progress < 1.0
            && before.len() == target.len()
            && before
                .iter()
                .zip(&target)
                .all(|(a, b)| a.values.len() == b.values.len())
            && before != target;
        let (series, shift) = if slid {
            (target.clone(), 1.0 - progress)
        } else if self.time_step.is_some() {
            (target.clone(), 0.0)
        } else {
            (lerp_series(&before, &target, progress), 0.0)
        };

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
        let horizontal = self.horizontal && kind == MarkKind::Bar;
        let total = match kind {
            MarkKind::Bar => count as f64,
            _ => count.saturating_sub(1) as f64,
        };
        let view = window
            .read()
            .map_or(Window::full(total), |w| w.pan(0.0, total));
        let index_at = move |(x, y): (f64, f64), (w, h): (f32, f32)| -> usize {
            let last = count.saturating_sub(1);
            if horizontal {
                return ((y / f64::from(h.max(1.0)) * count as f64) as usize).min(last);
            }
            let at = view.position((x / f64::from(w.max(1.0))).clamp(0.0, 1.0));
            match kind {
                MarkKind::Bar => (at.max(0.0) as usize).min(last),
                _ => (at.round().max(0.0) as usize).min(last),
            }
        };

        let colors: Vec<_> = (0..self.series.len()).map(series_color).collect();
        let plot = plot::Plot {
            kind,
            series,
            colors: colors.clone(),
            stacked: self.stacked,
            interpolation: self.interpolation,
            y_domain: domain,
            ticks: ticks.clone(),
            rules: self
                .rules
                .iter()
                .map(|(v, _)| (*v, palette.secondary_label))
                .collect(),
            grid: palette.separator,
            hover: hover.read().or(*selected.read()),
            hover_ink: palette.tertiary_label,
            halo: palette.content_background,
            window: view,
            shift,
            horizontal,
            layout: crate::glass::layout(Size::fill(), Size::fill(), Position::default()),
        };

        let at = hover.read().or(*selected.read());
        let selection = selected.read().filter(|i| *i < count);
        let readout = target.iter().enumerate().map(|(s, series)| {
            let value = at
                .or(series.values.len().checked_sub(1))
                .and_then(|i| series.values.get(i))
                .map(|v| format_value(*v, self.unit))
                .unwrap_or_default();
            legend_entry(colors[s], format!("{} {value}", series.name), &palette)
        });

        let (plot_width, plot_height) = *size.read();
        let line = CAPTION.line_height as f32;
        let value_fraction =
            |tick: f64| ((tick - domain.0) / (domain.1 - domain.0).max(f64::EPSILON)) as f32;
        let value_labels: Vec<Element> = ticks
            .iter()
            .map(|tick| {
                let label = axis_label(format_value(*tick, self.unit), &palette);
                if horizontal {
                    let center = value_fraction(*tick) * plot_width;
                    placed(
                        Position::new_absolute()
                            .left(center - CHART.y_axis_width / 2.0)
                            .top(0.0),
                        CHART.y_axis_width,
                        label,
                    )
                } else {
                    let top = ((1.0 - value_fraction(*tick)) * self.height - line / 2.0)
                        .clamp(0.0, self.height - line);
                    rect()
                        .position(Position::new_absolute().top(top).left(CHART.axis_gap))
                        .child(label)
                        .into_element()
                }
            })
            .collect();
        let category_marks: Vec<(f32, String)> = match self.time_step {
            Some(step) if !horizontal => {
                let newest = count.saturating_sub(1) as f64;
                let age = |position: f64| (newest - position) * step;
                time_ticks(age(view.start), CHART.ticks as usize + 1)
                    .into_iter()
                    .map(|seconds| {
                        (
                            view.fraction(newest - seconds / step) as f32,
                            format_age(seconds, tr_kit!("chart-now")),
                        )
                    })
                    .filter(|(fraction, _)| (0.0..=1.0).contains(fraction))
                    .collect()
            }
            _ => {
                let step = (self.x_labels.len() / 6).max(1);
                self.x_labels
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| horizontal || i % step == 0)
                    .map(|(i, label)| {
                        let fraction = if horizontal {
                            (i as f32 + 0.5) / count.max(1) as f32
                        } else {
                            view.fraction(plot::position(kind, i)) as f32
                        };
                        (fraction, label.clone())
                    })
                    .filter(|(fraction, _)| (0.0..=1.0).contains(fraction))
                    .collect()
            }
        };
        let category_labels: Vec<Element> = category_marks
            .into_iter()
            .filter(|_| plot_width > 0.0)
            .map(|(fraction, label)| {
                let label = axis_label(label, &palette);
                if horizontal {
                    let top = (fraction * plot_height - line / 2.0)
                        .clamp(0.0, (plot_height - line).max(0.0));
                    rect()
                        .position(Position::new_absolute().top(top).right(CHART.axis_gap))
                        .child(label)
                        .into_element()
                } else {
                    let center = fraction * plot_width;
                    placed(
                        Position::new_absolute()
                            .left(center - CHART.y_axis_width / 2.0)
                            .top(0.0),
                        CHART.y_axis_width,
                        label,
                    )
                }
            })
            .collect();

        let rule_labels: Vec<Element> = self
            .rules
            .iter()
            .filter(|_| plot_width > 0.0)
            .map(|(value, label)| {
                let label = axis_label(label.clone(), &palette);
                let position = if horizontal {
                    Position::new_absolute()
                        .top(0.0)
                        .left(value_fraction(*value) * plot_width + CHART.axis_gap)
                } else {
                    Position::new_absolute()
                        .left(CHART.axis_gap)
                        .top(((1.0 - value_fraction(*value)) * plot_height - line).max(0.0))
                };
                rect().position(position).child(label).into_element()
            })
            .collect();
        let annotation = selection.filter(|_| plot_width > 0.0).map(|i| {
            let menu = popover_palette(&crate::theme::resolved(), SurfaceKind::Toplevel);
            let values = target.iter().enumerate().filter_map(|(s, series)| {
                series.values.get(i).map(|v| {
                    legend_entry(
                        colors[s],
                        format!("{} {}", series.name, format_value(*v, self.unit)),
                        &palette,
                    )
                })
            });
            let position = if horizontal {
                Position::new_absolute()
                    .right(CHART.axis_gap)
                    .top(plot_height * i as f32 / count.max(1) as f32)
            } else {
                let fraction = view.fraction(plot::position(kind, i)) as f32;
                if fraction < 0.5 {
                    Position::new_absolute()
                        .top(0.0)
                        .left(fraction * plot_width + CHART.axis_gap)
                } else {
                    Position::new_absolute()
                        .top(0.0)
                        .right((1.0 - fraction) * plot_width + CHART.axis_gap)
                }
            };
            rect()
                .position(position)
                .direction(Direction::Vertical)
                .spacing(SPACE_1)
                .padding(Gaps::new_symmetric(TOOLTIP.padding_y, TOOLTIP.padding_x))
                .corner_radius(TOOLTIP.radius)
                .background(color(menu.bg))
                .border(
                    Border::new()
                        .width(HAIRLINE)
                        .alignment(BorderAlignment::Inner)
                        .fill(color(menu.separator_color)),
                )
                .a11y_role(AccessibilityRole::Tooltip)
                .child(text(self.category(i), CAPTION, menu.fg))
                .children(values)
                .into_element()
        });

        let scrollable = self.scrollable && !horizontal;
        let mut area = rect()
            .width(Size::flex(1.0))
            .height(Size::fill())
            .on_sized(move |event: Event<SizedEventData>| {
                let area = event.area.size;
                size.set_if_modified((area.width, area.height));
            })
            .on_pointer_move(move |event: Event<PointerEventData>| {
                let point = event.element_location();
                hover.set_if_modified(Some(index_at((point.x, point.y), *size.peek())));
            })
            .on_pointer_leave(move |_| hover.set_if_modified(None))
            .on_mouse_down(move |event: Event<MouseEventData>| {
                let point = event.element_location;
                pressed.set(Some((index_at((point.x, point.y), *size.peek()), view)));
                if scrollable {
                    drag.set(Some((event.global_location.x, view)));
                }
            })
            .on_mouse_up(move |event: Event<MouseEventData>| {
                let point = event.element_location;
                let index = index_at((point.x, point.y), *size.peek());
                if pressed.take() == Some((index, view)) {
                    let next = (*selected.peek() != Some(index)).then_some(index);
                    selected.set(next);
                }
            })
            .child(crate::glass::leaf(plot))
            .children(rule_labels)
            .maybe_child(annotation);
        if scrollable {
            area = area
                .on_wheel(move |event: Event<WheelEventData>| {
                    let width = f64::from(size.peek().0.max(1.0));
                    let next = if event.delta_x.abs() > event.delta_y.abs() {
                        view.pan(-event.delta_x / width * view.span, total)
                    } else {
                        let factor = (event.delta_y * f64::from(CHART.wheel_zoom)).exp();
                        let fraction = (event.element_location.x / width).clamp(0.0, 1.0);
                        view.zoom_at(fraction, factor, total, f64::from(CHART.scroll_min_span))
                    };
                    window.set(Some(next));
                })
                .on_global_pointer_move(move |event: Event<PointerEventData>| {
                    let Some((x, from)) = *drag.peek() else {
                        return;
                    };
                    let width = f64::from(size.peek().0.max(1.0));
                    let moved = event.global_location().x - x;
                    window.set(Some(from.pan(-moved / width * from.span, total)));
                })
                .on_global_pointer_press(move |_| drag.set_if_modified(None));
        }

        let side = if horizontal {
            CHART.y_axis_width * 2.0
        } else {
            CHART.y_axis_width
        };
        let (side_children, bottom_labels) = if horizontal {
            (category_labels, value_labels)
        } else {
            (value_labels, category_labels)
        };
        let side_labels = rect()
            .width(Size::px(side))
            .height(Size::fill())
            .children(side_children);
        let plot_row = rect()
            .width(Size::fill())
            .height(Size::px(self.height))
            .direction(Direction::Horizontal)
            .content(Content::flex());
        let plot_row = if horizontal {
            plot_row.child(side_labels).child(area)
        } else {
            plot_row.child(area).child(side_labels)
        };
        let padding = if horizontal {
            Gaps::new(0.0, 0.0, 0.0, side)
        } else {
            Gaps::new(0.0, side, 0.0, 0.0)
        };

        rect()
            .width(Size::fill())
            .direction(Direction::Vertical)
            .spacing(SPACE_1)
            .a11y_role(AccessibilityRole::Figure)
            .a11y_alt(self.label)
            .child(plot_row)
            .maybe(!bottom_labels.is_empty(), |el| {
                el.child(
                    rect()
                        .width(Size::fill())
                        .height(Size::px(line))
                        .padding(padding)
                        .children(bottom_labels),
                )
            })
            .child(
                rect()
                    .width(Size::fill())
                    .direction(Direction::Horizontal)
                    .spacing(CHART.legend_gap)
                    .children(readout),
            )
    }
}

pub(crate) fn legend_entry(color: ctdesign::Rgba, label: String, palette: &Palette) -> Element {
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
    fn data_changes_blend_from_the_old_values_and_new_points_grow_from_zero() {
        let old = [Series::new("a", vec![10.0])];
        let new = [Series::new("a", vec![20.0, 8.0])];
        assert_eq!(lerp_series(&old, &new, 0.5)[0].values, vec![15.0, 4.0]);
    }

    #[test]
    fn values_keep_two_digits_below_ten_one_below_a_hundred_none_above() {
        assert_eq!(format_value(4.256, "%"), "4.26%");
        assert_eq!(format_value(42.25, ""), "42.2");
        assert_eq!(format_value(512.7, " MB"), "513 MB");
        assert_eq!(format_value(20.0, ""), "20");
    }
}

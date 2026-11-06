use freya_components::cache::AssetAge;
use freya_components::svg_viewer::SvgViewer;
use freya_core::prelude::*;
use torin::prelude::{Direction, Position, Size};
use vow_design::{CAPTION, GAUGE, HAIRLINE, LEVEL_INDICATOR, LEVEL_INDICATOR_COLORS};

use crate::style::{Palette, text};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

#[derive(PartialEq)]
pub struct CapacityIndicator {
    label: &'static str,
    value: f32,
    warning: f32,
    critical: f32,
    segments: Option<u32>,
}

impl CapacityIndicator {
    pub fn new(label: &'static str, value: f32) -> Self {
        Self {
            label,
            value,
            warning: 1.0,
            critical: 1.0,
            segments: None,
        }
    }

    pub fn discrete(mut self, segments: u32) -> Self {
        self.segments = Some(segments.max(1));
        self
    }

    pub fn thresholds(mut self, warning: f32, critical: f32) -> Self {
        self.warning = warning;
        self.critical = critical;
        self
    }
}

pub fn level_color(value: f32, warning: f32, critical: f32) -> usize {
    if value >= critical {
        2
    } else if value >= warning {
        1
    } else {
        0
    }
}

impl Component for CapacityIndicator {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let value = self.value.clamp(0.0, 1.0);
        let fill = LEVEL_INDICATOR_COLORS[level_color(value, self.warning, self.critical)];
        let cell = |filled: f32| {
            rect()
                .height(Size::fill())
                .corner_radius(LEVEL_INDICATOR.radius)
                .overflow(Overflow::Clip)
                .background(color(palette.control_background))
                .border(
                    Border::new()
                        .width(HAIRLINE)
                        .alignment(BorderAlignment::Inner)
                        .fill(color(palette.card_border)),
                )
                .child(
                    rect()
                        .width(Size::percent(filled * 100.0))
                        .height(Size::fill())
                        .background(color(fill)),
                )
        };
        let bar = rect()
            .a11y_role(AccessibilityRole::Meter)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_numeric_value(f64::from(value)))
            .width(Size::fill())
            .height(Size::px(LEVEL_INDICATOR.height));
        match self.segments {
            None => bar.child(cell(value).width(Size::fill())),
            Some(segments) => bar
                .direction(Direction::Horizontal)
                .content(torin::prelude::Content::flex())
                .spacing(LEVEL_INDICATOR.segment_gap)
                .children((0..segments).map(|segment| {
                    let filled = (value * segments as f32 - segment as f32).clamp(0.0, 1.0);
                    cell(filled.round()).width(Size::flex(1.0)).into_element()
                })),
        }
    }
}

#[derive(PartialEq)]
pub struct RelevancyIndicator {
    label: &'static str,
    value: f32,
}

impl RelevancyIndicator {
    pub fn new(label: &'static str, value: f32) -> Self {
        Self {
            label,
            value: value.clamp(0.0, 1.0),
        }
    }
}

impl Component for RelevancyIndicator {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let value = self.value;
        let mut width = use_state(|| 0.0f32);
        let pitch = LEVEL_INDICATOR.relevancy_line + LEVEL_INDICATOR.relevancy_gap;
        let lines = ((*width.read() * value) / pitch).floor() as usize;
        rect()
            .a11y_role(AccessibilityRole::Meter)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_numeric_value(f64::from(value)))
            .width(Size::fill())
            .height(Size::px(LEVEL_INDICATOR.relevancy_height))
            .direction(Direction::Horizontal)
            .spacing(LEVEL_INDICATOR.relevancy_gap)
            .on_sized(move |event: Event<SizedEventData>| {
                width.set_if_modified(event.area.width());
            })
            .children((0..lines).map(|_| {
                rect()
                    .width(Size::px(LEVEL_INDICATOR.relevancy_line))
                    .height(Size::fill())
                    .background(color(palette.secondary_label))
                    .into_element()
            }))
    }
}

#[derive(PartialEq)]
pub struct CircularGauge {
    label: &'static str,
    value: f32,
    current: String,
    capacity: bool,
}

impl CircularGauge {
    pub fn new(label: &'static str, value: f32, current: impl Into<String>) -> Self {
        Self {
            label,
            value: value.clamp(0.0, 1.0),
            current: current.into(),
            capacity: false,
        }
    }

    pub fn capacity(mut self, capacity: bool) -> Self {
        self.capacity = capacity;
        self
    }
}

pub fn gauge_marker(value: f32) -> (f32, f32) {
    let radius = (GAUGE.size - GAUGE.stroke) / 2.0;
    let start = 90.0 + (360.0 - GAUGE.arc) / 2.0;
    let angle = (start + GAUGE.arc * value).to_radians();
    let center = GAUGE.size / 2.0;
    (center + radius * angle.cos(), center + radius * angle.sin())
}

impl Component for CircularGauge {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let size = GAUGE.size;
        let center = size / 2.0;
        let radius = (size - GAUGE.stroke) / 2.0;
        let circumference = std::f32::consts::TAU * radius;
        let (arc, start) = if self.capacity {
            (360.0, -90.0)
        } else {
            (GAUGE.arc, 90.0 + (360.0 - GAUGE.arc) / 2.0)
        };
        let arc_svg = |length: f32| {
            format!(
                r#"<svg viewBox="0 0 {size} {size}"><circle cx="{center}" cy="{center}" r="{radius}" fill="none" stroke="currentColor" stroke-width="{stroke}" stroke-linecap="round" stroke-dasharray="{length} {circumference}" transform="rotate({start} {center} {center})"/></svg>"#,
                stroke = GAUGE.stroke,
            )
        };
        let ring = arc_svg(circumference * arc / 360.0);
        let (x, y) = gauge_marker(self.value);
        let (marker, marker_ink) = if self.capacity {
            (arc_svg(circumference * self.value), palette.accent)
        } else {
            (
                format!(
                    r#"<svg viewBox="0 0 {size} {size}"><circle cx="{x}" cy="{y}" r="{dot}" fill="currentColor"/></svg>"#,
                    dot = GAUGE.marker / 2.0,
                ),
                palette.label,
            )
        };
        let layer = |svg: String, ink: vow_design::Rgba| {
            SvgViewer::new(Bytes::from(svg))
                .width(Size::px(size))
                .height(Size::px(size))
                .color(color(ink))
                .show_loader(false)
        };
        let value = self.value;
        rect()
            .a11y_role(AccessibilityRole::Meter)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_numeric_value(f64::from(value)))
            .width(Size::px(size))
            .height(Size::px(size))
            .child(layer(
                ring,
                if self.capacity {
                    palette.control_background
                } else {
                    palette.tertiary_label
                },
            ))
            .child(
                rect()
                    .position(Position::new_absolute())
                    .child(layer(marker, marker_ink).asset_age(AssetAge::zero())),
            )
            .child(
                rect()
                    .position(Position::new_absolute())
                    .width(Size::px(size))
                    .height(Size::px(size))
                    .center()
                    .child(text(self.current.clone(), CAPTION, palette.label)),
            )
    }
}

#[derive(PartialEq)]
pub struct RatingIndicator {
    label: &'static str,
    value: u32,
    on_change: Option<Callback<u32, ()>>,
}

impl RatingIndicator {
    pub fn new(label: &'static str, value: u32) -> Self {
        Self {
            label,
            value,
            on_change: None,
        }
    }

    pub fn on_change(mut self, callback: impl Into<Callback<u32, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }
}

impl Component for RatingIndicator {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let value = self.value.min(LEVEL_INDICATOR.stars);
        let stars = (1..=LEVEL_INDICATOR.stars).map(|star| {
            let (shape, ink) = if star <= value {
                (Symbol::StarFill, palette.secondary_label)
            } else {
                (Symbol::Star, palette.tertiary_label)
            };
            let on_change = self.on_change.clone();
            rect()
                .maybe(on_change.is_some(), |el| {
                    el.on_press(move |_| {
                        if let Some(on_change) = &on_change {
                            on_change.call(star);
                        }
                    })
                })
                .child(symbol(shape, LEVEL_INDICATOR.star, ink))
                .into_element()
        });
        rect()
            .a11y_role(AccessibilityRole::Slider)
            .a11y_alt(self.label)
            .a11y_builder(move |node| {
                node.set_numeric_value(f64::from(value));
                node.set_max_numeric_value(f64::from(LEVEL_INDICATOR.stars));
            })
            .direction(Direction::Horizontal)
            .children(stars)
    }
}

#[cfg(test)]
mod tests {
    use super::level_color;

    #[test]
    fn the_gauge_marker_runs_from_the_lower_left_to_the_lower_right() {
        let (low_x, low_y) = super::gauge_marker(0.0);
        let (high_x, high_y) = super::gauge_marker(1.0);
        let center = vow_design::GAUGE.size / 2.0;
        assert!(low_x < center && low_y > center);
        assert!(high_x > center && high_y > center);
        assert!((low_y - high_y).abs() < 0.01);
    }

    #[test]
    fn the_capacity_turns_yellow_at_the_warning_and_red_at_the_critical_value() {
        assert_eq!(level_color(0.5, 0.75, 0.9), 0);
        assert_eq!(level_color(0.75, 0.75, 0.9), 1);
        assert_eq!(level_color(0.95, 0.75, 0.9), 2);
    }
}

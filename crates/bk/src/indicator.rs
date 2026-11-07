use ctdesign::LEVEL_INDICATOR;
use freya_components::cache::AssetAge;
use freya_components::svg_viewer::SvgViewer;
use freya_core::prelude::*;
use torin::prelude::{Direction, Position, Size};

use crate::controls::{dimmed, inner_border, shadowed};
use crate::interaction::{control_state, use_interaction};
use crate::style::with_font;
use crate::styles::{
    CapacityIndicatorStyle, ControlState, GaugeStyle, LayoutDirection, Paints,
    RatingIndicatorStyle, RelevancyIndicatorStyle, ShapeStyle, StateValue, Theme, resolve,
};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

#[derive(PartialEq)]
pub struct CapacityIndicator {
    label: &'static str,
    value: f32,
    warning: f32,
    critical: f32,
    segments: Option<u32>,
    style: Theme,
}

impl CapacityIndicator {
    pub fn new(label: &'static str, value: f32) -> Self {
        Self {
            label,
            value,
            warning: 1.0,
            critical: 1.0,
            segments: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(CapacityIndicatorStyle) -> CapacityIndicatorStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
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
        let style: CapacityIndicatorStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default(), style.effect);
        let value = self.value.clamp(0.0, 1.0);
        let fill = [&style.fill, &style.warning, &style.critical]
            [level_color(value, self.warning, self.critical)];
        let height = style.height;
        let radius = style
            .shape
            .corner_radius(height, height, LayoutDirection::LeftToRight);
        let cell = |filled: f32| {
            shadowed(
                rect()
                    .height(Size::fill())
                    .corner_radius(radius)
                    .overflow(Overflow::Clip)
                    .background(paints.fill(&style.track))
                    .map(paints.blur(&style.track), Rect::blur)
                    .map(style.border.as_ref(), |el, border| {
                        el.border(inner_border(&paints, border))
                    })
                    .child(
                        rect()
                            .width(Size::percent(filled * 100.0))
                            .height(Size::fill())
                            .background(paints.fill(fill))
                            .map(paints.blur(fill), Rect::blur),
                    ),
                &style.shadows,
            )
        };
        let bar = rect()
            .a11y_role(AccessibilityRole::Meter)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_numeric_value(f64::from(value)))
            .width(Size::fill())
            .height(Size::px(height));
        let bar = match self.segments {
            None => bar.child(cell(value).width(Size::fill())),
            Some(segments) => bar
                .direction(Direction::Horizontal)
                .content(torin::prelude::Content::flex())
                .spacing(style.segment_gap)
                .children((0..segments).map(|segment| {
                    let filled = (value * segments as f32 - segment as f32).clamp(0.0, 1.0);
                    cell(filled.round()).width(Size::flex(1.0)).into_element()
                })),
        };
        dimmed(bar, style.opacity)
    }
}

#[derive(PartialEq)]
pub struct RelevancyIndicator {
    label: &'static str,
    value: f32,
    style: Theme,
}

impl RelevancyIndicator {
    pub fn new(label: &'static str, value: f32) -> Self {
        Self {
            label,
            value: value.clamp(0.0, 1.0),
            style: Theme::tengen(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(RelevancyIndicatorStyle) -> RelevancyIndicatorStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }
}

impl Component for RelevancyIndicator {
    fn render(&self) -> impl IntoElement {
        let style: RelevancyIndicatorStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default(), style.effect);
        let value = self.value;
        let mut width = use_state(|| 0.0f32);
        let pitch = style.line_width + style.gap;
        let lines = ((*width.read() * value) / pitch).floor() as usize;
        let relevancy = rect()
            .a11y_role(AccessibilityRole::Meter)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_numeric_value(f64::from(value)))
            .width(Size::fill())
            .height(Size::px(style.height))
            .direction(Direction::Horizontal)
            .spacing(style.gap)
            .on_sized(move |event: Event<SizedEventData>| {
                width.set_if_modified(event.area.width());
            })
            .children((0..lines).map(|_| {
                rect()
                    .width(Size::px(style.line_width))
                    .height(Size::fill())
                    .background(paints.fill(&style.line))
                    .into_element()
            }));
        dimmed(relevancy, style.opacity)
    }
}

#[derive(PartialEq)]
pub struct Gauge {
    label: &'static str,
    value: f32,
    current: String,
    capacity: bool,
    style: Theme,
}

impl Gauge {
    pub fn new(label: &'static str, value: f32, current: impl Into<String>) -> Self {
        Self {
            label,
            value: value.clamp(0.0, 1.0),
            current: current.into(),
            capacity: false,
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(GaugeStyle) -> GaugeStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn capacity(mut self, capacity: bool) -> Self {
        self.capacity = capacity;
        self
    }
}

pub fn gauge_marker(style: &GaugeStyle, value: f32) -> (f32, f32) {
    let radius = (style.size - style.stroke) / 2.0;
    let start = 90.0 + (360.0 - style.arc) / 2.0;
    let angle = (start + style.arc * value).to_radians();
    let center = style.size / 2.0;
    (center + radius * angle.cos(), center + radius * angle.sin())
}

impl Component for Gauge {
    fn render(&self) -> impl IntoElement {
        let style: GaugeStyle = resolve(&self.style);
        let state = ControlState::default();
        let paints = Paints::current(state, style.effect);
        let size = style.size;
        let center = size / 2.0;
        let radius = (size - style.stroke) / 2.0;
        let circumference = std::f32::consts::TAU * radius;
        let (arc, start) = if self.capacity {
            (360.0, -90.0)
        } else {
            (style.arc, 90.0 + (360.0 - style.arc) / 2.0)
        };
        let arc_svg = |length: f32| {
            format!(
                r#"<svg viewBox="0 0 {size} {size}"><circle cx="{center}" cy="{center}" r="{radius}" fill="none" stroke="currentColor" stroke-width="{stroke}" stroke-linecap="round" stroke-dasharray="{length} {circumference}" transform="rotate({start} {center} {center})"/></svg>"#,
                stroke = style.stroke,
            )
        };
        let ring = arc_svg(circumference * arc / 360.0);
        let (x, y) = gauge_marker(&style, self.value);
        let (track, marker, marker_ink) = if self.capacity {
            (
                &style.capacity_track,
                arc_svg(circumference * self.value),
                &style.capacity_fill,
            )
        } else {
            (
                &style.track,
                format!(
                    r#"<svg viewBox="0 0 {size} {size}"><circle cx="{x}" cy="{y}" r="{dot}" fill="currentColor"/></svg>"#,
                    dot = style.marker_size / 2.0,
                ),
                &style.marker,
            )
        };
        let layer = |svg: String, paint: &StateValue<ShapeStyle>| {
            SvgViewer::new(Bytes::from(svg))
                .width(Size::px(size))
                .height(Size::px(size))
                .color(color(paints.color(paint.resolve(state))))
                .show_loader(false)
        };
        let value = self.value;
        let gauge = rect()
            .a11y_role(AccessibilityRole::Meter)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_numeric_value(f64::from(value)))
            .width(Size::px(size))
            .height(Size::px(size))
            .child(layer(ring, track))
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
                    .child(
                        with_font(label().text(self.current.clone()), style.font)
                            .max_lines(1)
                            .color(paints.fill(&style.label)),
                    ),
            );
        dimmed(gauge, style.opacity)
    }
}

#[derive(PartialEq)]
pub struct RatingIndicator {
    label: &'static str,
    value: u32,
    on_change: Option<Callback<u32, ()>>,
    style: Theme,
}

impl RatingIndicator {
    pub fn new(label: &'static str, value: u32) -> Self {
        Self {
            label,
            value,
            on_change: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(RatingIndicatorStyle) -> RatingIndicatorStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn on_change(mut self, callback: impl Into<Callback<u32, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }
}

impl Component for RatingIndicator {
    fn render(&self) -> impl IntoElement {
        let style: RatingIndicatorStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state, style.effect);
        let value = self.value.min(LEVEL_INDICATOR.stars);
        let stars = (1..=LEVEL_INDICATOR.stars).map(|star| {
            let (shape, ink) = if star <= value {
                (Symbol::StarFill, &style.star)
            } else {
                (Symbol::Star, &style.empty_star)
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
                .child(symbol(
                    shape,
                    style.star_size,
                    paints.color(ink.resolve(state)),
                ))
                .into_element()
        });
        let on_change = self.on_change.clone();
        let rating = interaction
            .attach(rect(), on_change.is_some())
            .a11y_role(AccessibilityRole::Slider)
            .a11y_alt(self.label)
            .a11y_builder(move |node| {
                node.set_numeric_value(f64::from(value));
                node.set_min_numeric_value(0.0);
                node.set_max_numeric_value(f64::from(LEVEL_INDICATOR.stars));
                node.set_numeric_value_step(1.0);
            })
            .map(on_change, |el, on_change| {
                el.on_key_down(move |event: Event<KeyboardEventData>| match event.key {
                    Key::Named(NamedKey::ArrowLeft | NamedKey::ArrowDown) => {
                        on_change.call(value.saturating_sub(1))
                    }
                    Key::Named(NamedKey::ArrowRight | NamedKey::ArrowUp) => {
                        on_change.call((value + 1).min(LEVEL_INDICATOR.stars))
                    }
                    _ => {}
                })
            })
            .direction(Direction::Horizontal)
            .children(stars);
        interaction.focus_ring(dimmed(rating, style.opacity), 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::level_color;
    use crate::styles::{ControlStyle, GaugeStyle};

    #[test]
    fn the_gauge_marker_runs_from_the_lower_left_to_the_lower_right() {
        let style = GaugeStyle::tengen();
        let (low_x, low_y) = super::gauge_marker(&style, 0.0);
        let (high_x, high_y) = super::gauge_marker(&style, 1.0);
        let center = style.size / 2.0;
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

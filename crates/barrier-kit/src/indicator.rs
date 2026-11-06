use freya_core::prelude::*;
use torin::prelude::{Direction, Size};
use vow_design::{HAIRLINE, LEVEL_INDICATOR, LEVEL_INDICATOR_COLORS};

use crate::style::Palette;
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

#[derive(PartialEq)]
pub struct CapacityIndicator {
    label: &'static str,
    value: f32,
    warning: f32,
    critical: f32,
}

impl CapacityIndicator {
    pub fn new(label: &'static str, value: f32) -> Self {
        Self {
            label,
            value,
            warning: 1.0,
            critical: 1.0,
        }
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
        rect()
            .a11y_role(AccessibilityRole::Meter)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_numeric_value(f64::from(value)))
            .width(Size::fill())
            .height(Size::px(LEVEL_INDICATOR.height))
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
                    .width(Size::percent(value * 100.0))
                    .height(Size::fill())
                    .background(color(fill)),
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
    fn the_capacity_turns_yellow_at_the_warning_and_red_at_the_critical_value() {
        assert_eq!(level_color(0.5, 0.75, 0.9), 0);
        assert_eq!(level_color(0.75, 0.75, 0.9), 1);
        assert_eq!(level_color(0.95, 0.75, 0.9), 2);
    }
}

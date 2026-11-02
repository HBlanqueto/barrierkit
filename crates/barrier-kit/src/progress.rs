//! Progress bar (determinate and indeterminate) and progress ring.

use freya_animation::prelude::{AnimNum, Ease, Function, OnCreation, OnFinish, use_animation};
use freya_components::svg_viewer::SvgViewer;
use freya_core::prelude::*;
use torin::prelude::{Direction, Position, Size};
use vow_design::PROGRESS;

use crate::theme::color;

#[derive(PartialEq)]
pub struct ProgressBar {
    label: &'static str,
    value: Option<f32>,
}

impl ProgressBar {
    pub fn determinate(label: &'static str, value: f32) -> Self {
        Self {
            label,
            value: Some(value.clamp(0.0, 1.0)),
        }
    }

    pub fn indeterminate(label: &'static str) -> Self {
        Self { label, value: None }
    }
}

impl Component for ProgressBar {
    fn render(&self) -> impl IntoElement {
        let resolved = crate::theme::resolved();
        let reduced_motion = resolved.reduced_motion;
        let sweep = use_animation(move |conf| {
            if !reduced_motion {
                conf.on_creation(OnCreation::Run);
                conf.on_finish(OnFinish::reverse());
            }
            AnimNum::new(0.0, 1.0 - PROGRESS.indeterminate_segment)
                .time(PROGRESS.indeterminate_ms)
                .ease(Ease::InOut)
                .function(Function::Cubic)
        });
        let (start, length) = match self.value {
            Some(value) => (0.0, value),
            None => (sweep.get().value(), PROGRESS.indeterminate_segment),
        };
        let value = self.value;

        rect()
            .a11y_role(AccessibilityRole::ProgressIndicator)
            .a11y_alt(self.label)
            .a11y_builder(move |node| {
                if let Some(value) = value {
                    node.set_numeric_value(f64::from(value));
                }
            })
            .width(Size::fill())
            .height(Size::px(PROGRESS.height))
            .corner_radius(PROGRESS.height / 2.0)
            .overflow(Overflow::Clip)
            .background(color(resolved.surfaces.control))
            .direction(Direction::Horizontal)
            .child(rect().width(Size::percent(start * 100.0)))
            .child(
                rect()
                    .width(Size::percent(length * 100.0))
                    .height(Size::fill())
                    .corner_radius(PROGRESS.height / 2.0)
                    .background(color(resolved.accent)),
            )
    }
}

#[derive(PartialEq)]
pub struct ProgressRing {
    label: &'static str,
    value: f32,
}

impl ProgressRing {
    pub fn new(label: &'static str, value: f32) -> Self {
        Self {
            label,
            value: value.clamp(0.0, 1.0),
        }
    }
}

impl Component for ProgressRing {
    fn render(&self) -> impl IntoElement {
        let resolved = crate::theme::resolved();
        let size = PROGRESS.ring_size;
        let stroke = PROGRESS.ring_stroke;
        let radius = (size - stroke) / 2.0;
        let circumference = std::f32::consts::TAU * radius;
        let filled = circumference * self.value;
        let circle = |dash: String| {
            format!(
                r#"<svg viewBox="0 0 {size} {size}"><circle cx="{c}" cy="{c}" r="{radius}" fill="none" stroke="currentColor" stroke-width="{stroke}" stroke-linecap="round" {dash} transform="rotate(-90 {c} {c})"/></svg>"#,
                c = size / 2.0,
            )
        };
        let layer = |svg: String, fill: vow_design::Rgba| {
            SvgViewer::new(Bytes::from(svg))
                .width(Size::px(size))
                .height(Size::px(size))
                .color(color(fill))
                .show_loader(false)
        };
        let value = self.value;

        rect()
            .a11y_role(AccessibilityRole::ProgressIndicator)
            .a11y_alt(self.label)
            .a11y_builder(move |node| node.set_numeric_value(f64::from(value)))
            .width(Size::px(size))
            .height(Size::px(size))
            .child(layer(circle(String::new()), resolved.surfaces.control))
            .child(rect().position(Position::new_absolute()).child(layer(
                circle(format!(r#"stroke-dasharray="{filled} {circumference}""#)),
                resolved.accent,
            )))
    }
}

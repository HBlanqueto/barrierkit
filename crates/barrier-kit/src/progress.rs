//! Progress bar (determinate and indeterminate), progress ring and spinner.

use freya_animation::prelude::{AnimNum, Ease, Function, OnCreation, OnFinish, use_animation};
use freya_components::svg_viewer::SvgViewer;
use freya_core::prelude::*;
use torin::prelude::{Direction, Position, Size};
use vow_design::{PROGRESS, SYMBOL};

use crate::backdrop::Tint;
use crate::style::Palette;
use crate::theme::color;

#[derive(PartialEq)]
pub struct ProgressBar {
    label: &'static str,
    value: Option<f32>,
    tint: Tint,
}

impl ProgressBar {
    pub fn determinate(label: &'static str, value: f32) -> Self {
        Self {
            label,
            value: Some(value.clamp(0.0, 1.0)),
            tint: Tint::Accent,
        }
    }

    pub fn indeterminate(label: &'static str) -> Self {
        Self {
            label,
            value: None,
            tint: Tint::Accent,
        }
    }

    pub fn tint(mut self, tint: Tint) -> Self {
        self.tint = tint;
        self
    }
}

impl Component for ProgressBar {
    fn render(&self) -> impl IntoElement {
        let resolved = crate::theme::resolved();
        let palette = Palette::current();
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
            .background(color(palette.control_background))
            .direction(Direction::Horizontal)
            .child(rect().width(Size::percent(start * 100.0)))
            .child(
                rect()
                    .width(Size::percent(length * 100.0))
                    .height(Size::fill())
                    .corner_radius(PROGRESS.height / 2.0)
                    .background(self.tint.fill(&palette)),
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
        let palette = Palette::current();
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
            .child(layer(circle(String::new()), palette.control_background))
            .child(rect().position(Position::new_absolute()).child(layer(
                circle(format!(r#"stroke-dasharray="{filled} {circumference}""#)),
                palette.accent,
            )))
    }
}

#[derive(PartialEq)]
pub struct Spinner {
    label: &'static str,
}

impl Spinner {
    pub fn new(label: &'static str) -> Self {
        Self { label }
    }
}

impl Component for Spinner {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let reduced_motion = crate::theme::resolved().reduced_motion;
        let spokes = PROGRESS.spinner_spokes;
        let turn = use_animation(move |conf| {
            if !reduced_motion {
                conf.on_creation(OnCreation::Run);
                conf.on_finish(OnFinish::restart());
            }
            AnimNum::new(0.0, spokes as f32)
                .time(PROGRESS.spinner_ms)
                .function(Function::Linear)
        });
        let step = turn.get().value().floor() as u32 % spokes;
        let degrees_per_spoke = 360.0 / spokes as f32;
        let svg = use_hook(|| spinner_svg(spokes));

        rect()
            .a11y_role(AccessibilityRole::ProgressIndicator)
            .a11y_alt(self.label)
            .width(Size::px(PROGRESS.spinner_size))
            .height(Size::px(PROGRESS.spinner_size))
            .rotation(step as f32 * degrees_per_spoke)
            .child(
                SvgViewer::new(svg)
                    .width(Size::px(PROGRESS.spinner_size))
                    .height(Size::px(PROGRESS.spinner_size))
                    .color(color(palette.secondary_label))
                    .show_loader(false),
            )
    }
}

fn spinner_svg(spokes: u32) -> Bytes {
    let lines: String = (0..spokes)
        .map(|spoke| {
            let angle = spoke as f32 * 360.0 / spokes as f32;
            let opacity = 1.0 - spoke as f32 / spokes as f32;
            format!(
                r#"<line x1="8" y1="1.5" x2="8" y2="4.5" stroke-opacity="{opacity}" transform="rotate(-{angle} 8 8)"/>"#
            )
        })
        .collect();
    Bytes::from(format!(
        r#"<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="{}" stroke-linecap="round">{lines}</svg>"#,
        SYMBOL.stroke
    ))
}

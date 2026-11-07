use freya_animation::prelude::{AnimNum, Function, OnChange, OnCreation, OnFinish, use_animation};
use freya_components::cache::AssetAge;
use freya_components::svg_viewer::SvgViewer;
use freya_core::prelude::*;
use torin::prelude::{Direction, Position, Size};

use crate::backdrop::Tint;
use crate::controls::{dimmed, shadowed};
use crate::styles::Painted;
use crate::styles::{
    Animation, ControlState, LayoutDirection, Paints, ProgressRingStyle, ProgressViewStyle,
    SpinnerStyle, StateValue, Theme, resolve,
};
use crate::theme::color;

#[derive(PartialEq)]
pub struct ProgressView {
    label: &'static str,
    value: Option<f32>,
    style: Theme,
}

impl ProgressView {
    pub fn determinate(label: &'static str, value: f32) -> Self {
        Self {
            label,
            value: Some(value.clamp(0.0, 1.0)),
            style: Theme::tengen(),
        }
    }

    pub fn indeterminate(label: &'static str) -> Self {
        Self {
            label,
            value: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(ProgressViewStyle) -> ProgressViewStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn tint(self, tint: Tint) -> Self {
        self.style(move |style| ProgressViewStyle {
            fill: StateValue::new(tint.into()),
            ..style
        })
    }
}

fn use_running(animation: Animation, restart: bool) -> f32 {
    let duration_ms = use_reactive(&animation.duration_ms());
    let elapsed = use_animation(move |conf| {
        let duration_ms = *duration_ms.read();
        if duration_ms > 0 {
            conf.on_creation(OnCreation::Run)
                .on_change(OnChange::Rerun)
                .on_finish(if restart {
                    OnFinish::restart()
                } else {
                    OnFinish::reverse()
                });
        }
        AnimNum::new(0.0, 1.0)
            .time(duration_ms)
            .function(Function::Linear)
    });
    if animation.duration_ms() == 0 {
        0.0
    } else {
        animation.progress(elapsed.get().value())
    }
}

impl Component for ProgressView {
    fn render(&self) -> impl IntoElement {
        let style: ProgressViewStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default(), style.effect);
        let sweep = use_running(style.animation, false);
        let (start, length) = match self.value {
            Some(value) => (0.0, value),
            None => (
                sweep * (1.0 - style.indeterminate_length),
                style.indeterminate_length,
            ),
        };
        let value = self.value;
        let height = style.height;
        let radius = style
            .shape
            .corner_radius(height, height, LayoutDirection::LeftToRight);

        let bar = rect()
            .a11y_role(AccessibilityRole::ProgressIndicator)
            .a11y_alt(self.label)
            .a11y_builder(move |node| {
                if let Some(value) = value {
                    node.set_numeric_value(f64::from(value));
                }
            })
            .width(Size::fill())
            .height(Size::px(height))
            .corner_radius(radius)
            .overflow(Overflow::Clip)
            .painted(&paints, &style.track)
            .bordered(&paints, style.border.as_ref())
            .direction(Direction::Horizontal)
            .child(rect().width(Size::percent(start * 100.0)))
            .child(
                rect()
                    .width(Size::percent(length * 100.0))
                    .height(Size::fill())
                    .corner_radius(radius)
                    .painted(&paints, &style.fill),
            );
        dimmed(shadowed(bar, &style.shadows), style.opacity)
    }
}

#[derive(PartialEq)]
pub struct ProgressRing {
    label: &'static str,
    value: f32,
    style: Theme,
}

impl ProgressRing {
    pub fn new(label: &'static str, value: f32) -> Self {
        Self {
            label,
            value: value.clamp(0.0, 1.0),
            style: Theme::tengen(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(ProgressRingStyle) -> ProgressRingStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }
}

impl Component for ProgressRing {
    fn render(&self) -> impl IntoElement {
        let style: ProgressRingStyle = resolve(&self.style);
        let state = ControlState::default();
        let paints = Paints::current(state, style.effect);
        let size = style.size;
        let stroke = style.stroke;
        let radius = (size - stroke) / 2.0;
        let circumference = std::f32::consts::TAU * radius;
        let filled = circumference * self.value;
        let circle = |dash: String| {
            format!(
                r#"<svg viewBox="0 0 {size} {size}"><circle cx="{c}" cy="{c}" r="{radius}" fill="none" stroke="currentColor" stroke-width="{stroke}" stroke-linecap="round" {dash} transform="rotate(-90 {c} {c})"/></svg>"#,
                c = size / 2.0,
            )
        };
        let layer = |svg: String, paint: &StateValue<_>| {
            SvgViewer::new(Bytes::from(svg))
                .width(Size::px(size))
                .height(Size::px(size))
                .color(color(paints.color(paint.resolve(state))))
                .show_loader(false)
        };
        let value = self.value;

        dimmed(
            rect()
                .a11y_role(AccessibilityRole::ProgressIndicator)
                .a11y_alt(self.label)
                .a11y_builder(move |node| node.set_numeric_value(f64::from(value)))
                .width(Size::px(size))
                .height(Size::px(size))
                .child(layer(circle(String::new()), &style.track))
                .child(
                    rect().position(Position::new_absolute()).child(
                        layer(
                            circle(format!(r#"stroke-dasharray="{filled} {circumference}""#)),
                            &style.fill,
                        )
                        .asset_age(AssetAge::zero()),
                    ),
                ),
            style.opacity,
        )
    }
}

#[derive(PartialEq)]
pub struct Spinner {
    label: &'static str,
    style: Theme,
}

impl Spinner {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(SpinnerStyle) -> SpinnerStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }
}

impl Component for Spinner {
    fn render(&self) -> impl IntoElement {
        let style: SpinnerStyle = resolve(&self.style);
        let state = ControlState::default();
        let paints = Paints::current(state, style.effect);
        let spokes = style.spokes;
        let turn = use_running(style.animation, true);
        let step = (turn * spokes as f32).floor() as u32 % spokes;
        let degrees_per_spoke = 360.0 / spokes as f32;
        let size = style.size;

        dimmed(
            rect()
                .a11y_role(AccessibilityRole::ProgressIndicator)
                .a11y_alt(self.label)
                .width(Size::px(size))
                .height(Size::px(size))
                .rotation(step as f32 * degrees_per_spoke)
                .child(
                    SvgViewer::new(spinner_svg(spokes, style.stroke))
                        .width(Size::px(size))
                        .height(Size::px(size))
                        .color(color(paints.color(style.spoke.resolve(state))))
                        .show_loader(false),
                ),
            style.opacity,
        )
    }
}

fn spinner_svg(spokes: u32, stroke: f32) -> Bytes {
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
        stroke
    ))
}

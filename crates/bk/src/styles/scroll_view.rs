use ctappearance::Resolved;
use ctdesign::{DURATION_FAST_MS, SCROLL_BAR};

use super::{
    Animation, ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, reduce_motion,
    system_effect,
};

/// States: only `normal`. The user's setting decides whether the bar is always
/// shown; `.scroll_bar(false)` hides it.
#[derive(Clone, Debug)]
pub struct ScrollViewStyle {
    /// Tengen, with `track_width`: `SCROLL_BAR.thumb` for the bar that shows while scrolling,
    /// `SCROLL_BAR.track` for the thumb and track of the bar that is always shown.
    pub thumb_width: f32,
    /// Tengen, with `thumb_width`: `SCROLL_BAR.thumb` for the bar that shows while scrolling,
    /// `SCROLL_BAR.track` for the thumb and track of the bar that is always shown.
    pub track_width: f32,
    /// Tengen, with `inset`: `SCROLL_BAR.min_length`, `SCROLL_BAR.inset` from the edge.
    pub min_thumb_length: f32,
    /// Tengen, with `min_thumb_length`: `SCROLL_BAR.min_length`, `SCROLL_BAR.inset` from the
    /// edge.
    pub inset: f32,
    /// Tengen: `SCROLL_BAR.edge_height`, the fade of `.edge_effect(true)`.
    pub edge_height: f32,
    /// Tengen: `Capsule`, the thumb and the track.
    pub thumb_shape: Shape,
    /// Tengen, with `track`: `Role(Label)`, `Role(Label)`: the color of the paint with the
    /// opacity below.
    pub thumb: StateValue<ShapeStyle>,
    /// Tengen, with `thumb`: `Role(Label)`, `Role(Label)`: the color of the paint with the
    /// opacity below.
    pub track: StateValue<ShapeStyle>,
    /// Tengen: `Role(WindowBackground)`, fading to clear.
    pub edge: StateValue<ShapeStyle>,
    /// Tengen: `Color` of the sidebar line, the line of `.top_line(true)`.
    pub top_line: StateValue<ShapeStyle>,
    /// Tengen: `SCROLL_BAR.opacity`.
    pub thumb_opacity: f32,
    /// Tengen: `SCROLL_BAR.always_opacity`.
    pub always_thumb_opacity: f32,
    /// Tengen: `SCROLL_BAR.track_opacity`.
    pub track_opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `SCROLL_BAR.hide_delay_ms`, from the last scroll until the bar fades out.
    pub hide_delay_ms: u64,
    /// Tengen: `Ease { duration_ms: DURATION_FAST_MS }`: the bar fades; `None` makes it appear
    /// and vanish at once.
    pub animation: Animation,
}

impl ControlStyle for ScrollViewStyle {
    fn tengen() -> Self {
        Self {
            thumb_width: SCROLL_BAR.thumb,
            track_width: SCROLL_BAR.track,
            min_thumb_length: SCROLL_BAR.min_length,
            inset: SCROLL_BAR.inset,
            edge_height: SCROLL_BAR.edge_height,
            thumb_shape: Shape::Capsule,
            thumb: StateValue::new(ShapeStyle::Role(Role::Label)),
            track: StateValue::new(ShapeStyle::Role(Role::Label)),
            edge: StateValue::new(ShapeStyle::Role(Role::WindowBackground)),
            top_line: StateValue::new(ShapeStyle::Color(crate::sidebar::sidebar_palette().line)),
            thumb_opacity: SCROLL_BAR.opacity,
            always_thumb_opacity: SCROLL_BAR.always_opacity,
            track_opacity: SCROLL_BAR.track_opacity,
            effect: system_effect,
            hide_delay_ms: SCROLL_BAR.hide_delay_ms,
            animation: Animation::Ease {
                duration_ms: DURATION_FAST_MS,
            },
        }
    }

    fn accessible(self, appearance: &Resolved) -> Self {
        reduce_motion(self, appearance, |style| Self {
            animation: Animation::None,
            ..style
        })
    }
}

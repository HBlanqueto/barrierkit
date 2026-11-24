use ctdesign::{DURATION_FAST_MS, SCROLL_BAR};

use crate::styles::{
    Animation, ControlStyle, Role, Shape, ShapeStyle, StateValue, appearance_index,
};

/// States: only `normal`. The user's setting decides whether the bar is always
/// shown; `.scroll_bar(false)` hides it.
#[derive(Clone, Debug)]
pub struct ScrollViewStyle {
    pub thumb_width: f32,
    pub track_width: f32,
    pub min_thumb_length: f32,
    pub inset: f32,
    /// Tengen: `SCROLL_BAR.edge_height`, the fade of `.edge_effect(true)`.
    pub edge_height: f32,
    /// Tengen: `Capsule`, the thumb and the track.
    pub thumb_shape: Shape,
    pub thumb: StateValue<ShapeStyle>,
    pub track: StateValue<ShapeStyle>,
    /// Tengen: `Role(WindowBackground)`, fading to clear.
    pub edge: StateValue<ShapeStyle>,
    /// Tengen: `Color` of the sidebar line, the line of `.top_line(true)`.
    pub top_line: StateValue<ShapeStyle>,
    /// Tengen: `SCROLL_BAR.opacity` of the appearance (light, dark, then both with Increase
    /// Contrast), over the track when the bar is always shown.
    pub thumb_opacity: f32,
    /// Tengen: `SCROLL_BAR.track_opacity` of the appearance.
    pub track_opacity: f32,
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
            track_width: SCROLL_BAR.track - 2.0 * SCROLL_BAR.inset,
            min_thumb_length: SCROLL_BAR.min_length,
            inset: SCROLL_BAR.inset,
            edge_height: SCROLL_BAR.edge_height,
            thumb_shape: Shape::Capsule,
            thumb: StateValue::new(ShapeStyle::Role(Role::Label)),
            track: StateValue::new(ShapeStyle::Role(Role::Label)),
            edge: StateValue::new(ShapeStyle::Role(Role::WindowBackground)),
            top_line: StateValue::new(ShapeStyle::Color(crate::sidebar::sidebar_palette().line)),
            thumb_opacity: SCROLL_BAR.opacity[appearance_index()],
            track_opacity: SCROLL_BAR.track_opacity[appearance_index()],
            hide_delay_ms: SCROLL_BAR.hide_delay_ms,
            animation: Animation::Ease {
                duration_ms: DURATION_FAST_MS,
            },
        }
    }
}

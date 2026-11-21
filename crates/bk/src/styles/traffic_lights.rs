use ctappearance::traffic::{self, ButtonVisual, TrafficInput};
use ctdesign::{Rgba, TITLEBAR, TRAFFIC_GLYPH_FADE_MS, TRAFFIC_RING_WIDTH, TrafficHalo};

use super::{Animation, Axis, ControlStyle, StateValue};

/// `looks` holds one `ButtonVisual` per light, in the order close, minimize, zoom.
///
/// States: hovered (the pointer is over the group), pressed (that light), focused
/// (the window is focused), disabled (zoom without zoom support, the other
/// lights with `.close_only(true)`). `.vertical(true)` is an instance override of
/// `axis`. `Titlebar`, the rail and the sidebar place their lights with
/// `TrafficLights::for_window`, so their size comes from the theme and the containers.
#[derive(Clone, Debug)]
pub struct TrafficLightsStyle {
    /// Tengen: `TITLEBAR.buttons.diameter`.
    pub diameter: f32,
    /// Tengen: `TITLEBAR.buttons.spacing`.
    pub spacing: f32,
    /// Tengen: `TRAFFIC_RING_WIDTH`.
    pub ring_width: f32,
    /// Tengen: the `ButtonVisual`s of `ctappearance::traffic::visual` for the user's
    /// `traffic_lights`: normal and disabled are the inactive light without a glyph, focused the
    /// lit light, hovered the lit light with its glyph, pressed the enlarged brightened light.
    pub looks: [StateValue<ButtonVisual>; 3],
    /// Tengen: `Resolved::traffic_lights.halo`.
    pub halo: TrafficHalo,
    /// Tengen: `Resolved::traffic_lights.backdrop`; what a pressed light brightens against.
    pub backdrop: Rgba,
    /// Tengen: `Ease` of `TRAFFIC_GLYPH_FADE_MS`; the cross-fade of the zoom glyph when the window enters or leaves full screen.
    pub animation: Animation,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `Horizontal`.
    pub axis: Axis,
}

impl TrafficLightsStyle {
    pub(crate) fn length(&self) -> f32 {
        let count = TITLEBAR.buttons.order.len() as f32;
        count * self.diameter + (count - 1.0) * self.spacing
    }
}

impl ControlStyle for TrafficLightsStyle {
    fn tengen() -> Self {
        let resolved = crate::theme::resolved();
        let looks = |focused, group_hovered, pressed| {
            traffic::visual(
                &resolved.traffic_lights,
                TrafficInput {
                    focused,
                    group_hovered,
                    pressed,
                    zoom_supported: true,
                    fullscreen: false,
                },
            )
        };
        let (idle, lit, hovered) = (
            looks(false, false, None),
            looks(true, false, None),
            looks(true, true, None),
        );
        Self {
            diameter: TITLEBAR.buttons.diameter as f32,
            spacing: TITLEBAR.buttons.spacing as f32,
            ring_width: TRAFFIC_RING_WIDTH,
            looks: std::array::from_fn(|index| {
                let kind = TITLEBAR.buttons.order[index].kind;
                let focused = if resolved.differentiate_without_color {
                    hovered[index]
                } else {
                    lit[index]
                };
                StateValue::new(idle[index])
                    .focused(focused)
                    .hovered(hovered[index])
                    .pressed(looks(true, true, Some(kind))[index])
                    .disabled(idle[index])
            }),
            halo: resolved.traffic_lights.halo,
            backdrop: resolved.traffic_lights.backdrop,
            animation: Animation::Ease {
                duration_ms: TRAFFIC_GLYPH_FADE_MS,
            },
            opacity: 1.0,
            axis: Axis::Horizontal,
        }
    }
}

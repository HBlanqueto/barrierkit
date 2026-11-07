use ctappearance::Resolved;
use ctdesign::{DURATION_FAST_MS, SCROLL_BAR};

use super::{
    Animation, ColorEffect, ControlStyle, Role, Shape, ShapeStyle, StateValue, system_effect,
};

#[derive(Clone, Debug)]
pub struct ScrollViewStyle {
    pub thumb_width: f32,
    pub track_width: f32,
    pub min_thumb_length: f32,
    pub inset: f32,
    pub edge_height: f32,
    pub thumb_shape: Shape,
    pub thumb: StateValue<ShapeStyle>,
    pub track: StateValue<ShapeStyle>,
    pub edge: StateValue<ShapeStyle>,
    pub top_line: StateValue<ShapeStyle>,
    pub thumb_opacity: f32,
    pub always_thumb_opacity: f32,
    pub track_opacity: f32,
    pub effect: ColorEffect,
    pub hide_delay_ms: u64,
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
        if appearance.reduced_motion {
            Self {
                animation: Animation::None,
                ..self
            }
        } else {
            self
        }
    }
}

use ctappearance::Resolved;
use ctdesign::{KNOB_SHADOW, SLIDER, SLIDER_FAT, SWITCH, SYMBOL};

use crate::symbol::Symbol;

use super::{
    Animation, ColorEffect, ControlStyle, LayoutDirection, Role, ShadowStyle, Shape, ShapeStyle,
    StateValue, reduce_motion, system_effect,
};

/// States: hovered, pressed, focused, disabled. `LevelSlider::tint(Tint)` is an
/// instance override of `fill`. The preset `LevelSliderStyle::fat(symbol)` is the
/// Control Center bar: `height` and `track_height` `SLIDER_FAT.height`,
/// `knob_size` `None`, `fill_min` `SLIDER_FAT.fill_min` and the symbol.
#[derive(Clone, Debug)]
pub struct LevelSliderStyle {
    /// Tengen: `SLIDER.height`; the slider fills its width.
    pub height: f32,
    /// Tengen: `SLIDER.track`.
    pub track_height: f32,
    /// Tengen: `Some((SLIDER.knob_width, SLIDER.knob_height))`; `None` draws no knob and the
    /// fill starts and ends at the edges.
    pub knob_size: Option<(f32, f32)>,
    /// Tengen: `0.0`: the shortest fill, so the symbol always sits on it.
    pub fill_min: f32,
    /// Tengen: `None`; a symbol drawn inside the track at the leading end.
    pub symbol: Option<Symbol>,
    /// Tengen: `SYMBOL.grid`.
    pub icon_size: f32,
    /// Tengen: `SLIDER_FAT.lead_icon_x`.
    pub icon_inset: f32,
    /// Tengen: `Capsule`, `Capsule`.
    pub track_shape: Shape,
    /// Tengen: the fill takes the track's shape.
    pub knob_shape: Shape,
    /// Tengen: `Role(ControlBackground)`.
    pub track: StateValue<ShapeStyle>,
    /// Tengen: `Accent`.
    pub fill: StateValue<ShapeStyle>,
    /// Tengen: `Color(SWITCH.knob_color)`.
    pub knob: StateValue<ShapeStyle>,
    /// Tengen: `Role(OnAccent)`.
    pub icon: StateValue<ShapeStyle>,
    /// Tengen: `None`.
    pub knob_border: Option<(ShapeStyle, f32)>,
    /// Tengen: `[KNOB_SHADOW]`.
    pub knob_shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`, for every part.
    pub opacity: f32,
    /// Tengen: `SWITCH.disabled_opacity`, for the track, fill and symbol.
    pub disabled_opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `LeftToRight`; `RightToLeft` fills from the right and swaps ← →.
    pub layout_direction: LayoutDirection,
    /// Tengen: `None`: the knob and the fill jump, as in Tengen.
    pub animation: Animation,
}

impl LevelSliderStyle {
    pub fn fat(self, symbol: Symbol) -> Self {
        Self {
            height: SLIDER_FAT.height,
            track_height: SLIDER_FAT.height,
            knob_size: None,
            fill_min: SLIDER_FAT.fill_min,
            symbol: Some(symbol),
            ..self
        }
    }
}

impl ControlStyle for LevelSliderStyle {
    fn tengen() -> Self {
        Self {
            height: SLIDER.height,
            track_height: SLIDER.track,
            knob_size: Some((SLIDER.knob_width, SLIDER.knob_height)),
            fill_min: 0.0,
            symbol: None,
            icon_size: SYMBOL.grid,
            icon_inset: SLIDER_FAT.lead_icon_x,
            track_shape: Shape::Capsule,
            knob_shape: Shape::Capsule,
            track: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            fill: StateValue::new(ShapeStyle::Accent),
            knob: StateValue::new(ShapeStyle::Color(SWITCH.knob_color)),
            icon: StateValue::new(ShapeStyle::Role(Role::OnAccent)),
            knob_border: None,
            knob_shadows: vec![KNOB_SHADOW.into()],
            opacity: 1.0,
            disabled_opacity: SWITCH.disabled_opacity,
            effect: system_effect,
            layout_direction: LayoutDirection::LeftToRight,
            animation: Animation::None,
        }
    }

    fn accessible(self, appearance: &Resolved) -> Self {
        reduce_motion(self, appearance, |style| Self {
            animation: Animation::None,
            ..style
        })
    }
}

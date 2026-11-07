use ctappearance::Resolved;
use ctdesign::{KNOB_SHADOW, SLIDER, SLIDER_FAT, SWITCH, SYMBOL};

use crate::symbol::Symbol;

use super::{
    Animation, ColorEffect, ControlStyle, LayoutDirection, Role, ShadowStyle, Shape, ShapeStyle,
    StateValue, reduce_motion, system_effect,
};

#[derive(Clone, Debug)]
pub struct LevelSliderStyle {
    pub height: f32,
    pub track_height: f32,
    pub knob_size: Option<(f32, f32)>,
    pub fill_min: f32,
    pub symbol: Option<Symbol>,
    pub icon_size: f32,
    pub icon_inset: f32,
    pub track_shape: Shape,
    pub knob_shape: Shape,
    pub track: StateValue<ShapeStyle>,
    pub fill: StateValue<ShapeStyle>,
    pub knob: StateValue<ShapeStyle>,
    pub icon: StateValue<ShapeStyle>,
    pub knob_border: Option<(ShapeStyle, f32)>,
    pub knob_shadows: Vec<ShadowStyle>,
    pub opacity: f32,
    pub disabled_opacity: f32,
    pub effect: ColorEffect,
    pub layout_direction: LayoutDirection,
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

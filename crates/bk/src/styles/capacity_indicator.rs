use ctdesign::{HAIRLINE, LEVEL_INDICATOR, LEVEL_INDICATOR_COLORS};

use super::button::rounded;
use super::{
    ColorEffect, ControlStyle, Role, ShadowStyle, Shape, ShapeStyle, StateValue, system_effect,
};

/// States: only `normal`.
#[derive(Clone, Debug)]
pub struct CapacityIndicatorStyle {
    /// Tengen, with `segment_gap`: `LEVEL_INDICATOR.height`, `LEVEL_INDICATOR.segment_gap`
    /// between the cells of `discrete`.
    pub height: f32,
    /// Tengen, with `height`: `LEVEL_INDICATOR.height`, `LEVEL_INDICATOR.segment_gap` between
    /// the cells of `discrete`.
    pub segment_gap: f32,
    /// Tengen: `RoundedRectangle { radius: LEVEL_INDICATOR.radius, style: Circular }`, the bar
    /// or each cell.
    pub shape: Shape,
    /// Tengen: `Role(ControlBackground)`.
    pub track: StateValue<ShapeStyle>,
    /// Tengen, with `warning`, `critical`: `Color` of `LEVEL_INDICATOR_COLORS[0]`, `[1]` and
    /// `[2]`, picked by `level_color`.
    pub fill: StateValue<ShapeStyle>,
    /// Tengen, with `fill`, `critical`: `Color` of `LEVEL_INDICATOR_COLORS[0]`, `[1]` and
    /// `[2]`, picked by `level_color`.
    pub warning: StateValue<ShapeStyle>,
    /// Tengen, with `fill`, `warning`: `Color` of `LEVEL_INDICATOR_COLORS[0]`, `[1]` and `[2]`,
    /// picked by `level_color`.
    pub critical: StateValue<ShapeStyle>,
    /// Tengen: `Some((Role(CardBorder), HAIRLINE))`.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: empty, for the bar or each cell.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
}

impl ControlStyle for CapacityIndicatorStyle {
    fn tengen() -> Self {
        let [fill, warning, critical] =
            LEVEL_INDICATOR_COLORS.map(|rgba| StateValue::new(ShapeStyle::Color(rgba)));
        Self {
            height: LEVEL_INDICATOR.height,
            segment_gap: LEVEL_INDICATOR.segment_gap,
            shape: rounded(LEVEL_INDICATOR.radius),
            track: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            fill,
            warning,
            critical,
            border: Some((ShapeStyle::Role(Role::CardBorder), HAIRLINE)),
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
        }
    }
}

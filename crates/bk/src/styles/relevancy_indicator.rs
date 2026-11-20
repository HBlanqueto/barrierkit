use ctdesign::{
    GAUGE_CONTRAST_DARK, GAUGE_CONTRAST_DARK_TINT, GAUGE_CONTRAST_LIGHT, LEVEL_INDICATOR,
};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, per_look, system_effect};

/// States: `RatingIndicator` is hovered, pressed and focused; the others only
/// `normal`. The ring and the marker of a `Gauge` take one color each, like the
/// progress ring. `gauge_marker(style, value)` places the marker for a
/// `GaugeStyle`.
#[derive(Clone, Debug)]
pub struct RelevancyIndicatorStyle {
    /// Tengen: `LEVEL_INDICATOR.relevancy_height`.
    pub height: f32,
    /// Tengen: `LEVEL_INDICATOR.relevancy_line`.
    pub line_width: f32,
    /// Tengen: `LEVEL_INDICATOR.relevancy_gap`.
    pub gap: f32,
    /// Tengen: `Role(SystemFill)`, `Color(GAUGE_CONTRAST.level_track)` under Increase Contrast,
    /// behind the whole width.
    pub track: StateValue<ShapeStyle>,
    /// Tengen: `Role(TertiaryLabel)`, behind the lines of the value.
    pub filled: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`.
    pub line: StateValue<ShapeStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
}

impl ControlStyle for RelevancyIndicatorStyle {
    fn tengen() -> Self {
        Self {
            height: LEVEL_INDICATOR.relevancy_height,
            line_width: LEVEL_INDICATOR.relevancy_line,
            gap: LEVEL_INDICATOR.relevancy_gap,
            track: StateValue::new(per_look(
                [
                    ShapeStyle::Role(Role::SystemFill),
                    ShapeStyle::Role(Role::SystemFill),
                    ShapeStyle::Role(Role::SystemFill),
                ],
                [
                    ShapeStyle::Color(GAUGE_CONTRAST_LIGHT.level_track),
                    ShapeStyle::Color(GAUGE_CONTRAST_DARK.level_track),
                    ShapeStyle::Color(GAUGE_CONTRAST_DARK_TINT.level_track),
                ],
            )),
            filled: StateValue::new(ShapeStyle::Role(Role::TertiaryLabel)),
            line: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            opacity: 1.0,
            effect: system_effect,
        }
    }
}

use ctdesign::{
    GAUGE_CONTRAST_DARK, GAUGE_CONTRAST_DARK_TINT, GAUGE_CONTRAST_LIGHT, LEVEL_INDICATOR,
    RELEVANCY_CONTRAST_DARK, RELEVANCY_CONTRAST_DARK_TINT, RELEVANCY_CONTRAST_LIGHT,
    RELEVANCY_DARK, RELEVANCY_DARK_TINT, RELEVANCY_LIGHT, RELEVANCY_TRACK_DARK,
    RELEVANCY_TRACK_DARK_TINT, RELEVANCY_TRACK_LIGHT,
};

use super::{ColorEffect, ControlStyle, ShapeStyle, StateValue, per_look, system_effect};

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
    /// Tengen: `Color(RELEVANCY_TRACK)`, `Color(GAUGE_CONTRAST.level_track)` under Increase
    /// Contrast, behind the whole width.
    pub track: StateValue<ShapeStyle>,
    /// Tengen: `Color(RELEVANCY.fill)`, `Color(RELEVANCY_CONTRAST.fill)` under Increase Contrast,
    /// behind the lines of the value.
    pub filled: StateValue<ShapeStyle>,
    /// Tengen: `Color(RELEVANCY.line)`, `Color(RELEVANCY_CONTRAST.line)` under Increase Contrast.
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
            track: StateValue::new(ShapeStyle::Color(per_look(
                [
                    RELEVANCY_TRACK_LIGHT,
                    RELEVANCY_TRACK_DARK,
                    RELEVANCY_TRACK_DARK_TINT,
                ],
                [
                    GAUGE_CONTRAST_LIGHT.level_track,
                    GAUGE_CONTRAST_DARK.level_track,
                    GAUGE_CONTRAST_DARK_TINT.level_track,
                ],
            ))),
            filled: StateValue::new(ShapeStyle::Color(per_look(
                [
                    RELEVANCY_LIGHT.fill,
                    RELEVANCY_DARK.fill,
                    RELEVANCY_DARK_TINT.fill,
                ],
                [
                    RELEVANCY_CONTRAST_LIGHT.fill,
                    RELEVANCY_CONTRAST_DARK.fill,
                    RELEVANCY_CONTRAST_DARK_TINT.fill,
                ],
            ))),
            line: StateValue::new(ShapeStyle::Color(per_look(
                [
                    RELEVANCY_LIGHT.line,
                    RELEVANCY_DARK.line,
                    RELEVANCY_DARK_TINT.line,
                ],
                [
                    RELEVANCY_CONTRAST_LIGHT.line,
                    RELEVANCY_CONTRAST_DARK.line,
                    RELEVANCY_CONTRAST_DARK_TINT.line,
                ],
            ))),
            opacity: 1.0,
            effect: system_effect,
        }
    }
}

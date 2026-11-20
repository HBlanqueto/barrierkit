use ctdesign::{
    GAUGE_CONTRAST_DARK, GAUGE_CONTRAST_DARK_TINT, GAUGE_CONTRAST_LIGHT, HAIRLINE,
    LEVEL_BORDER_DARK, LEVEL_BORDER_DARK_TINT, LEVEL_BORDER_LIGHT, LEVEL_INDICATOR,
    LEVEL_INDICATOR_COLORS, LEVEL_INDICATOR_COLORS_CONTRAST, LEVEL_INDICATOR_COLORS_CONTRAST_DARK,
    LEVEL_INDICATOR_COLORS_DARK, LEVEL_TRACK_DARK, LEVEL_TRACK_DARK_TINT, LEVEL_TRACK_LIGHT,
};

use super::{
    ColorEffect, ControlStyle, CornerStyle, ShadowStyle, Shape, ShapeStyle, StateValue, per_look,
    system_effect,
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
    /// Tengen: `RoundedRectangle { radius: LEVEL_INDICATOR.radius, style: Continuous }`, the bar
    /// or each cell.
    pub shape: Shape,
    /// Tengen: `Color` of `LEVEL_TRACK`.
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
    /// Tengen: `Some((Color of LEVEL_BORDER, HAIRLINE))`.
    pub border: Option<(ShapeStyle, f32)>,
    /// Tengen: `LEVEL_INDICATOR.discrete_border`, `discrete_border_contrast_light` in light under
    /// Increase Contrast: the factor by which the filled part of the bar, or a filled cell of
    /// `discrete`, darkens the color of its level for its border.
    pub cell_border_shade: f32,
    /// Tengen: `LEVEL_INDICATOR.edge_softness`: how far the border of the bar or of a cell blends
    /// into its fill, in three rings of half a hairline: the border, then the border mixed this
    /// much with the fill, then the fill mixed this much with the border.
    pub edge_softness: f32,
    /// Tengen: empty, for the bar or each cell.
    pub shadows: Vec<ShadowStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
}

impl ControlStyle for CapacityIndicatorStyle {
    fn tengen() -> Self {
        let [fill, warning, critical] = per_look(
            [
                LEVEL_INDICATOR_COLORS,
                LEVEL_INDICATOR_COLORS_DARK,
                LEVEL_INDICATOR_COLORS_DARK,
            ],
            [
                LEVEL_INDICATOR_COLORS_CONTRAST,
                LEVEL_INDICATOR_COLORS_CONTRAST_DARK,
                LEVEL_INDICATOR_COLORS_CONTRAST_DARK,
            ],
        )
        .map(|rgba| StateValue::new(ShapeStyle::Color(rgba)));
        Self {
            height: LEVEL_INDICATOR.height,
            segment_gap: LEVEL_INDICATOR.segment_gap,
            shape: Shape::RoundedRectangle {
                radius: LEVEL_INDICATOR.radius,
                style: CornerStyle::Continuous,
            },
            track: StateValue::new(ShapeStyle::Color(per_look(
                [LEVEL_TRACK_LIGHT, LEVEL_TRACK_DARK, LEVEL_TRACK_DARK_TINT],
                [
                    GAUGE_CONTRAST_LIGHT.level_track,
                    GAUGE_CONTRAST_DARK.level_track,
                    GAUGE_CONTRAST_DARK_TINT.level_track,
                ],
            ))),
            fill,
            warning,
            critical,
            border: Some((
                ShapeStyle::Color(per_look(
                    [
                        LEVEL_BORDER_LIGHT,
                        LEVEL_BORDER_DARK,
                        LEVEL_BORDER_DARK_TINT,
                    ],
                    [
                        GAUGE_CONTRAST_LIGHT.level_border,
                        GAUGE_CONTRAST_DARK.level_border,
                        GAUGE_CONTRAST_DARK_TINT.level_border,
                    ],
                )),
                HAIRLINE,
            )),
            cell_border_shade: per_look(
                [LEVEL_INDICATOR.discrete_border; 3],
                [
                    LEVEL_INDICATOR.discrete_border_contrast_light,
                    LEVEL_INDICATOR.discrete_border,
                    LEVEL_INDICATOR.discrete_border,
                ],
            ),
            edge_softness: LEVEL_INDICATOR.edge_softness,
            shadows: Vec::new(),
            opacity: 1.0,
            effect: system_effect,
        }
    }
}

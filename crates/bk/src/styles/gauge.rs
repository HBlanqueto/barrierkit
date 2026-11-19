use ctdesign::{CAPTION, Font, GAUGE};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct GaugeStyle {
    /// Tengen: `GAUGE.size`.
    pub size: f32,
    /// Tengen, with `arc`, `marker_size`: `GAUGE.stroke`, `GAUGE.arc` in degrees,
    /// `GAUGE.marker`.
    pub stroke: f32,
    /// Tengen, with `stroke`, `marker_size`: `GAUGE.stroke`, `GAUGE.arc` in degrees,
    /// `GAUGE.marker`.
    pub arc: f32,
    /// Tengen, with `stroke`, `arc`: `GAUGE.stroke`, `GAUGE.arc` in degrees, `GAUGE.marker`.
    pub marker_size: f32,
    /// Tengen, with `marker`, `capacity_track`, `capacity_fill`, `label`: `track`
    /// `Role(TertiaryLabel)`, `marker` `Role(Label)`, `capacity_track`
    /// `Role(ControlBackground)`, `capacity_fill` `Accent`, `label` `Role(Label)`.
    pub track: StateValue<ShapeStyle>,
    /// Tengen, with `track`, `capacity_track`, `capacity_fill`, `label`: `track`
    /// `Role(TertiaryLabel)`, `marker` `Role(Label)`, `capacity_track`
    /// `Role(ControlBackground)`, `capacity_fill` `Accent`, `label` `Role(Label)`.
    pub marker: StateValue<ShapeStyle>,
    /// Tengen, with `track`, `marker`, `capacity_fill`, `label`: `track` `Role(TertiaryLabel)`,
    /// `marker` `Role(Label)`, `capacity_track` `Role(ControlBackground)`, `capacity_fill`
    /// `Accent`, `label` `Role(Label)`.
    pub capacity_track: StateValue<ShapeStyle>,
    /// Tengen, with `track`, `marker`, `capacity_track`, `label`: `track`
    /// `Role(TertiaryLabel)`, `marker` `Role(Label)`, `capacity_track`
    /// `Role(ControlBackground)`, `capacity_fill` `Accent`, `label` `Role(Label)`.
    pub capacity_fill: StateValue<ShapeStyle>,
    /// Tengen, with `track`, `marker`, `capacity_track`, `capacity_fill`: `track`
    /// `Role(TertiaryLabel)`, `marker` `Role(Label)`, `capacity_track`
    /// `Role(ControlBackground)`, `capacity_fill` `Accent`, `label` `Role(Label)`.
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
    /// Tengen: `system_effect`.
    pub effect: ColorEffect,
    /// Tengen: `CAPTION`, the value in the center.
    pub font: Font,
}

impl ControlStyle for GaugeStyle {
    fn tengen() -> Self {
        Self {
            size: GAUGE.size,
            stroke: GAUGE.stroke,
            arc: GAUGE.arc,
            marker_size: GAUGE.marker,
            track: StateValue::new(ShapeStyle::Role(Role::TertiaryLabel)),
            marker: StateValue::new(ShapeStyle::Role(Role::Label)),
            capacity_track: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            capacity_fill: StateValue::new(ShapeStyle::Accent),
            label: StateValue::new(ShapeStyle::Role(Role::Label)),
            opacity: 1.0,
            effect: system_effect,
            font: CAPTION,
        }
    }
}

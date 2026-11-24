use ctdesign::{CAPTION, Font, GAUGE};

use crate::styles::{ControlStyle, Role, ShapeStyle, StateValue};

#[derive(Clone, Debug)]
pub struct GaugeStyle {
    /// Tengen: `GAUGE.size`.
    pub size: f32,
    pub stroke: f32,
    pub arc: f32,
    pub marker_size: f32,
    pub track: StateValue<ShapeStyle>,
    pub marker: StateValue<ShapeStyle>,
    pub capacity_track: StateValue<ShapeStyle>,
    pub capacity_fill: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    /// Tengen: `1.0`.
    pub opacity: f32,
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
            font: CAPTION,
        }
    }
}

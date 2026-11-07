use ctappearance::Resolved;
use ctdesign::{CAPTION, Font, GAUGE};

use super::{ColorEffect, ControlStyle, Role, ShapeStyle, StateValue, system_effect};

#[derive(Clone, Debug)]
pub struct GaugeStyle {
    pub size: f32,
    pub stroke: f32,
    pub arc: f32,
    pub marker_size: f32,
    pub track: StateValue<ShapeStyle>,
    pub marker: StateValue<ShapeStyle>,
    pub capacity_track: StateValue<ShapeStyle>,
    pub capacity_fill: StateValue<ShapeStyle>,
    pub label: StateValue<ShapeStyle>,
    pub opacity: f32,
    pub effect: ColorEffect,
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

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

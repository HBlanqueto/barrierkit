use ctappearance::Resolved;
use ctdesign::{
    CALLOUT, CAPTION, CHART, CHART_SERIES, DURATION_BASE, Font, HAIRLINE, HEADLINE, SPACE_1,
    SPACE_2, TOOLTIP,
};

use super::{
    Animation, ColorEffect, ControlStyle, CornerStyle, Role, Shape, ShapeStyle, StateValue,
    system_effect,
};
use crate::theme::{SurfaceKind, popover_palette};

#[derive(Clone, Debug)]
pub struct ChartStyle {
    pub axis_width: f32,
    pub axis_gap: f32,
    pub spacing: f32,
    pub group_spacing: f32,
    pub legend_gap: f32,
    pub legend_swatch: f32,
    pub tooltip_padding: (f32, f32),
    pub line_width: f32,
    pub point_size: f32,
    pub rule_width: f32,
    pub grid_width: f32,
    pub bar_gap: f32,
    pub bar_radius: f32,
    pub sector_inner: f32,
    pub sector_inset: f32,
    pub sector_radius: f32,
    pub heat_gap: f32,
    pub heat_radius: f32,
    pub stream_hole: f32,
    pub legend_shape: Shape,
    pub tooltip_shape: Shape,
    pub series: Vec<ShapeStyle>,
    pub grid: StateValue<ShapeStyle>,
    pub rule: StateValue<ShapeStyle>,
    pub hover: StateValue<ShapeStyle>,
    pub halo: StateValue<ShapeStyle>,
    pub heat_empty: StateValue<ShapeStyle>,
    pub heat_fill: StateValue<ShapeStyle>,
    pub axis_label: StateValue<ShapeStyle>,
    pub legend_label: StateValue<ShapeStyle>,
    pub tooltip_background: StateValue<ShapeStyle>,
    pub tooltip_label: StateValue<ShapeStyle>,
    pub tooltip_border: Option<(ShapeStyle, f32)>,
    pub area_opacity: f32,
    pub unselected_opacity: f32,
    pub hover_opacity: f32,
    pub effect: ColorEffect,
    pub font: Font,
    pub value_font: Font,
    pub title_font: Font,
    pub animation: Animation,
}

impl ControlStyle for ChartStyle {
    fn tengen() -> Self {
        let menu = popover_palette(&crate::theme::resolved(), SurfaceKind::Toplevel);
        Self {
            axis_width: CHART.y_axis_width,
            axis_gap: CHART.axis_gap,
            spacing: SPACE_1,
            group_spacing: SPACE_2,
            legend_gap: CHART.legend_gap,
            legend_swatch: CHART.legend_swatch,
            tooltip_padding: (TOOLTIP.padding_x, TOOLTIP.padding_y),
            line_width: CHART.line_width,
            point_size: CHART.point_size,
            rule_width: CHART.rule_width,
            grid_width: CHART.grid_width,
            bar_gap: CHART.bar_gap,
            bar_radius: CHART.bar_radius,
            sector_inner: CHART.sector_inner,
            sector_inset: CHART.sector_inset,
            sector_radius: CHART.sector_radius,
            heat_gap: CHART.heat_gap,
            heat_radius: CHART.heat_radius,
            stream_hole: CHART.stream_hole,
            legend_shape: Shape::Circle,
            tooltip_shape: Shape::RoundedRectangle {
                radius: TOOLTIP.radius,
                style: CornerStyle::Circular,
            },
            series: CHART_SERIES.iter().map(|c| ShapeStyle::Color(*c)).collect(),
            grid: StateValue::new(ShapeStyle::Role(Role::Separator)),
            rule: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            hover: StateValue::new(ShapeStyle::Role(Role::TertiaryLabel)),
            halo: StateValue::new(ShapeStyle::Role(Role::ContentBackground)),
            heat_empty: StateValue::new(ShapeStyle::Role(Role::ControlBackground)),
            heat_fill: StateValue::new(ShapeStyle::Accent),
            axis_label: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
            legend_label: StateValue::new(ShapeStyle::Role(Role::Label)),
            tooltip_background: StateValue::new(ShapeStyle::Color(menu.bg)),
            tooltip_label: StateValue::new(ShapeStyle::Color(menu.fg)),
            tooltip_border: Some((ShapeStyle::Color(menu.separator_color), HAIRLINE)),
            area_opacity: CHART.area_opacity,
            unselected_opacity: CHART.unselected_opacity,
            hover_opacity: CHART.hover_opacity,
            effect: system_effect,
            font: CAPTION,
            value_font: HEADLINE,
            title_font: CALLOUT,
            animation: Animation::Ease {
                duration_ms: DURATION_BASE,
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

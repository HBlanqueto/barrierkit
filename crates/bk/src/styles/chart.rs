use ctappearance::Resolved;
use ctdesign::{
    CALLOUT, CAPTION, CHART, CHART_SERIES, DURATION_BASE, Font, HAIRLINE, HEADLINE, SPACE_1,
    SPACE_2, TOOLTIP,
};

use super::{
    Animation, ColorEffect, ControlStyle, CornerStyle, Role, Shape, ShapeStyle, StateValue,
    reduce_motion, system_effect,
};
use crate::theme::{SurfaceKind, popover_palette};

/// One style for `Chart`, `SectorChart`, `HeatMap`, `StreamChart` and
/// `Chart3D`, so a theme colors every chart alike. Each chart draws only the
/// fields of its parts. The height or size of a chart stays an option of the
/// chart (`.height`, `.size`): it is layout.
///
/// States: only `normal`. `Chart3D` keeps its camera controls, a
/// `SegmentedControl`, `LevelSlider`s and a small `Button`, which take their own
/// styles.
#[derive(Clone, Debug)]
pub struct ChartStyle {
    /// Tengen, with `axis_gap`: `CHART.y_axis_width`, `CHART.axis_gap` between an axis label
    /// and the plot. Drawn by: `Chart`; `Chart3D` makes its camera labels twice as wide.
    pub axis_width: f32,
    /// Tengen, with `axis_width`: `CHART.y_axis_width`, `CHART.axis_gap` between an axis label
    /// and the plot. Drawn by: `Chart`; `Chart3D` makes its camera labels twice as wide.
    pub axis_gap: f32,
    /// Tengen: `SPACE_1` between the plot, the labels, the legend entries, a swatch and its
    /// label and the readout rows. Drawn by: `spacing` `Chart`, `SectorChart`, `StreamChart`;
    /// `group_spacing` `SectorChart`, `Chart3D`.
    pub spacing: f32,
    /// Tengen: `SPACE_2` between the sections. Drawn by: `spacing` `Chart`, `SectorChart`,
    /// `StreamChart`; `group_spacing` `SectorChart`, `Chart3D`.
    pub group_spacing: f32,
    /// Tengen, with `legend_swatch`: `CHART.legend_gap` between the entries of a row,
    /// `CHART.legend_swatch`. Drawn by: `Chart`, `SectorChart`, `StreamChart`.
    pub legend_gap: f32,
    /// Tengen, with `legend_gap`: `CHART.legend_gap` between the entries of a row,
    /// `CHART.legend_swatch`. Drawn by: `Chart`, `SectorChart`, `StreamChart`.
    pub legend_swatch: f32,
    /// Tengen: `(TOOLTIP.padding_x, TOOLTIP.padding_y)`. Drawn by: `Chart`, the readout of a
    /// selected point.
    pub tooltip_padding: (f32, f32),
    /// Tengen: `CHART.line_width`. Drawn by: `Chart`; `grid_width` and `rule_width` also the
    /// guides of `StreamChart`, `grid_width` and `point_size` the box and points of `Chart3D`.
    pub line_width: f32,
    /// Tengen: `CHART.point_size`. Drawn by: `Chart`; `grid_width` and `rule_width` also the
    /// guides of `StreamChart`, `grid_width` and `point_size` the box and points of `Chart3D`.
    pub point_size: f32,
    /// Tengen: `CHART.rule_width`. Drawn by: `Chart`; `grid_width` and `rule_width` also the
    /// guides of `StreamChart`, `grid_width` and `point_size` the box and points of `Chart3D`.
    pub rule_width: f32,
    /// Tengen: `CHART.grid_width`. Drawn by: `Chart`; `grid_width` and `rule_width` also the
    /// guides of `StreamChart`, `grid_width` and `point_size` the box and points of `Chart3D`.
    pub grid_width: f32,
    /// Tengen: `CHART.bar_gap`, the share of a category left empty. Drawn by: `Chart`.
    pub bar_gap: f32,
    /// Tengen: `CHART.bar_radius`. Drawn by: `Chart`.
    pub bar_radius: f32,
    /// Tengen: `CHART.sector_inner`, the hole of a donut. Drawn by: `SectorChart`.
    pub sector_inner: f32,
    /// Tengen: `CHART.sector_inset` degrees between slices. Drawn by: `SectorChart`.
    pub sector_inset: f32,
    /// Tengen: `CHART.sector_radius`. Drawn by: `SectorChart`.
    pub sector_radius: f32,
    /// Tengen: `CHART.heat_gap`. Drawn by: `HeatMap`.
    pub heat_gap: f32,
    /// Tengen: `CHART.heat_radius`. Drawn by: `HeatMap`.
    pub heat_radius: f32,
    /// Tengen: `CHART.stream_hole`, the hole of the flow clock. Drawn by: `StreamChart`.
    pub stream_hole: f32,
    /// Tengen: `Circle`. Drawn by: the legends; `Chart`.
    pub legend_shape: Shape,
    /// Tengen: `RoundedRectangle { radius: TOOLTIP.radius, style: Circular }`. Drawn by: the
    /// legends; `Chart`.
    pub tooltip_shape: Shape,
    /// Tengen: `Color` of each of `CHART_SERIES`, repeated when there are more series. Drawn
    /// by: every chart but `HeatMap`; `Chart3D` goes from the first to the third.
    pub series: Vec<ShapeStyle>,
    /// Tengen, with `rule`, `hover`, `halo`: `Role(Separator)`, `Role(SecondaryLabel)`,
    /// `Role(TertiaryLabel)`, `Role(ContentBackground)` around the hovered points. Drawn by:
    /// `Chart`; `grid` also `StreamChart`, `Chart3D`.
    pub grid: StateValue<ShapeStyle>,
    /// Tengen, with `grid`, `hover`, `halo`: `Role(Separator)`, `Role(SecondaryLabel)`,
    /// `Role(TertiaryLabel)`, `Role(ContentBackground)` around the hovered points. Drawn by:
    /// `Chart`; `grid` also `StreamChart`, `Chart3D`.
    pub rule: StateValue<ShapeStyle>,
    /// Tengen, with `grid`, `rule`, `halo`: `Role(Separator)`, `Role(SecondaryLabel)`,
    /// `Role(TertiaryLabel)`, `Role(ContentBackground)` around the hovered points. Drawn by:
    /// `Chart`; `grid` also `StreamChart`, `Chart3D`.
    pub hover: StateValue<ShapeStyle>,
    /// Tengen, with `grid`, `rule`, `hover`: `Role(Separator)`, `Role(SecondaryLabel)`,
    /// `Role(TertiaryLabel)`, `Role(ContentBackground)` around the hovered points. Drawn by:
    /// `Chart`; `grid` also `StreamChart`, `Chart3D`.
    pub halo: StateValue<ShapeStyle>,
    /// Tengen, with `heat_fill`: `Role(ControlBackground)` for the lowest value, `Accent` for
    /// the highest. Drawn by: `HeatMap`.
    pub heat_empty: StateValue<ShapeStyle>,
    /// Tengen, with `heat_empty`: `Role(ControlBackground)` for the lowest value, `Accent` for
    /// the highest. Drawn by: `HeatMap`.
    pub heat_fill: StateValue<ShapeStyle>,
    /// Tengen: `Role(SecondaryLabel)`, `Role(Label)`. Drawn by: `axis_label` `Chart`,
    /// `SectorChart`, `Chart3D`; `legend_label` `Chart`, `SectorChart`, `StreamChart`.
    pub axis_label: StateValue<ShapeStyle>,
    /// Tengen: the center of a donut takes `legend_label` for the value and `axis_label` for
    /// the name. Drawn by: `axis_label` `Chart`, `SectorChart`, `Chart3D`; `legend_label`
    /// `Chart`, `SectorChart`, `StreamChart`.
    pub legend_label: StateValue<ShapeStyle>,
    /// Tengen, with `tooltip_label`: `Color` of the popover background and text. Drawn by:
    /// `Chart`.
    pub tooltip_background: StateValue<ShapeStyle>,
    /// Tengen, with `tooltip_background`: `Color` of the popover background and text. Drawn by:
    /// `Chart`.
    pub tooltip_label: StateValue<ShapeStyle>,
    /// Tengen: `Some((Color of the popover separator, HAIRLINE))`. Drawn by: `Chart`.
    pub tooltip_border: Option<(ShapeStyle, f32)>,
    /// Tengen, with `unselected_opacity`, `hover_opacity`: `CHART.area_opacity`,
    /// `CHART.unselected_opacity` of the other slices, `CHART.hover_opacity` of the hovered row
    /// of horizontal bars. Drawn by: `Chart`; `SectorChart`.
    pub area_opacity: f32,
    /// Tengen, with `area_opacity`, `hover_opacity`: `CHART.area_opacity`,
    /// `CHART.unselected_opacity` of the other slices, `CHART.hover_opacity` of the hovered row
    /// of horizontal bars. Drawn by: `Chart`; `SectorChart`.
    pub unselected_opacity: f32,
    /// Tengen, with `area_opacity`, `unselected_opacity`: `CHART.area_opacity`,
    /// `CHART.unselected_opacity` of the other slices, `CHART.hover_opacity` of the hovered row
    /// of horizontal bars. Drawn by: `Chart`; `SectorChart`.
    pub hover_opacity: f32,
    /// Tengen: `system_effect`. Drawn by: every chart.
    pub effect: ColorEffect,
    /// Tengen: `CAPTION` for the labels, legend and readout. Drawn by: `font` `Chart`,
    /// `SectorChart`, `StreamChart`; `value_font` `SectorChart`; `title_font` `Chart3D`.
    pub font: Font,
    /// Tengen: `HEADLINE` for the value in a donut. Drawn by: `font` `Chart`, `SectorChart`,
    /// `StreamChart`; `value_font` `SectorChart`; `title_font` `Chart3D`.
    pub value_font: Font,
    /// Tengen: `CALLOUT` for the camera labels. Drawn by: `font` `Chart`, `SectorChart`,
    /// `StreamChart`; `value_font` `SectorChart`; `title_font` `Chart3D`.
    pub title_font: Font,
    /// Tengen: `Ease { duration_ms: DURATION_BASE }`: new data blends in and a time axis
    /// slides; `None` makes it jump. Drawn by: `Chart`.
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
        reduce_motion(self, appearance, |style| Self {
            animation: Animation::None,
            ..style
        })
    }
}

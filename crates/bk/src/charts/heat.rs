use std::borrow::Cow;

use ctdesign::{CHART, Rgba};
use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{PaintStyle, Rect as SkRect};
use torin::prelude::{Position, Size};

use super::plot::{mix, normalized, paint};
use super::{Marks, ink};
use crate::styles::{ChartStyle, ControlState, Paints, Theme, resolve};

#[derive(PartialEq)]
struct Heat {
    rows: Vec<Vec<f64>>,
    color: Rgba,
    empty: Rgba,
    marks: Marks,
    layout: LayoutData,
}

impl ElementExt for Heat {
    fn changed(&self, other: &std::rc::Rc<dyn ElementExt>) -> bool {
        !crate::style_diff(self, other).is_empty()
    }

    fn diff(&self, other: &std::rc::Rc<dyn ElementExt>) -> DiffModifies {
        crate::style_diff(self, other)
    }

    fn layout(&'_ self) -> Cow<'_, LayoutData> {
        Cow::Borrowed(&self.layout)
    }

    fn render(&self, context: RenderContext) {
        let area = context.layout_node.visible_area();
        let scale = context.scale_factor as f32;
        let values: Vec<f64> = self.rows.iter().flatten().copied().collect();
        let (min, span) = normalized(&values);
        let rows = self.rows.len().max(1) as f32;
        let cols = self.rows.iter().map(Vec::len).max().unwrap_or(1).max(1) as f32;
        let gap = self.marks.heat_gap * scale;
        let (w, h) = (
            (area.width() - gap * (cols - 1.0)) / cols,
            (area.height() - gap * (rows - 1.0)) / rows,
        );
        let radius = (self.marks.heat_radius * scale).min(w.min(h) / 2.0);
        for (r, row) in self.rows.iter().enumerate() {
            for (c, value) in row.iter().enumerate() {
                let t = ((value - min) / span) as f32;
                let x = area.min_x() + c as f32 * (w + gap);
                let y = area.min_y() + r as f32 * (h + gap);
                let fill = paint(mix(self.empty, self.color, t), PaintStyle::Fill, 0.0);
                context.canvas.draw_round_rect(
                    SkRect::new(x, y, x + w, y + h),
                    radius,
                    radius,
                    &fill,
                );
            }
        }
    }
}

#[derive(PartialEq)]
pub struct HeatMap {
    label: &'static str,
    rows: Vec<Vec<f64>>,
    height: f32,
    style: Theme,
}

impl HeatMap {
    pub fn new(label: &'static str, rows: Vec<Vec<f64>>) -> Self {
        Self {
            label,
            rows,
            height: CHART.height,
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(ChartStyle) -> ChartStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl Component for HeatMap {
    fn render(&self) -> impl IntoElement {
        let style: ChartStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default(), style.effect);
        rect()
            .width(Size::fill())
            .height(Size::px(self.height))
            .a11y_role(AccessibilityRole::Figure)
            .a11y_alt(self.label)
            .child(crate::glass::leaf(Heat {
                rows: self.rows.clone(),
                color: ink(&paints, &style.heat_fill),
                empty: ink(&paints, &style.heat_empty),
                marks: Marks::of(&style),
                layout: crate::glass::layout(Size::fill(), Size::fill(), Position::default()),
            }))
    }
}

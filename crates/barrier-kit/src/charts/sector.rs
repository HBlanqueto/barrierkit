use std::borrow::Cow;

use ct_design::{CAPTION, CHART, HEADLINE, Rgba, SPACE_1, SPACE_2};
use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::prelude::*;
use freya_core::tree::DiffModifies;
use freya_engine::prelude::{PaintStyle, PathBuilder, Rect as SkRect};
use torin::prelude::{Direction, Position, Size};

use super::plot::paint;
use super::scale::{sector_angles, sector_at};
use super::{format_value, legend_entry, series_color};
use crate::style::{Palette, text};

#[derive(PartialEq)]
struct Sectors {
    angles: Vec<(f32, f32)>,
    colors: Vec<Rgba>,
    inner: f32,
    selected: Option<usize>,
    layout: LayoutData,
}

impl ElementExt for Sectors {
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
        let (cx, cy) = (area.center().x, area.center().y);
        let outer = area.width().min(area.height()) / 2.0;
        let inner = outer * self.inner;
        let ring = |radius: f32| SkRect::new(cx - radius, cy - radius, cx + radius, cy + radius);
        for (i, ((start, sweep), color)) in self.angles.iter().zip(&self.colors).enumerate() {
            let inset = if *sweep >= 360.0 {
                0.0
            } else {
                CHART.sector_inset
            };
            let (start, sweep) = (start + inset / 2.0, (sweep - inset).max(0.0));
            if sweep <= 0.0 {
                continue;
            }
            let mut path = PathBuilder::new();
            if inner > 0.0 {
                path.arc_to(ring(outer), start, sweep, true);
                path.arc_to(ring(inner), start + sweep, -sweep, false);
            } else {
                path.move_to((cx, cy));
                path.arc_to(ring(outer), start, sweep, false);
            }
            path.close();
            let mut fill = paint(*color, PaintStyle::Fill, 0.0);
            if self.selected.is_some_and(|selected| selected != i) {
                fill.set_alpha_f(color.a * CHART.unselected_opacity);
            }
            fill.set_path_effect(skia_safe::corner_path_effect::new(
                CHART.sector_radius * context.scale_factor as f32,
            ));
            context.canvas.draw_path(&path.detach(), &fill);
        }
    }
}

#[derive(PartialEq)]
pub struct SectorChart {
    label: &'static str,
    slices: Vec<(String, f64)>,
    donut: bool,
    unit: &'static str,
    size: f32,
}

impl SectorChart {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            slices: Vec::new(),
            donut: false,
            unit: "",
            size: CHART.height,
        }
    }

    pub fn slice(mut self, name: impl Into<String>, value: f64) -> Self {
        self.slices.push((name.into(), value));
        self
    }

    pub fn donut(mut self, donut: bool) -> Self {
        self.donut = donut;
        self
    }

    pub fn unit(mut self, unit: &'static str) -> Self {
        self.unit = unit;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl Component for SectorChart {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let mut selected = use_state(|| None::<usize>);
        let values: Vec<f64> = self.slices.iter().map(|(_, v)| *v).collect();
        let total: f64 = values.iter().sum();
        let colors: Vec<Rgba> = (0..values.len()).map(series_color).collect();
        let angles = sector_angles(&values);
        let hovered = selected.read().and_then(|i| self.slices.get(i));
        let size = self.size;
        let inner = if self.donut { CHART.sector_inner } else { 0.0 };
        let hit_angles = angles.clone();
        let sectors = Sectors {
            angles,
            colors: colors.clone(),
            inner,
            selected: *selected.read(),
            layout: crate::glass::layout(Size::fill(), Size::fill(), Position::default()),
        };
        let legend = self.slices.iter().enumerate().map(|(i, (name, value))| {
            let share = if total > 0.0 {
                value / total * 100.0
            } else {
                0.0
            };
            legend_entry(
                colors[i],
                format!(
                    "{name} {} · {}",
                    format_value(*value, self.unit),
                    format_value(share, "%")
                ),
                &palette,
            )
        });
        rect()
            .direction(Direction::Vertical)
            .spacing(SPACE_2)
            .a11y_role(AccessibilityRole::Figure)
            .a11y_alt(self.label)
            .child(
                rect()
                    .width(Size::px(self.size))
                    .height(Size::px(self.size))
                    .center()
                    .on_pointer_move(move |event: Event<PointerEventData>| {
                        let point = event.element_location();
                        let offset = (point.x as f32 - size / 2.0, point.y as f32 - size / 2.0);
                        let radii = (size / 2.0 * inner, size / 2.0);
                        selected.set_if_modified(sector_at(&hit_angles, offset, radii));
                    })
                    .on_pointer_leave(move |_| selected.set_if_modified(None))
                    .child(crate::glass::leaf(sectors))
                    .maybe(self.donut, |el| {
                        el.child(
                            rect()
                                .position(Position::new_absolute().top(0.0).left(0.0))
                                .width(Size::px(self.size))
                                .height(Size::px(self.size))
                                .center()
                                .direction(Direction::Vertical)
                                .child(text(
                                    hovered.map_or(format_value(total, self.unit), |(_, value)| {
                                        format_value(*value, self.unit)
                                    }),
                                    HEADLINE,
                                    palette.label,
                                ))
                                .maybe_child(hovered.map(|(name, _)| {
                                    text(name.clone(), CAPTION, palette.secondary_label)
                                })),
                        )
                    }),
            )
            .child(
                rect()
                    .direction(Direction::Vertical)
                    .spacing(SPACE_1)
                    .children(legend),
            )
    }
}

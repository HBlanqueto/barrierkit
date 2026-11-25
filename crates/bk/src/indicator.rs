use ctdesign::{HAIRLINE, LEVEL_INDICATOR, Rgba};
use freya_components::cache::AssetAge;
use freya_components::svg_viewer::SvgViewer;
use freya_core::prelude::*;
use std::borrow::Cow;
use torin::prelude::{Direction, Gaps, Position, Size};

use crate::animation::use_animated;
use crate::controlled::commit;
use crate::controls::{dimmed, shadowed};
use crate::elements::{Overrides, forward_overrides};
use crate::interaction::{control_state, use_interaction};
use crate::style::with_font;
use crate::styles::{
    CapacityIndicatorStyle, ControlState, GaugeStyle, Painted, Paints, RatingIndicatorStyle,
    RelevancyIndicatorStyle, ShapeStyle, StateValue, Theme, resolve,
};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

#[derive(PartialEq)]
pub struct CapacityIndicator {
    label: Cow<'static, str>,
    value: f32,
    warning: f32,
    critical: f32,
    segments: Option<u32>,
    style: Theme,
    overrides: Overrides,
}

impl CapacityIndicator {
    pub fn new(label: impl Into<Cow<'static, str>>, value: f32) -> Self {
        Self {
            label: label.into(),
            value,
            warning: 1.0,
            critical: 1.0,
            segments: None,
            style: Theme::tengen(),
            overrides: Overrides::default(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(CapacityIndicatorStyle) -> CapacityIndicatorStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn discrete(mut self, segments: u32) -> Self {
        self.segments = Some(segments.max(1));
        self
    }

    pub fn thresholds(mut self, warning: f32, critical: f32) -> Self {
        self.warning = warning;
        self.critical = critical;
        self
    }
}

pub fn level_color(value: f32, warning: f32, critical: f32) -> usize {
    if value >= critical {
        2
    } else if value >= warning {
        1
    } else {
        0
    }
}

fn soft_cell(style: &CapacityIndicatorStyle, filled: bool, border: Rgba, fill: Rgba) -> Rect {
    let step = HAIRLINE / 2.0;
    let softness = style.edge_softness;
    let [filled_thin, edge, spill, empty_thin] = style.bottom;
    let thin = if filled { filled_thin } else { empty_thin };
    let rings = [
        border,
        border.blend(fill, softness),
        fill.blend(border, softness),
    ];
    let center = rect()
        .width(Size::fill())
        .height(Size::fill())
        .corner_radius(style.ring_radii[rings.len()])
        .background(color(fill));
    let cell = rings
        .iter()
        .enumerate()
        .rev()
        .fold(center, |inner, (index, ring)| {
            let width = BorderWidth {
                top: step,
                right: step,
                bottom: if index == 0 { step - thin } else { step },
                left: step,
            };
            rect()
                .width(Size::fill())
                .height(if index == 0 {
                    Size::px(style.height)
                } else {
                    Size::fill()
                })
                .padding(Gaps::new_all(step))
                .corner_radius(style.ring_radii[index])
                .border(
                    Border::new()
                        .width(width)
                        .alignment(BorderAlignment::Inner)
                        .fill(color(*ring)),
                )
                .child(inner)
        });
    let inset = style.ring_radii[0];
    let line = |top: f32, opacity: f32| {
        rect()
            .position(Position::new_absolute().top(top).left(inset))
            .layer(1)
            .width(Size::func_data(
                move |context| Some(context.parent - 2.0 * inset),
                &inset.to_bits(),
            ))
            .height(Size::px(step))
            .background(color(Rgba {
                a: border.a * opacity,
                ..border
            }))
    };
    cell.maybe(filled, |cell| {
        cell.child(line(style.height - 2.0 * step, edge))
            .child(line(style.height - step, spill))
    })
}

forward_overrides!(CapacityIndicator);

impl Component for CapacityIndicator {
    fn render(&self) -> impl IntoElement {
        let style: CapacityIndicatorStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default());
        let value = self.value.clamp(0.0, 1.0);
        let fill = [&style.fill, &style.warning, &style.critical]
            [level_color(value, self.warning, self.critical)];
        let height = style.height;
        let border = style
            .border
            .as_ref()
            .map_or(Rgba::TRANSPARENT, |(paint, _)| paints.color(paint));
        let shade = style.cell_border_shade;
        let track_cell = || {
            soft_cell(
                &style,
                false,
                border,
                paints.color(style.track.resolve(ControlState::default())),
            )
        };
        let filled_cell = |level: Rgba| {
            let rim = Rgba {
                r: level.r * shade,
                g: level.g * shade,
                b: level.b * shade,
                ..level
            };
            soft_cell(&style, true, rim, level)
        };
        let bar = rect()
            .a11y_role(AccessibilityRole::Meter)
            .a11y_alt(&*self.label)
            .a11y_builder(move |node| node.set_numeric_value(f64::from(value)))
            .width(Size::fill())
            .height(Size::px(height));
        let level = paints.color(fill.resolve(ControlState::default()));
        let mut width = use_state(|| 0.0f32);
        let bar = match self.segments {
            None => bar.child(shadowed(
                rect()
                    .width(Size::fill())
                    .height(Size::fill())
                    .on_sized(move |event: Event<SizedEventData>| {
                        width.set_if_modified(event.area.width());
                    })
                    .child(track_cell().width(Size::fill()))
                    .maybe(value > 0.0, |el| {
                        el.child(
                            rect()
                                .position(Position::new_absolute().top(0.0).left(0.0))
                                .layer(1)
                                .width(Size::px(*width.read() * value))
                                .height(Size::px(height + HAIRLINE / 2.0))
                                .overflow(Overflow::Clip)
                                .child(filled_cell(level).width(Size::px(*width.read()))),
                        )
                    }),
                &style.shadows,
            )),
            Some(segments) => bar
                .direction(Direction::Horizontal)
                .content(torin::prelude::Content::flex())
                .spacing(style.segment_gap)
                .children((0..segments).map(|segment| {
                    let filled = (value * segments as f32 - segment as f32).clamp(0.0, 1.0);
                    if filled.round() == 0.0 {
                        return track_cell().width(Size::flex(1.0)).into_element();
                    }
                    filled_cell(level).width(Size::flex(1.0)).into_element()
                })),
        };
        self.overrides.apply(dimmed(bar, style.opacity))
    }
}

#[derive(PartialEq)]
pub struct RelevancyIndicator {
    label: Cow<'static, str>,
    value: f32,
    style: Theme,
    overrides: Overrides,
}

impl RelevancyIndicator {
    pub fn new(label: impl Into<Cow<'static, str>>, value: f32) -> Self {
        Self {
            label: label.into(),
            value: value.clamp(0.0, 1.0),
            style: Theme::tengen(),
            overrides: Overrides::default(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(RelevancyIndicatorStyle) -> RelevancyIndicatorStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }
}

forward_overrides!(RelevancyIndicator);

impl Component for RelevancyIndicator {
    fn render(&self) -> impl IntoElement {
        let style: RelevancyIndicatorStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default());
        let value = self.value;
        let mut width = use_state(|| 0.0f32);
        let pitch = style.line_width + style.gap;
        let lines = ((*width.read() * value) / pitch).floor() as usize;
        let relevancy = rect()
            .a11y_role(AccessibilityRole::Meter)
            .a11y_alt(&*self.label)
            .a11y_builder(move |node| node.set_numeric_value(f64::from(value)))
            .width(Size::fill())
            .height(Size::px(style.height))
            .painted(&paints, &style.track)
            .on_sized(move |event: Event<SizedEventData>| {
                width.set_if_modified(event.area.width());
            })
            .child(
                rect()
                    .width(Size::px(lines as f32 * pitch))
                    .height(Size::fill())
                    .painted(&paints, &style.filled)
                    .direction(Direction::Horizontal)
                    .spacing(style.gap)
                    .children((0..lines).map(|_| {
                        rect()
                            .width(Size::px(style.line_width))
                            .height(Size::fill())
                            .background(paints.fill(&style.line))
                            .into_element()
                    })),
            );
        self.overrides.apply(dimmed(relevancy, style.opacity))
    }
}

#[derive(PartialEq)]
pub struct Gauge {
    label: Cow<'static, str>,
    value: f32,
    current: Cow<'static, str>,
    capacity: bool,
    style: Theme,
    overrides: Overrides,
}

impl Gauge {
    pub fn new(
        label: impl Into<Cow<'static, str>>,
        value: f32,
        current: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            label: label.into(),
            value: value.clamp(0.0, 1.0),
            current: current.into(),
            capacity: false,
            style: Theme::tengen(),
            overrides: Overrides::default(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(GaugeStyle) -> GaugeStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn capacity(mut self, capacity: bool) -> Self {
        self.capacity = capacity;
        self
    }
}

pub fn gauge_marker(style: &GaugeStyle, value: f32) -> (f32, f32) {
    let radius = (style.size - style.stroke) / 2.0;
    let start = 90.0 + (360.0 - style.arc) / 2.0;
    let angle = (start + style.arc * value).to_radians();
    let center = style.size / 2.0;
    (center + radius * angle.cos(), center + radius * angle.sin())
}

forward_overrides!(Gauge);

impl Component for Gauge {
    fn render(&self) -> impl IntoElement {
        let style: GaugeStyle = resolve(&self.style);
        let state = ControlState::default();
        let paints = Paints::current(state);
        let size = style.size;
        let center = size / 2.0;
        let radius = (size - style.stroke) / 2.0;
        let circumference = std::f32::consts::TAU * radius;
        let animated = use_animated(self.value, style.animation);
        let value = if self.capacity { self.value } else { animated };
        let (arc, start) = if self.capacity {
            (360.0, -90.0)
        } else {
            (style.arc, 90.0 + (360.0 - style.arc) / 2.0)
        };
        let arc_svg = |length: f32| {
            format!(
                r#"<svg viewBox="0 0 {size} {size}"><circle cx="{center}" cy="{center}" r="{radius}" fill="none" stroke="currentColor" stroke-width="{stroke}" stroke-linecap="round" stroke-dasharray="{length} {circumference}" transform="rotate({start} {center} {center})"/></svg>"#,
                stroke = style.stroke,
            )
        };
        let ring = arc_svg(circumference * arc / 360.0);
        let (x, y) = gauge_marker(&style, value);
        let (track, marker, marker_ink) = if self.capacity {
            (
                &style.capacity_track,
                arc_svg(circumference * value),
                &style.capacity_fill,
            )
        } else {
            (
                &style.track,
                format!(
                    r#"<svg viewBox="0 0 {size} {size}"><circle cx="{x}" cy="{y}" r="{dot}" fill="currentColor"/></svg>"#,
                    dot = style.marker_size / 2.0,
                ),
                &style.marker,
            )
        };
        let layer = |svg: String, paint: &StateValue<ShapeStyle>| {
            SvgViewer::new(Bytes::from(svg))
                .width(Size::px(size))
                .height(Size::px(size))
                .color(color(paints.color(paint.resolve(state))))
                .show_loader(false)
        };
        let value = self.value;
        let gauge = rect()
            .a11y_role(AccessibilityRole::Meter)
            .a11y_alt(&*self.label)
            .a11y_builder(move |node| node.set_numeric_value(f64::from(value)))
            .width(Size::px(size))
            .height(Size::px(size))
            .child(layer(ring, track))
            .child(
                rect()
                    .position(Position::new_absolute())
                    .child(layer(marker, marker_ink).asset_age(AssetAge::zero())),
            )
            .child(
                rect()
                    .position(Position::new_absolute())
                    .width(Size::px(size))
                    .height(Size::px(size))
                    .center()
                    .child(
                        with_font(label().text(self.current.clone()), style.font)
                            .max_lines(1)
                            .color(paints.fill(&style.label)),
                    ),
            );
        self.overrides.apply(dimmed(gauge, style.opacity))
    }
}

#[derive(PartialEq)]
pub struct RatingIndicator {
    label: Cow<'static, str>,
    value: State<u32>,
    editable: bool,
    style: Theme,
    overrides: Overrides,
}

impl RatingIndicator {
    pub fn new(label: impl Into<Cow<'static, str>>, value: State<u32>) -> Self {
        Self {
            label: label.into(),
            value,
            editable: false,
            style: Theme::tengen(),
            overrides: Overrides::default(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(RatingIndicatorStyle) -> RatingIndicatorStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }
}

forward_overrides!(RatingIndicator);

impl Component for RatingIndicator {
    fn render(&self) -> impl IntoElement {
        let style: RatingIndicatorStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state);
        let bound = self.value;
        let value = bound.read().min(LEVEL_INDICATOR.stars);
        let editable = self.editable;
        let stars = (1..=LEVEL_INDICATOR.stars).map(|star| {
            let ink = if star <= value {
                &style.star
            } else {
                &style.empty_star
            };
            rect()
                .width(Size::px(style.star_size.1))
                .height(Size::px(style.star_size.1))
                .center()
                .offset_y(style.star_drop)
                .maybe(editable, |el| el.on_press(move |_| commit(bound, star)))
                .child(symbol(
                    Symbol::StarFill,
                    style.star_size.0,
                    paints.color(ink.resolve(state)),
                ))
                .into_element()
        });
        let rating = interaction
            .attach(rect(), editable)
            .a11y_role(AccessibilityRole::Slider)
            .a11y_alt(&*self.label)
            .a11y_builder(move |node| {
                node.set_numeric_value(f64::from(value));
                node.set_min_numeric_value(0.0);
                node.set_max_numeric_value(f64::from(LEVEL_INDICATOR.stars));
                node.set_numeric_value_step(1.0);
            })
            .maybe(editable, |el| {
                el.on_key_down(move |event: Event<KeyboardEventData>| match event.key {
                    Key::Named(NamedKey::ArrowLeft | NamedKey::ArrowDown) => {
                        commit(bound, value.saturating_sub(1))
                    }
                    Key::Named(NamedKey::ArrowRight | NamedKey::ArrowUp) => {
                        commit(bound, (value + 1).min(LEVEL_INDICATOR.stars))
                    }
                    _ => {}
                })
            })
            .direction(Direction::Horizontal)
            .children(stars);
        self.overrides
            .apply(interaction.focus_ring(dimmed(rating, style.opacity), 0.0))
    }
}

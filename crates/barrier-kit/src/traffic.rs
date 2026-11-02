//! Traffic-light window buttons for client-side windows.

use freya_components::svg_viewer::SvgViewer;
use freya_core::prelude::*;
use torin::prelude::{Direction, Size};
use vow_appearance::traffic::{self, TrafficInput, TrafficPointer};
use vow_appearance::{TrafficLightStyle, palette as base_palette};
use vow_design::{ButtonKind, Rgba, TITLEBAR, TRAFFIC_GLYPHS, Triangle};

use crate::ProbeExt;
use crate::theme;

#[derive(PartialEq)]
pub struct TrafficLights {
    input: Option<TrafficInput>,
    focused: bool,
    zoom_supported: bool,
    fixed_width: bool,
    close_only: bool,
    vertical: bool,
    round: bool,
    on_action: Option<Callback<ButtonKind, ()>>,
    probe: Option<&'static str>,
}

impl Default for TrafficLights {
    fn default() -> Self {
        Self::new()
    }
}

impl TrafficLights {
    pub fn new() -> Self {
        Self {
            input: None,
            focused: true,
            zoom_supported: true,
            fixed_width: false,
            close_only: false,
            vertical: false,
            round: false,
            on_action: None,
            probe: None,
        }
    }

    pub fn input(mut self, input: TrafficInput) -> Self {
        self.input = Some(input);
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn zoom_supported(mut self, supported: bool) -> Self {
        self.zoom_supported = supported;
        self
    }

    pub fn fixed_width(mut self, fixed: bool) -> Self {
        self.fixed_width = fixed;
        self
    }

    pub fn close_only(mut self, close_only: bool) -> Self {
        self.close_only = close_only;
        self
    }

    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    pub fn round(mut self, round: bool) -> Self {
        self.round = round;
        self
    }

    pub fn on_action(mut self, callback: impl Into<Callback<ButtonKind, ()>>) -> Self {
        self.on_action = Some(callback.into());
        self
    }
}

impl ProbeExt for TrafficLights {
    fn probe(mut self, id: &'static str) -> Self {
        self.probe = Some(id);
        self
    }
}

impl Component for TrafficLights {
    fn render(&self) -> impl IntoElement {
        let resolved = crate::theme::resolved();
        let inactive = base_palette(resolved.scheme).traffic_lights_inactive;
        let mut pointer = use_state(TrafficPointer::default);
        let input = self.input.unwrap_or_else(|| {
            let pointer = pointer.read();
            TrafficInput {
                focused: self.focused,
                group_hovered: pointer.hovered(),
                pressed: pointer.pressed_visual(),
                zoom_supported: self.zoom_supported,
            }
        });
        let visual_input = if self.fixed_width {
            TrafficInput {
                zoom_supported: true,
                ..input
            }
        } else {
            input
        };
        let mut visuals = traffic::visual(
            resolved.lights,
            resolved.traffic_style,
            inactive,
            visual_input,
        );
        if self.close_only {
            for (index, visual) in visuals.iter_mut().enumerate() {
                if TITLEBAR.buttons.order[index].kind != ButtonKind::Close {
                    visual.fill = inactive;
                    visual.glyph = None;
                }
            }
        }
        let close_only = self.close_only;
        let interactive = self.input.is_none();
        let action = self.on_action.clone();
        let zoom_supported = self.zoom_supported;
        let fixed_width = self.fixed_width;
        let (_, _, diameter) = traffic::button_rect(0);
        let gap = light_gap();
        let corner_radius = if self.round {
            diameter as f32 / 2.0
        } else {
            TITLEBAR.buttons.corner_radius as f32
        };

        let lights = visuals.iter().enumerate().map(|(index, visual)| {
            let kind = TITLEBAR.buttons.order[index].kind;
            let mut hover = pointer;
            let mut press = pointer;
            let mut release = pointer;
            let mut leave = pointer;
            let action = action.clone();
            let clickable = if close_only {
                kind == ButtonKind::Close
            } else {
                zoom_supported || fixed_width || kind != ButtonKind::Maximize
            };
            let child_probe = self.probe.map(|base| match kind {
                ButtonKind::Close => format!("{base}.close").into_boxed_str(),
                ButtonKind::Minimize => format!("{base}.minimize").into_boxed_str(),
                ButtonKind::Maximize => format!("{base}.zoom").into_boxed_str(),
            });
            rect()
                .width(Size::px(diameter as f32))
                .height(Size::px(diameter as f32))
                .corner_radius(corner_radius)
                .background(theme::color(visual.fill))
                .center()
                .maybe_child(visual.glyph.map(|glyph| {
                    SvgViewer::new((
                        format!("traffic-{index}-{:?}-{fixed_width}", glyph.to_rgba8()),
                        Bytes::from(glyph_svg(index, glyph, fixed_width)),
                    ))
                    .width(Size::px(TRAFFIC_GLYPHS.box_size))
                    .height(Size::px(TRAFFIC_GLYPHS.box_size))
                    .color(theme::color(glyph))
                    .show_loader(false)
                    .into_element()
                }))
                .maybe(interactive, |light| {
                    light
                        .on_pointer_enter(move |_| {
                            let mut p = *hover.read();
                            if p.on_button_enter(kind) {
                                hover.set(p);
                            }
                        })
                        .on_pointer_leave(move |_| {
                            let mut p = *leave.read();
                            if p.on_leave_button() {
                                leave.set(p);
                            }
                        })
                        .on_mouse_down(move |event: Event<MouseEventData>| {
                            if event.button == Some(MouseButton::Left) && clickable {
                                let mut p = *press.read();
                                p.on_press(Some(kind), true);
                                press.set(p);
                            }
                        })
                        .on_mouse_up(move |event: Event<MouseEventData>| {
                            if event.button != Some(MouseButton::Left) {
                                return;
                            }
                            let mut p = *release.read();
                            let fired = p.on_release(Some(kind), true);
                            release.set(p);
                            if let (Some(kind), Some(action)) = (fired, &action) {
                                action.call(kind);
                            }
                        })
                })
                .maybe(child_probe.is_some(), |light| {
                    let id = child_probe.unwrap();
                    light.a11y_builder(move |node| node.set_author_id(id))
                })
                .into_element()
        });

        let (length, thickness) = (traffic::group_rect().2 as f32, diameter as f32);
        let (width, height, direction) = if self.vertical {
            (thickness, length, Direction::Vertical)
        } else {
            (length, thickness, Direction::Horizontal)
        };
        let mut root = rect()
            .width(Size::px(width))
            .height(Size::px(height))
            .direction(direction)
            .spacing(gap as f32)
            .maybe(interactive, |group| {
                group
                    .on_pointer_over(move |_| {
                        let mut p = *pointer.read();
                        if p.on_group_enter() {
                            pointer.set(p);
                        }
                    })
                    .on_pointer_leave(move |_| {
                        let mut p = *pointer.read();
                        if p.on_leave() {
                            pointer.set(p);
                        }
                    })
            })
            .children(lights);
        if let Some(id) = self.probe {
            root = root.a11y_builder(move |node| node.set_author_id(id));
        }
        root
    }
}

pub fn traffic_lights_view(
    input: TrafficInput,
    lights: [Rgba; 3],
    style: TrafficLightStyle,
    inactive: Rgba,
) -> Element {
    let visuals = traffic::visual(lights, style, inactive, input);
    let (_, _, diameter) = traffic::button_rect(0);
    let gap = light_gap();
    rect()
        .width(Size::px(traffic::group_rect().2 as f32))
        .height(Size::px(diameter as f32))
        .direction(Direction::Horizontal)
        .spacing(gap as f32)
        .children(visuals.iter().enumerate().map(|(index, visual)| {
            rect()
                .width(Size::px(diameter as f32))
                .height(Size::px(diameter as f32))
                .corner_radius(TITLEBAR.buttons.corner_radius as f32)
                .background(theme::color(visual.fill))
                .center()
                .maybe_child(visual.glyph.map(|glyph| {
                    SvgViewer::new((
                        format!("traffic-view-{index}-{:?}-false", glyph.to_rgba8()),
                        Bytes::from(glyph_svg(index, glyph, false)),
                    ))
                    .width(Size::px(TRAFFIC_GLYPHS.box_size))
                    .height(Size::px(TRAFFIC_GLYPHS.box_size))
                    .color(theme::color(glyph))
                    .show_loader(false)
                    .into_element()
                }))
                .into_element()
        }))
        .into_element()
}

fn light_gap() -> f64 {
    f64::from(TITLEBAR.buttons.spacing)
}

fn glyph_svg(index: usize, color: Rgba, fixed_width: bool) -> String {
    let [r, g, b, a] = color.to_rgba8();
    let color = format!("#{r:02x}{g:02x}{b:02x}");
    let opacity = f32::from(a) / 255.0;
    let box_size = TRAFFIC_GLYPHS.box_size;
    match index {
        0 | 1 => {
            let segments = if index == 0 {
                TRAFFIC_GLYPHS.close
            } else {
                TRAFFIC_GLYPHS.minimize
            };
            let path = segments
                .iter()
                .map(|segment| {
                    format!(
                        "M{} {} L{} {}",
                        segment.a.0, segment.a.1, segment.b.0, segment.b.1
                    )
                })
                .collect::<Vec<_>>()
                .join(" ");
            format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{box_size}\" height=\"{box_size}\" viewBox=\"0 0 {box_size} {box_size}\"><path d=\"{path}\" fill=\"none\" stroke=\"{color}\" stroke-opacity=\"{opacity}\" stroke-width=\"{}\" stroke-linecap=\"round\"/></svg>",
                TRAFFIC_GLYPHS.stroke
            )
        }
        _ => {
            let segments = if fixed_width {
                TRAFFIC_GLYPHS.plus
            } else {
                &[]
            };
            if !segments.is_empty() {
                let path = segments
                    .iter()
                    .map(|segment| {
                        format!(
                            "M{} {} L{} {}",
                            segment.a.0, segment.a.1, segment.b.0, segment.b.1
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                format!(
                    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{box_size}\" height=\"{box_size}\" viewBox=\"0 0 {box_size} {box_size}\"><path d=\"{path}\" fill=\"none\" stroke=\"{color}\" stroke-opacity=\"{opacity}\" stroke-width=\"{}\" stroke-linecap=\"round\"/></svg>",
                    TRAFFIC_GLYPHS.stroke
                )
            } else {
                let polygons = TRAFFIC_GLYPHS
                    .zoom
                    .iter()
                    .map(polygon)
                    .collect::<Vec<_>>()
                    .join("");
                format!(
                    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{box_size}\" height=\"{box_size}\" viewBox=\"0 0 {box_size} {box_size}\"><g fill=\"{color}\" fill-opacity=\"{opacity}\">{polygons}</g></svg>"
                )
            }
        }
    }
}

fn polygon(triangle: &Triangle) -> String {
    let points = triangle
        .0
        .iter()
        .map(|(x, y)| format!("{x},{y}"))
        .collect::<Vec<_>>()
        .join(" ");
    format!("<polygon points=\"{points}\"/>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_traffic_lights_sit_at_their_token_positions() {
        let xs = [0, 1, 2].map(|index| traffic::button_rect(index).0);
        assert_eq!(xs, [16.0, 36.0, 56.0], "12 px lights, 8 px apart");
        assert_eq!(light_gap(), 8.0);
        assert_eq!(traffic::button_rect(0).2, 12.0);
    }
}

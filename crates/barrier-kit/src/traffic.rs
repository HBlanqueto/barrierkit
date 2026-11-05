use freya_components::svg_viewer::SvgViewer;
use freya_core::prelude::*;
use torin::prelude::{Direction, Size};
use vow_appearance::TrafficLightShape;
use vow_appearance::palette as base_palette;
use vow_appearance::traffic::{self, TrafficInput, TrafficPointer};
use vow_design::{ButtonKind, Rgba, TITLEBAR, TRAFFIC_GLYPHS, TRAFFIC_RIM_WIDTH};

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
        let palette = base_palette(resolved.scheme);
        let (inactive, rim) = (palette.traffic_lights_inactive, palette.traffic_rim);
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
            rim,
            visual_input,
        );
        if self.close_only {
            for (index, visual) in visuals.iter_mut().enumerate() {
                if TITLEBAR.buttons.order[index].kind != ButtonKind::Close {
                    *visual = traffic::ButtonVisual::idle(inactive, rim);
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
        let shape = if self.round {
            TrafficLightShape::Round
        } else {
            TrafficLightShape::Square
        };
        let corner_radius = shape.corner_radius(diameter as f32);

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
                .border(rim_border(visual.rim))
                .center()
                .maybe_child(visual.glyph.map(|glyph| {
                    SvgViewer::new((
                        format!("traffic-{index}-{:?}-{fixed_width}", glyph.to_rgba8()),
                        Bytes::from(traffic::glyph_svg(
                            index,
                            glyph,
                            fixed_width,
                            TRAFFIC_GLYPHS.box_size,
                        )),
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
                .map(child_probe, |light, id| {
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
        let root = rect()
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
        root.map(self.probe, ProbeExt::probe)
    }
}

fn rim_border(color: Rgba) -> Border {
    Border::new()
        .width(TRAFFIC_RIM_WIDTH)
        .alignment(BorderAlignment::Inner)
        .fill(theme::color(color))
}

fn light_gap() -> f64 {
    f64::from(TITLEBAR.buttons.spacing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_traffic_lights_sit_at_their_token_positions() {
        let xs = [0, 1, 2].map(|index| traffic::button_rect(index).0);
        assert_eq!(xs, [9.0, 32.0, 55.0], "14 px lights, 9 px apart");
        assert_eq!(light_gap(), 9.0);
        assert_eq!(traffic::button_rect(0).2, 14.0);
    }
}

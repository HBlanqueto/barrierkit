use ctappearance::traffic::{self, TrafficInput, TrafficPointer};
use ctdesign::{ButtonKind, TITLEBAR};
use freya_components::svg_viewer::SvgViewer;
use freya_core::prelude::*;
use torin::prelude::{Direction, Size};

use crate::ProbeExt;
use crate::controls::{dimmed, inner_border};
use crate::i18n::text;
use crate::styles::{
    Axis, ControlState, LayoutDirection, Paints, Shape, Theme, TrafficLightsStyle, resolve,
};
use crate::theme;

#[derive(PartialEq)]
pub struct TrafficLights {
    input: Option<TrafficInput>,
    focused: bool,
    zoom_supported: bool,
    fixed_width: bool,
    close_only: bool,
    style: Theme,
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
            style: Theme::tengen(),
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

    pub fn vertical(self, vertical: bool) -> Self {
        let axis = if vertical {
            Axis::Vertical
        } else {
            Axis::Horizontal
        };
        self.style(move |s| TrafficLightsStyle { axis, ..s })
    }

    pub fn round(self, round: bool) -> Self {
        if round {
            self.style(|s| TrafficLightsStyle {
                shape: Shape::Circle,
                ..s
            })
        } else {
            self
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(TrafficLightsStyle) -> TrafficLightsStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
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
        let style: TrafficLightsStyle = resolve(&self.style);
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
        let zoom_shown = input.zoom_supported || self.fixed_width;
        let close_blocked = crate::views::feedback::OpenSheets::any();
        let close_only = self.close_only;
        let interactive = self.input.is_none();
        let action = self.on_action.clone();
        let zoom_supported = self.zoom_supported;
        let fixed_width = self.fixed_width;
        let diameter = style.diameter;
        let corner_radius =
            style
                .shape
                .corner_radius(diameter, diameter, LayoutDirection::LeftToRight);

        let lights = TITLEBAR
            .buttons
            .order
            .iter()
            .enumerate()
            .map(|(index, button)| {
                let kind = button.kind;
                let admitted = if close_only {
                    kind == ButtonKind::Close
                } else {
                    zoom_shown || kind != ButtonKind::Maximize
                } && !(close_blocked && kind == ButtonKind::Close);
                let state = ControlState {
                    hovered: input.group_hovered,
                    pressed: input.pressed == Some(kind) && (input.focused || input.group_hovered),
                    focused: input.focused,
                    disabled: !admitted,
                };
                let paints = Paints::current(state, style.effect);
                let fill = paints.fill(&style.fills[index]);
                let rim = (style.rims[index].resolve(state).clone(), style.rim_width);
                let glyph = paints.color(style.glyphs[index].resolve(state));
                let mut hover = pointer;
                let mut press = pointer;
                let mut release = pointer;
                let mut leave = pointer;
                let action = action.clone();
                let clickable = if close_only {
                    kind == ButtonKind::Close
                } else {
                    zoom_supported || fixed_width || kind != ButtonKind::Maximize
                } && !(close_blocked && kind == ButtonKind::Close);
                let child_probe = self.probe.map(|base| match kind {
                    ButtonKind::Close => format!("{base}.close").into_boxed_str(),
                    ButtonKind::Minimize => format!("{base}.minimize").into_boxed_str(),
                    ButtonKind::Maximize => format!("{base}.zoom").into_boxed_str(),
                });
                let name = match kind {
                    ButtonKind::Close => text!("window-close"),
                    ButtonKind::Minimize => text!("window-minimize"),
                    ButtonKind::Maximize => text!("window-zoom"),
                };
                rect()
                    .a11y_role(AccessibilityRole::Button)
                    .a11y_alt(name)
                    .width(Size::px(diameter))
                    .height(Size::px(diameter))
                    .corner_radius(corner_radius)
                    .background(fill)
                    .border(inner_border(&paints, &rim))
                    .center()
                    .maybe_child((glyph.a > 0.0).then(|| {
                        SvgViewer::new((
                            format!("traffic-{index}-{:?}-{fixed_width}", glyph.to_rgba8()),
                            Bytes::from(traffic::glyph_svg(
                                index,
                                glyph,
                                fixed_width,
                                style.glyph_size,
                            )),
                        ))
                        .width(Size::px(style.glyph_size))
                        .height(Size::px(style.glyph_size))
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

        let (length, thickness) = (style.length(), diameter);
        let (width, height, direction) = match style.axis {
            Axis::Vertical => (thickness, length, Direction::Vertical),
            Axis::Horizontal => (length, thickness, Direction::Horizontal),
        };
        let root = rect()
            .width(Size::px(width))
            .height(Size::px(height))
            .direction(direction)
            .spacing(style.spacing)
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
        dimmed(root, style.opacity).map(self.probe, ProbeExt::probe)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_traffic_lights_sit_at_their_token_positions() {
        let xs = [0, 1, 2].map(|index| traffic::button_rect(index).0);
        assert_eq!(xs, [9.0, 32.0, 55.0], "14 px lights, 9 px apart");
        assert_eq!(traffic::button_rect(0).2, 14.0);
    }
}

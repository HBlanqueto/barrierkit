use ctappearance::traffic::{ButtonVisual, Glyph, TrafficInput, TrafficPointer};
use ctdesign::{ButtonKind, TITLEBAR, TRAFFIC_MARGIN};
use freya_core::elements::image::image;
use freya_core::prelude::*;
use torin::prelude::{Direction, Position, Size};

use crate::ProbeExt;
use crate::controls::dimmed;
use crate::elements::{Overrides, forward_overrides};
use crate::i18n::text;
use crate::styles::{Animation, Axis, ControlState, Theme, TrafficLightsStyle, resolve};
use crate::traffic_raster::{Layers, light};

#[derive(PartialEq)]
pub struct TrafficLights {
    input: Option<TrafficInput>,
    focused: bool,
    zoom_supported: bool,
    fixed_width: bool,
    fullscreen: bool,
    close_only: bool,
    style: Theme,
    on_action: Option<Callback<ButtonKind, ()>>,
    probe: Option<&'static str>,
    overrides: Overrides,
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
            fullscreen: false,
            close_only: false,
            style: Theme::tengen(),
            on_action: None,
            probe: None,
            overrides: Overrides::default(),
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

    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
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

fn light_images(
    style: &TrafficLightsStyle,
    visual: &ButtonVisual,
    (index, fixed_width, fullscreen): (usize, bool, bool),
    exit_mix: f32,
) -> Vec<Element> {
    let side = style.diameter + 2.0 * TRAFFIC_MARGIN;
    let layer = |glyph, layers, opacity: f32| {
        light(
            visual,
            style.halo,
            style.backdrop,
            style.ring_width,
            style.diameter,
            glyph,
            layers,
        )
        .map(|handle| {
            rect()
                .position(
                    Position::new_absolute()
                        .top(-TRAFFIC_MARGIN)
                        .left(-TRAFFIC_MARGIN),
                )
                .width(Size::px(side))
                .height(Size::px(side))
                .opacity(opacity)
                .child(image(handle).width(Size::px(side)).height(Size::px(side)))
                .into_element()
        })
    };
    let crossing = index == 2 && visual.glyph.is_some() && visual.dodge == 0.0;
    let crossing = crossing && exit_mix > 0.0 && exit_mix < 1.0;
    if crossing {
        [
            layer(Glyph::ZoomEnter, Layers::Disc, 1.0),
            layer(
                Glyph::of(index, fixed_width, false),
                Layers::GlyphOnly,
                1.0 - exit_mix,
            ),
            layer(Glyph::ZoomExit, Layers::GlyphOnly, exit_mix),
        ]
        .into_iter()
        .flatten()
        .collect()
    } else {
        layer(Glyph::of(index, fixed_width, fullscreen), Layers::All, 1.0)
            .into_iter()
            .collect()
    }
}

forward_overrides!(TrafficLights);

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
                fullscreen: self.fullscreen,
            }
        });
        let animation = if *crate::animation::use_reduced_motion().read() {
            Animation::None
        } else {
            style.animation
        };
        let exit_mix =
            crate::animation::use_animated(f32::from(u8::from(input.fullscreen)), animation);
        let zoom_shown = input.zoom_supported || self.fixed_width;
        let close_blocked = crate::views::feedback::OpenSheets::any();
        let close_only = self.close_only;
        let interactive = self.input.is_none();
        let action = self.on_action.clone();
        let fixed_width = self.fixed_width;
        let diameter = style.diameter;

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
                    (zoom_shown || kind != ButtonKind::Maximize)
                        && (!input.fullscreen || kind != ButtonKind::Minimize)
                } && !(close_blocked && kind == ButtonKind::Close);
                let state = ControlState {
                    hovered: input.group_hovered,
                    pressed: input.pressed == Some(kind) && (input.focused || input.group_hovered),
                    focused: input.focused,
                    disabled: !admitted,
                };
                let visual = *style.looks[index].resolve(state);
                let mut hover = pointer;
                let mut press = pointer;
                let mut release = pointer;
                let mut leave = pointer;
                let action = action.clone();
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
                    .children(light_images(
                        &style,
                        &visual,
                        (index, fixed_width, input.fullscreen),
                        exit_mix,
                    ))
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
                                event.stop_propagation();
                                if event.button == Some(MouseButton::Left) && admitted {
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
            .children(lights)
            .maybe(close_blocked, |group| {
                group.layer(crate::views::feedback::SHEET_TRAFFIC_LIGHTS)
            });
        let root = dimmed(root, style.opacity).map(self.probe, ProbeExt::probe);
        self.overrides.apply(root)
    }
}

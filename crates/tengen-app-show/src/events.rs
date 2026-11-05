use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use barrier_kit::adornments::badge;
use barrier_kit::controls::{Button, ButtonStyle, TextField};
use barrier_kit::interaction::use_interaction;
use barrier_kit::platform::Timers;
use barrier_kit::style::{Palette, text};
use barrier_kit::symbol::{Symbol, symbol};
use barrier_kit::theme::color;
use freya_animation::prelude::{
    AnimNum, AnimSequential, Ease, Function, OnChange, OnCreation, use_animation,
    use_animation_transition, use_animation_with_dependencies,
};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Position, Size};
use vow_design::{
    BADGE, CALLOUT, COLLECTION, DURATION_BASE, DURATION_SLOW_MS, HEADLINE, Rgba, SPACE_2,
    SPLIT_VIEW, TABLE, TEXT,
};

use crate::motion::{spring, time, unit, use_reduced_motion};

const PASSWORD: &str = "tengen";
const SHAKE: f32 = 8.0;
const SLIDE_IN: f32 = 24.0;
const TOAST_HOLD_MS: u64 = 1500;

fn card(base: Rect, fill: Rgba) -> Rect {
    base.width(Size::px(SPLIT_VIEW.inspector))
        .height(Size::px(TABLE.row_height * 2.0))
        .corner_radius(COLLECTION.radius)
        .center()
        .background(color(fill))
}

fn horizontal() -> Rect {
    rect()
        .direction(Direction::Horizontal)
        .cross_align(Alignment::Center)
        .spacing(SPACE_2)
}

#[derive(PartialEq)]
pub struct HoverCard;

impl Component for HoverCard {
    fn render(&self) -> impl IntoElement {
        let reduced_motion = use_reduced_motion();
        let interaction = use_interaction();
        let hovered = use_reactive(&interaction.hovered());
        let animation = use_animation_transition(hovered, move |from, to| {
            AnimNum::new(unit(from), unit(to))
                .time(time(reduced_motion, DURATION_BASE))
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        let progress = animation.get().value();
        let palette = Palette::current();
        card(
            interaction.attach(rect(), true),
            palette
                .control_background
                .mix(palette.control_hover, progress),
        )
        .scale(1.0 + 0.05 * progress)
        .child(text("Pasa el puntero", TEXT, palette.label))
    }
}

#[derive(PartialEq)]
pub struct PressCard;

impl Component for PressCard {
    fn render(&self) -> impl IntoElement {
        let reduced_motion = use_reduced_motion();
        let interaction = use_interaction();
        let pressed = use_reactive(&interaction.pressed());
        let animation = use_animation_transition(pressed, move |from, to| {
            let bounce = if to { 0.0 } else { 0.5 };
            spring(reduced_motion, unit(from), unit(to), DURATION_BASE, bounce)
        });
        let palette = Palette::current();
        card(interaction.attach(rect(), true), palette.control_background)
            .scale(1.0 - 0.06 * animation.get().current())
            .child(text("Mantén pulsado", TEXT, palette.label))
    }
}

#[derive(PartialEq)]
pub struct ShakeField;

impl Component for ShakeField {
    fn render(&self) -> impl IntoElement {
        let reduced_motion = use_reduced_motion();
        let password = use_state(String::new);
        let mut welcomed = use_state(|| false);
        let shake = use_animation(move |_| {
            AnimNum::new(0.0, 1.0)
                .time(time(reduced_motion, DURATION_SLOW_MS))
                .function(Function::Linear)
        });
        let welcome = use_animation_transition(welcomed, move |from, to| {
            spring(reduced_motion, unit(from), unit(to), DURATION_SLOW_MS, 0.5)
        });
        let progress = shake.get().value();
        let offset = (progress * std::f32::consts::PI * 6.0).sin() * (1.0 - progress) * SHAKE;
        let greeting = welcome.get().current();
        let mut replay = shake;
        let palette = Palette::current();
        rect()
            .spacing(SPACE_2)
            .child(
                rect().offset_x(offset).child(
                    horizontal()
                        .child(
                            rect()
                                .width(Size::px(SPLIT_VIEW.inspector))
                                .child(TextField::new(password, "Contraseña")),
                        )
                        .child(Button::new("Entrar").style(ButtonStyle::Primary).on_press(
                            move |_| {
                                let right = *password.peek() == PASSWORD;
                                welcomed.set(right);
                                if !right {
                                    replay.start();
                                }
                            },
                        )),
                ),
            )
            .child(
                rect()
                    .scale(greeting.max(0.0))
                    .opacity(greeting.clamp(0.0, 1.0))
                    .child(text("Bienvenido", HEADLINE, palette.accent)),
            )
    }
}

#[derive(PartialEq)]
pub struct Inbox;

impl Component for Inbox {
    fn render(&self) -> impl IntoElement {
        let reduced_motion = use_reduced_motion();
        let mut count = use_state(|| 0u32);
        let pop = use_animation_transition(count, move |_: u32, _| {
            spring(reduced_motion, 0.4, 1.0, DURATION_SLOW_MS, 0.6)
        });
        let unread = *count.read();
        let palette = Palette::current();
        let size = COLLECTION.icon;
        let corner = -BADGE.height / 2.0;
        horizontal()
            .child(
                rect()
                    .width(Size::px(size))
                    .height(Size::px(size))
                    .corner_radius(COLLECTION.radius)
                    .center()
                    .background(color(palette.control_background))
                    .child(symbol(Symbol::ListBullet, size / 2.0, palette.label))
                    .maybe(unread > 0, |el| {
                        el.child(
                            badge(unread.to_string())
                                .scale(pop.get().current())
                                .position(Position::new_absolute().top(corner).right(corner)),
                        )
                    }),
            )
            .child(Button::new("Recibir").on_press(move |_| *count.write() += 1))
    }
}

#[derive(PartialEq)]
pub struct PlusToggle;

impl Component for PlusToggle {
    fn render(&self) -> impl IntoElement {
        let reduced_motion = use_reduced_motion();
        let mut open = use_state(|| false);
        let turn = use_animation_transition(open, move |from, to| {
            spring(reduced_motion, unit(from), unit(to), DURATION_BASE, 0.4)
        });
        let progress = turn.get().current();
        let palette = Palette::current();
        let size = COLLECTION.icon;
        rect()
            .width(Size::px(size))
            .height(Size::px(size))
            .corner_radius(size / 2.0)
            .center()
            .background(color(
                palette
                    .accent
                    .mix(palette.destructive, progress.clamp(0.0, 1.0)),
            ))
            .rotate(45.0 * progress)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt("Abrir")
            .on_press(move |_| open.toggle())
            .child(symbol(Symbol::Plus, size / 2.0, palette.on_accent))
    }
}

#[derive(PartialEq)]
struct Arrival {
    id: u32,
    leaving: bool,
}

impl Component for Arrival {
    fn render(&self) -> impl IntoElement {
        let reduced_motion = use_reduced_motion();
        let presence = use_animation_with_dependencies(&self.leaving, move |conf, leaving| {
            conf.on_creation(OnCreation::Run);
            conf.on_change(OnChange::Rerun);
            let (from, to) = if *leaving { (1.0, 0.0) } else { (0.0, 1.0) };
            AnimNum::new(from, to)
                .time(time(reduced_motion, DURATION_BASE))
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        let progress = presence.get().value();
        let palette = Palette::current();
        horizontal()
            .width(Size::fill())
            .height(Size::px(TABLE.row_height * progress))
            .opacity(progress)
            .overflow(Overflow::Clip)
            .offset_x(SLIDE_IN * (1.0 - progress))
            .child(text(format!("Elemento {}", self.id), TEXT, palette.label))
    }
}

#[derive(PartialEq)]
pub struct Arrivals;

impl Component for Arrivals {
    fn render(&self) -> impl IntoElement {
        let reduced_motion = use_reduced_motion();
        let mut items = use_state(|| vec![1u32, 2]);
        let mut next = use_state(|| 3u32);
        let leaving = use_state(Vec::<u32>::new);
        let alive = use_hook(|| Rc::new(Cell::new(true)));
        use_drop({
            let alive = alive.clone();
            move || alive.set(false)
        });
        let leaving_now = leaving.read().clone();
        let removable = items
            .read()
            .iter()
            .rev()
            .find(|&&id| !leaving_now.contains(&id))
            .copied();
        let rows = items
            .read()
            .iter()
            .map(|&id| {
                rect()
                    .key(id)
                    .child(Arrival {
                        id,
                        leaving: leaving_now.contains(&id),
                    })
                    .into_element()
            })
            .collect::<Vec<_>>();
        rect()
            .spacing(SPACE_2)
            .child(
                horizontal()
                    .child(Button::new("Añadir").on_press(move |_| {
                        let id = *next.peek();
                        next.set(id + 1);
                        items.write().push(id);
                    }))
                    .child(
                        Button::new("Quitar")
                            .disabled(removable.is_none())
                            .on_press(move |_| {
                                let (Some(id), Some(timers)) =
                                    (removable, try_consume_root_context::<Timers>())
                                else {
                                    return;
                                };
                                let mut leaving = leaving;
                                leaving.write().push(id);
                                let delay = time(reduced_motion, DURATION_BASE);
                                let alive = alive.clone();
                                timers.after(Duration::from_millis(delay), move || {
                                    if !alive.get() {
                                        return;
                                    }
                                    let (mut items, mut leaving) = (items, leaving);
                                    items.write().retain(|item| *item != id);
                                    leaving.write().retain(|item| *item != id);
                                });
                            }),
                    ),
            )
            .child(rect().width(Size::px(SPLIT_VIEW.inspector)).children(rows))
    }
}

#[derive(PartialEq)]
pub struct Popover;

impl Component for Popover {
    fn render(&self) -> impl IntoElement {
        let reduced_motion = use_reduced_motion();
        let mut shown = use_state(|| false);
        let reveal = use_animation_transition(shown, move |from, to| {
            let bounce = if to { 0.3 } else { 0.0 };
            spring(
                reduced_motion,
                unit(from),
                unit(to),
                DURATION_SLOW_MS,
                bounce,
            )
        });
        let progress = reveal.get().current();
        let label = if *shown.read() { "Ocultar" } else { "Mostrar" };
        let palette = Palette::current();
        rect()
            .spacing(SPACE_2)
            .child(Button::new(label).on_press(move |_| shown.toggle()))
            .child(
                card(rect(), palette.control_background)
                    .scale(0.9 + 0.1 * progress)
                    .opacity(progress.clamp(0.0, 1.0))
                    .child(text("Llega con resorte", TEXT, palette.label)),
            )
    }
}

#[derive(PartialEq)]
pub struct CopyToast;

impl Component for CopyToast {
    fn render(&self) -> impl IntoElement {
        let reduced_motion = use_reduced_motion();
        let toast = use_animation(move |_| {
            AnimSequential::new([
                AnimNum::new(0.0, 1.0)
                    .time(time(reduced_motion, DURATION_BASE))
                    .ease(Ease::Out)
                    .function(Function::Cubic),
                AnimNum::new(1.0, 1.0).time(TOAST_HOLD_MS),
                AnimNum::new(1.0, 0.0)
                    .time(time(reduced_motion, DURATION_SLOW_MS))
                    .ease(Ease::In)
                    .function(Function::Cubic),
            ])
        });
        let visible = {
            let steps = toast.get();
            steps[0].value() * steps[2].value()
        };
        let mut replay = toast;
        let palette = Palette::current();
        horizontal()
            .child(Button::new("Copiar enlace").on_press(move |_| replay.start()))
            .child(
                rect()
                    .padding(SPACE_2)
                    .corner_radius(COLLECTION.radius)
                    .background(color(palette.label))
                    .opacity(visible)
                    .scale(0.94 + 0.06 * visible)
                    .child(text("Enlace copiado", CALLOUT, palette.content_background)),
            )
    }
}

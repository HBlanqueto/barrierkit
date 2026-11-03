use barrier_kit::compact::IconButton;
use barrier_kit::style::{Palette, text};
use barrier_kit::symbol::{Symbol, symbol};
use barrier_kit::theme::color;
use freya_animation::prelude::{AnimNum, Ease, Function, use_animation_transition};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Position, Size};
use vow_appearance::backdrop::ink_on;
use vow_design::{
    ACCENT_SWATCHES, CAPTION, COLLECTION, DURATION_BASE, DURATION_SLOW_MS, HEADLINE, SPACE_2,
    SPLIT_VIEW,
};

use crate::motion::{spring, time, unit, use_reduced_motion};

pub const SLIDES: [(&str, &str, Symbol); 4] = [
    (
        "Bienvenida",
        "Todo lo que se mueve en el kit.",
        Symbol::Play,
    ),
    (
        "Curvas",
        "Cada gesto con su ritmo.",
        Symbol::SlidersVertical,
    ),
    (
        "Resorte",
        "Grade Animation sobre Freya.",
        Symbol::ChevronUpDown,
    ),
    ("Listo", "Úsalo en tu app.", Symbol::SquareGrid),
];

pub const TRANSITIONS: [&str; 5] = ["Desplazar", "Fundido", "Zoom", "Cubrir", "Resorte"];

const BOUNCY: usize = 4;
const WIDTH: f32 = SPLIT_VIEW.sidebar_max;
const HEIGHT: f32 = SPLIT_VIEW.inspector * 0.75;
const DOT: f32 = 6.0;
const INCOMING_LAYER: i16 = 2;

fn slide(index: usize) -> Rect {
    let (title, caption, icon) = SLIDES[index];
    let fill = ACCENT_SWATCHES[index % ACCENT_SWATCHES.len()].1;
    let ink = ink_on(fill);
    rect()
        .width(Size::px(WIDTH))
        .height(Size::px(HEIGHT))
        .center()
        .spacing(SPACE_2)
        .background(color(fill))
        .child(symbol(icon, COLLECTION.icon, ink))
        .child(text(title, HEADLINE, ink))
        .child(text(caption, CAPTION, ink))
}

fn at(layer: Rect, left: f32) -> Rect {
    layer.position(Position::new_absolute().left(left).top(0.0))
}

#[derive(PartialEq)]
pub struct Stage {
    pub page: usize,
    pub transition: usize,
}

impl Component for Stage {
    fn render(&self) -> impl IntoElement {
        let reduced_motion = use_reduced_motion();
        let page = use_reactive(&self.page);
        let transition = use_reactive(&self.transition);
        let pages: State<(usize, usize)> = use_previous_and_current(page);
        let animation = use_animation_transition(page, move |_: usize, _| {
            let bounce = if *transition.peek() == BOUNCY {
                0.3
            } else {
                0.0
            };
            spring(reduced_motion, 0.0, 1.0, DURATION_SLOW_MS, bounce)
        });
        let (from, to) = *pages.read();
        let progress = animation.get().current();
        let shown = progress.min(1.0);
        let direction = if to >= from { 1.0 } else { -1.0 };
        let stage = rect()
            .width(Size::px(WIDTH))
            .height(Size::px(HEIGHT))
            .corner_radius(COLLECTION.radius)
            .overflow(Overflow::Clip);
        if from == to || progress >= 1.0 {
            return stage.child(at(slide(to), 0.0));
        }
        let (outgoing, incoming) = match self.transition {
            1 => (
                at(slide(from), 0.0).opacity(1.0 - shown),
                at(slide(to), 0.0).opacity(shown),
            ),
            2 => (
                at(slide(from), 0.0)
                    .scale(1.0 + 0.1 * shown)
                    .opacity(1.0 - shown),
                at(slide(to), 0.0).scale(0.9 + 0.1 * shown).opacity(shown),
            ),
            3 => (
                at(slide(from), 0.0)
                    .scale(1.0 - 0.08 * shown)
                    .opacity(1.0 - 0.5 * shown),
                at(slide(to), direction * WIDTH * (1.0 - progress)),
            ),
            _ => (
                at(slide(from), -direction * WIDTH * progress),
                at(slide(to), direction * WIDTH * (1.0 - progress)),
            ),
        };
        // Freya 0.4.3 paints by tree depth: lift the incoming slide above the outgoing slide's children.
        stage.child(outgoing).child(incoming.layer(INCOMING_LAYER))
    }
}

#[derive(PartialEq)]
struct Dot {
    active: bool,
}

impl Component for Dot {
    fn render(&self) -> impl IntoElement {
        let reduced_motion = use_reduced_motion();
        let active = use_reactive(&self.active);
        let animation = use_animation_transition(active, move |from, to| {
            AnimNum::new(unit(from), unit(to))
                .time(time(reduced_motion, DURATION_BASE))
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        let progress = animation.get().value();
        let palette = Palette::current();
        rect()
            .width(Size::px(DOT * (1.0 + 2.0 * progress)))
            .height(Size::px(DOT))
            .corner_radius(DOT / 2.0)
            .background(color(palette.tertiary_label.mix(palette.accent, progress)))
    }
}

pub fn pager(mut page: State<usize>) -> Rect {
    let current = *page.read();
    let last = SLIDES.len() - 1;
    rect()
        .direction(Direction::Horizontal)
        .cross_align(Alignment::Center)
        .spacing(SPACE_2)
        .child(
            IconButton::new(Symbol::ChevronLeft, "Anterior")
                .disabled(current == 0)
                .on_press(move |_| page.set(current.saturating_sub(1))),
        )
        .children((0..SLIDES.len()).map(|index| {
            Dot {
                active: index == current,
            }
            .into_element()
        }))
        .child(
            IconButton::new(Symbol::ChevronRight, "Siguiente")
                .disabled(current == last)
                .on_press(move |_| page.set((current + 1).min(last))),
        )
}

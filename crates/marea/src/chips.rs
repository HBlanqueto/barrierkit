use barrier_kit::style::Palette;
use barrier_kit::theme::color;
use freya_animation::prelude::{AnimNum, Ease, Function, use_animation_transition};
use freya_core::prelude::*;
use torin::prelude::{Direction, Gaps, Position, Size};
use vow_design::{DURATION_SLOW_MS, MEDIA, Rgba, TEXT};

use crate::catalog::GENRES;

const HEIGHT: f32 = 28.0;
const PADDING_X: f32 = 14.0;
const GAP: f32 = 8.0;
const INSET: f32 = 40.0;

#[derive(PartialEq)]
pub struct Chips {
    pub selected: usize,
    pub on_select: Callback<usize, ()>,
}

impl Component for Chips {
    fn render(&self) -> impl IntoElement {
        let mut origin = use_state(|| 0.0f32);
        let mut spans = use_state(|| [(0.0f32, 0.0f32); GENRES.len()]);
        let selected = use_reactive(&self.selected);
        let slide = use_animation_transition(selected, |from: usize, to: usize| {
            AnimNum::new(from as f32, to as f32)
                .time(DURATION_SLOW_MS)
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        let at = slide.get().value();
        let spans_now = *spans.read();
        let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
        let low = (at.floor() as usize).min(GENRES.len() - 1);
        let high = (low + 1).min(GENRES.len() - 1);
        let t = at - low as f32;
        let left = lerp(spans_now[low].0, spans_now[high].0, t) - *origin.read() - INSET;
        let width = lerp(spans_now[low].1, spans_now[high].1, t);
        let palette = Palette::current();
        let pick = self.on_select.clone();

        rect()
            .direction(Direction::Horizontal)
            .spacing(GAP)
            .padding(Gaps::new(0.0, INSET, 20.0, INSET))
            .on_sized(move |event: Event<SizedEventData>| {
                origin.set_if_modified(event.area.min_x())
            })
            .child(
                rect()
                    .position(Position::new_absolute().left(left).top(0.0))
                    .width(Size::px(width))
                    .height(Size::px(HEIGHT))
                    .corner_radius(HEIGHT / 2.0)
                    .background(color(MEDIA.button_fill)),
            )
            .children(GENRES.iter().enumerate().map(|(index, (_, name))| {
                let pick = pick.clone();
                let covered = (1.0 - (at - index as f32).abs()).clamp(0.0, 1.0);
                let (rest, picked) = (palette.secondary_label, MEDIA.button_ink);
                let ink = Rgba {
                    a: rest.a + (picked.a - rest.a) * covered,
                    ..rest.mix(picked, covered)
                };
                rect()
                    .height(Size::px(HEIGHT))
                    .padding(Gaps::new(0.0, PADDING_X, 0.0, PADDING_X))
                    .center()
                    .a11y_role(AccessibilityRole::Button)
                    .a11y_alt(*name)
                    .on_sized(move |event: Event<SizedEventData>| {
                        let span = (event.area.min_x(), event.area.width());
                        if spans.peek()[index] != span {
                            spans.write()[index] = span;
                        }
                    })
                    .on_press(move |_| pick.call(index))
                    .child(
                        label()
                            .text(*name)
                            .font_family(TEXT.family)
                            .font_size(12.0)
                            .font_weight(500)
                            .color(color(ink)),
                    )
                    .into_element()
            }))
    }
}

use std::time::Duration;

use barrier_kit::media::{MediaButton, RoundButton, ScrimEdge, ground, scrim};
use barrier_kit::style::{Palette, text};
use barrier_kit::symbol::{Symbol, symbol};
use barrier_kit::theme::color;
use barrier_kit::video::{VideoStream, VideoView};
use freya_animation::prelude::{
    AnimNum, Ease, Function, OnChange, OnCreation, use_animation, use_animation_transition,
    use_animation_with_dependencies,
};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Area, Direction, Gaps, Position, Size};
use vow_design::{CALLOUT, CAPTION, DURATION_BASE, DURATION_SLOW_MS, Rgba, SPACE_2, TEXT};

use crate::art::{art_fill, logo};
use crate::assets;
use crate::catalog::{FEATURED, title};

pub const HERO_HEIGHT: f32 = 580.0;
pub const AUTOPLAY: Duration = Duration::from_millis(8000);
const PARALLAX: f32 = 0.45;
const DIM_AFTER: f32 = 420.0;
const SIDE_SCRIM: f32 = 0.62;
const BOTTOM_SCRIM: f32 = 280.0;
const DOT: f32 = 6.0;
// Freya paints by tree depth and the video sits inside a component: lift what goes over it.
const OVER_VIDEO: i16 = 4;
const ACTIVE_DOT: f32 = 26.0;

#[derive(PartialEq)]
pub struct HeroBackground {
    pub id: &'static str,
    pub scrolled: f32,
    pub width: f32,
}

impl Component for HeroBackground {
    fn render(&self) -> impl IntoElement {
        let stream = use_hook({
            let id = self.id;
            move || {
                let stream = VideoStream::open(assets::video(id), true).ok();
                if let Some(stream) = &stream {
                    stream.play();
                }
                stream
            }
        });
        let arrive = use_animation(|conf| {
            conf.on_creation(OnCreation::Run);
            AnimNum::new(1.0, 0.0)
                .time(DURATION_SLOW_MS + DURATION_BASE)
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        let veil = arrive.get().value();
        let dim = 0.6 * (self.scrolled / DIM_AFTER).min(1.0);
        let window = ground();
        let hero = rect()
            .position(
                Position::new_absolute()
                    .top(-self.scrolled * PARALLAX)
                    .left(0.0),
            )
            .width(Size::fill())
            .height(Size::px(HERO_HEIGHT))
            .background(art_fill(title(self.id)))
            .maybe_child(stream.map(VideoView::new))
            .child(
                scrim(ScrimEdge::Leading)
                    .layer(OVER_VIDEO)
                    .position(Position::new_absolute().top(0.0).left(0.0))
                    .width(Size::px(self.width * SIDE_SCRIM))
                    .height(Size::percent(100.0)),
            )
            .child(
                scrim(ScrimEdge::Bottom)
                    .layer(OVER_VIDEO)
                    .position(Position::new_absolute().bottom(0.0).left(0.0))
                    .width(Size::fill())
                    .height(Size::px(BOTTOM_SCRIM)),
            )
            .child(
                rect()
                    .layer(OVER_VIDEO + 1)
                    .position(Position::new_absolute().top(0.0).left(0.0))
                    .width(Size::fill())
                    .height(Size::percent(100.0))
                    .background(color(Rgba {
                        a: (veil * 0.8 + dim).min(1.0),
                        ..window
                    })),
            );
        // The veil sits on a higher layer than the scroll view, so it would catch its wheel.
        rect()
            .width(Size::fill())
            .height(Size::percent(100.0))
            .background(color(window))
            .interactive(Interactive::No)
            .child(hero)
    }
}

#[derive(PartialEq)]
struct Dot {
    active: bool,
    progress: f32,
    on_press: Callback<(), ()>,
}

impl Component for Dot {
    fn render(&self) -> impl IntoElement {
        let active = use_reactive(&self.active);
        let grow = use_animation_transition(active, |from: bool, to: bool| {
            AnimNum::new(f32::from(u8::from(from)), f32::from(u8::from(to)))
                .time(DURATION_BASE)
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        let grown = grow.get().value();
        let width = DOT + (ACTIVE_DOT - DOT) * grown;
        let on_press = self.on_press.clone();
        rect()
            .width(Size::px(width))
            .height(Size::px(DOT))
            .corner_radius(DOT / 2.0)
            .overflow(Overflow::Clip)
            .background(color(Rgba::rgba8(255, 255, 255, 0.33 - 0.08 * grown)))
            .a11y_role(AccessibilityRole::Button)
            .on_press(move |_| on_press.call(()))
            .maybe_child(self.active.then(|| {
                rect()
                    .width(Size::px(width * self.progress))
                    .height(Size::fill())
                    .background(color(Rgba::rgb8(255, 255, 255)))
            }))
    }
}

fn edge(symbol_kind: Symbol, label: &'static str, on_press: Callback<(), ()>) -> Rect {
    rect()
        .width(Size::px(28.0))
        .height(Size::px(56.0))
        .center()
        .a11y_role(AccessibilityRole::Button)
        .a11y_alt(label)
        .on_press(move |_| on_press.call(()))
        .child(symbol(symbol_kind, 22.0, Rgba::rgba8(255, 255, 255, 0.7)))
}

#[derive(PartialEq)]
pub struct HeroCopy {
    pub featured: usize,
    pub scrolled: f32,
    pub listed: bool,
    pub on_select: Callback<usize, ()>,
    pub on_play: Callback<Area, ()>,
    pub on_list: Callback<(), ()>,
}

impl Component for HeroCopy {
    fn render(&self) -> impl IntoElement {
        let featured = use_reactive(&self.featured);
        let mut hovered = use_state(|| false);
        let mut area = use_state(Area::default);
        let reveal = use_animation_transition(featured, |_: usize, _| {
            AnimNum::new(0.0, 1.0)
                .time(DURATION_SLOW_MS)
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        let paused = *hovered.read();
        let progress =
            use_animation_with_dependencies(&(self.featured, paused), |conf, &(_, paused)| {
                conf.on_creation(OnCreation::Run);
                conf.on_change(OnChange::Rerun);
                AnimNum::new(0.0, if paused { 0.0 } else { 1.0 })
                    .time(if paused {
                        0
                    } else {
                        AUTOPLAY.as_millis() as u64
                    })
                    .function(Function::Linear)
            });
        let advance = self.on_select.clone();
        use_side_effect(move || {
            if progress.get().value() >= 1.0 {
                advance.call((*featured.peek() + 1) % FEATURED.len());
            }
        });
        let shown = reveal.get().value();
        let fade = 1.0 - ease_out((self.scrolled / 260.0).min(1.0));
        let title = title(FEATURED[self.featured]);
        let palette = Palette::current();
        let (on_select, on_play, on_list) = (
            self.on_select.clone(),
            self.on_play.clone(),
            self.on_list.clone(),
        );
        let count = FEATURED.len();
        let current = self.featured;
        let step = move |delta: isize| {
            let on_select = on_select.clone();
            Callback::new(move |()| {
                on_select.call((current as isize + delta).rem_euclid(count as isize) as usize)
            })
        };
        let pick = self.on_select.clone();

        rect()
            .width(Size::fill())
            .height(Size::px(HERO_HEIGHT))
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .on_pointer_enter(move |_| hovered.set(true))
            .on_pointer_leave(move |_| hovered.set(false))
            .child(
                rect()
                    .position(Position::new_absolute().left(40.0).bottom(104.0))
                    .width(Size::px(460.0))
                    .opacity(shown * fade)
                    .offset_y(-10.0 * (1.0 - shown) - self.scrolled * 0.2)
                    .spacing(SPACE_2)
                    .child(text(
                        title.kind.to_uppercase(),
                        CAPTION,
                        palette.secondary_label,
                    ))
                    .child(logo(title, 1.0))
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .spacing(SPACE_2)
                            .cross_align(Alignment::Center)
                            .children(title.meta.iter().enumerate().map(|(index, meta)| {
                                if index == 2 {
                                    rect()
                                        .padding(Gaps::new(0.0, 4.0, 0.0, 4.0))
                                        .corner_radius(3.0)
                                        .border(
                                            Border::new()
                                                .width(1.0)
                                                .fill(color(Rgba::rgba8(255, 255, 255, 0.33))),
                                        )
                                        .child(text(*meta, CAPTION, palette.secondary_label))
                                        .into_element()
                                } else {
                                    text(*meta, CALLOUT, palette.secondary_label).into_element()
                                }
                            })),
                    )
                    .child(
                        rect().padding(Gaps::new(2.0, 0.0, 10.0, 0.0)).child(
                            label()
                                .text(title.lede)
                                .font_family(TEXT.family)
                                .font_size(14.0)
                                .color(color(Rgba::rgb8(0xd8, 0xd9, 0xdd))),
                        ),
                    )
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .spacing(10.0)
                            .cross_align(Alignment::Center)
                            .child(
                                MediaButton::new("Reproducir")
                                    .symbol(Symbol::Play)
                                    .on_press(move |_| on_play.call(*area.peek())),
                            )
                            .child(
                                RoundButton::new(
                                    if self.listed {
                                        Symbol::Checkmark
                                    } else {
                                        Symbol::Plus
                                    },
                                    "Añadir a Mi lista",
                                )
                                .selected(self.listed)
                                .on_press(move |_| on_list.call(())),
                            ),
                    ),
            )
            .maybe_child(paused.then(|| {
                edge(Symbol::ChevronLeft, "Destacado anterior", step(-1))
                    .position(Position::new_absolute().left(8.0).top(240.0))
            }))
            .maybe_child(paused.then(|| {
                edge(Symbol::ChevronRight, "Destacado siguiente", step(1))
                    .position(Position::new_absolute().right(8.0).top(240.0))
            }))
            .child(
                rect()
                    .position(Position::new_absolute().right(40.0).bottom(112.0))
                    .direction(Direction::Horizontal)
                    .spacing(6.0)
                    .cross_align(Alignment::Center)
                    .opacity(fade)
                    .children((0..count).map(|index| {
                        let pick = pick.clone();
                        Dot {
                            active: index == current,
                            progress: progress.get().value(),
                            on_press: Callback::new(move |()| pick.call(index)),
                        }
                        .into_element()
                    })),
            )
    }
}

pub fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

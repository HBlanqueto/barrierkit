use std::time::Duration;

use barrier_kit::controls::ScrollController;
use barrier_kit::media::{MediaButton, RoundButton, ScrimEdge, ground, scrim, use_artwork};
use barrier_kit::platform::Timers;
use barrier_kit::style::{Palette, text};
use barrier_kit::symbol::{Symbol, symbol};
use barrier_kit::theme::color;
use barrier_kit::video::{VideoStream, VideoView};
use freya_animation::prelude::{
    AnimNum, Ease, Function, OnChange, OnCreation, OnFinish, use_animation,
    use_animation_transition, use_animation_with_dependencies,
};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Area, Direction, Gaps, Position, Size};
use vow_design::{CALLOUT, CAPTION, DURATION_BASE, DURATION_SLOW_MS, Rgba, SPACE_2, TEXT, TITLE_2};

use crate::art::{art_fill, logo};
use crate::assets;
use crate::catalog::{FEATURED, Gallery, gallery, title};
use crate::localize::tr;
use crate::{INSET, WHITE};

pub const HERO_HEIGHT: f32 = 580.0;
pub const AUTOPLAY: Duration = Duration::from_millis(8000);
const PARALLAX: f32 = 0.45;
const DIM_AFTER: f32 = 420.0;
const SIDE_SCRIM: f32 = 0.62;
const BOTTOM_SCRIM: f32 = 280.0;
const DOT: f32 = 6.0;
// Freya paints by layer plus tree depth, and siblings on one layer in no set order: the
// incoming slide sits one layer over the outgoing one, the preview over both, the scrims last.
const OUTGOING_SLIDE: i16 = -2;
const ABOVE_SLIDES: i16 = 1;
const OVER_VIDEO: i16 = 4;
const ACTIVE_DOT: f32 = 26.0;
const POSTER_MS: u64 = 6000;
const STILL_MS: u64 = 5000;
const GALLERY_MS: u64 = POSTER_MS + 3 * STILL_MS;
const CROSSFADE_MS: f32 = 1100.0;
const KEN_BURNS: f32 = 0.06;
const DWELL: Duration = Duration::from_millis(2000);
const PREVIEW: Duration = Duration::from_millis(15000);
const LOGO: (f32, f32) = (360.0, 132.0);

pub fn autoplay(id: &str) -> Duration {
    if gallery(id).is_some() {
        Duration::from_millis(GALLERY_MS)
    } else {
        AUTOPLAY
    }
}

#[derive(PartialEq)]
struct HeroGallery {
    gallery: &'static Gallery,
}

impl Component for HeroGallery {
    fn render(&self) -> impl IntoElement {
        let poster = use_artwork(assets::image(self.gallery.poster));
        let stills = self
            .gallery
            .stills
            .map(|file| use_artwork(assets::image(file)));
        let clock = use_animation(|conf| {
            conf.on_creation(OnCreation::Run);
            conf.on_finish(OnFinish::Restart {
                delay: Duration::ZERO,
            });
            AnimNum::new(0.0, GALLERY_MS as f32)
                .time(GALLERY_MS)
                .function(Function::Linear)
        });
        let elapsed = clock.get().value();
        let first_lap = *clock.runs().read() <= 1;
        let slides = [poster, stills[0], stills[1], stills[2]];
        let starts = [0, POSTER_MS, POSTER_MS + STILL_MS, POSTER_MS + 2 * STILL_MS];
        let current = starts
            .iter()
            .rposition(|&start| elapsed >= start as f32)
            .unwrap_or(0);
        let length = if current == 0 { POSTER_MS } else { STILL_MS };
        let local = elapsed - starts[current] as f32;
        let fade = (local / CROSSFADE_MS).min(1.0);
        let previous = (fade < 1.0 && !(current == 0 && first_lap))
            .then(|| (current + slides.len() - 1) % slides.len());
        let slide = |index: usize, layer: i16, opacity: f32, progress: f32| {
            slides[index].read().clone().map(|handle| {
                rect()
                    .layer(layer)
                    .position(Position::new_absolute().top(0.0).left(0.0))
                    .width(Size::fill())
                    .height(Size::percent(100.0))
                    .opacity(opacity)
                    .scale(1.0 + KEN_BURNS * progress)
                    .child(
                        image(handle)
                            .width(Size::fill())
                            .height(Size::fill())
                            .aspect_ratio(AspectRatio::Max)
                            .image_cover(ImageCover::Center),
                    )
            })
        };
        rect()
            .position(Position::new_absolute().top(0.0).left(0.0))
            .width(Size::fill())
            .height(Size::percent(100.0))
            .overflow(Overflow::Clip)
            .maybe_child(previous.and_then(|index| slide(index, OUTGOING_SLIDE, 1.0, 1.0)))
            .maybe_child(slide(
                current,
                OUTGOING_SLIDE + 1,
                if previous.is_some() { fade } else { 1.0 },
                (local / length as f32).min(1.0),
            ))
    }
}

#[derive(PartialEq)]
struct HeroLogo {
    gallery: &'static Gallery,
}

impl Component for HeroLogo {
    fn render(&self) -> impl IntoElement {
        let logo = use_artwork(assets::image(self.gallery.logo));
        rect()
            .spacing(4.0)
            .maybe_child(logo.read().clone().map(|handle| {
                image(handle)
                    .width(Size::px(LOGO.0))
                    .height(Size::px(LOGO.1))
                    .aspect_ratio(AspectRatio::Min)
            }))
            .child(
                label()
                    .text(self.gallery.franchise.to_uppercase())
                    .font_family(TEXT.family)
                    .font_size(13.0)
                    .font_weight(700)
                    .color(color(WHITE)),
            )
    }
}

#[derive(PartialEq)]
pub struct HeroBackground {
    pub id: &'static str,
    pub scroll: ScrollController,
    pub width: f32,
    pub previewing: bool,
}

impl Component for HeroBackground {
    fn render(&self) -> impl IntoElement {
        let id = self.id;
        let gallery = gallery(id);
        let stream = use_hook(move || {
            let stream = gallery
                .is_none()
                .then(|| assets::open_video(id, true))
                .flatten();
            if let Some(stream) = &stream {
                stream.play();
            }
            stream
        });
        let mut preview = use_state(|| None::<VideoStream>);
        use_side_effect_with_deps(&self.previewing, move |&previewing| {
            let stream = (previewing && gallery.is_some())
                .then(|| assets::open_video(id, false))
                .flatten();
            if let Some(stream) = &stream {
                stream.play();
            }
            preview.set(stream);
        });
        let arrive = use_animation(|conf| {
            conf.on_creation(OnCreation::Run);
            AnimNum::new(1.0, 0.0)
                .time(DURATION_SLOW_MS + DURATION_BASE)
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        let veil = arrive.get().value();
        let scrolled = crate::scrolled(self.scroll);
        let dim = 0.6 * (scrolled / DIM_AFTER).min(1.0);
        let window = ground();
        let hero = rect()
            .position(Position::new_absolute().top(-scrolled * PARALLAX).left(0.0))
            .width(Size::fill())
            .height(Size::px(HERO_HEIGHT))
            .background(art_fill(title(self.id)))
            .maybe_child(gallery.map(|gallery| HeroGallery { gallery }.into_element()))
            .maybe_child(
                stream
                    .or_else(|| preview.read().clone())
                    .map(|stream| rect().layer(ABOVE_SLIDES).child(VideoView::new(stream))),
            )
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
                    .background(color(WHITE))
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
    pub scroll: ScrollController,
    pub listed: bool,
    pub previewing: bool,
    pub on_select: Callback<usize, ()>,
    pub on_play: Callback<Area, ()>,
    pub on_list: Callback<(), ()>,
    pub on_preview: Callback<bool, ()>,
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
            use_animation_with_dependencies(&(self.featured, paused), |conf, &(index, paused)| {
                conf.on_creation(OnCreation::Run);
                conf.on_change(OnChange::Rerun);
                AnimNum::new(0.0, if paused { 0.0 } else { 1.0 })
                    .time(if paused {
                        0
                    } else {
                        autoplay(FEATURED[index]).as_millis() as u64
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
        let scrolled = crate::scrolled(self.scroll);
        let fade = 1.0 - ease_out((scrolled / 260.0).min(1.0));
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
        let featured_gallery = gallery(FEATURED[self.featured]);
        let timers = try_consume_root_context::<Timers>();
        let mut moved = use_state(|| 0u64);
        let previews = use_state(|| 0u64);
        let previewing = self.previewing;
        let on_preview = self.on_preview.clone();
        let dwell = move |_| {
            let next = *moved.peek() + 1;
            moved.set(next);
            let (Some(timers), Some(_), false) = (&timers, featured_gallery, previewing) else {
                return;
            };
            let on_preview = on_preview.clone();
            let ending = timers.clone();
            timers.after(DWELL, move || {
                if *moved.peek() == next && *hovered.peek() {
                    on_preview.call(true);
                    let mut previews = previews;
                    let started = *previews.peek() + 1;
                    previews.set(started);
                    let on_preview = on_preview.clone();
                    ending.after(PREVIEW, move || {
                        if *previews.peek() == started {
                            on_preview.call(false);
                        }
                    });
                }
            });
        };
        let stop_preview = self.on_preview.clone();

        rect()
            .width(Size::fill())
            .height(Size::px(HERO_HEIGHT))
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .on_pointer_enter(move |_| hovered.set(true))
            .on_pointer_move(dwell)
            .on_pointer_leave(move |_| {
                hovered.set(false);
                stop_preview.call(false);
            })
            .child(
                rect()
                    .position(Position::new_absolute().left(INSET).bottom(104.0))
                    .width(Size::px(460.0))
                    .opacity(shown * fade)
                    .offset_y(-10.0 * (1.0 - shown) - scrolled * 0.2)
                    .spacing(SPACE_2)
                    .child(text(
                        title.kind.to_uppercase(),
                        CAPTION,
                        palette.secondary_label,
                    ))
                    .child(match featured_gallery {
                        Some(gallery) => HeroLogo { gallery }.into_element(),
                        None => logo(title, 1.0).into_element(),
                    })
                    .maybe_child(featured_gallery.map(|_| text(title.name, TITLE_2, WHITE)))
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
                                        .child(text(
                                            meta.as_str(),
                                            CAPTION,
                                            palette.secondary_label,
                                        ))
                                        .into_element()
                                } else {
                                    text(meta.as_str(), CALLOUT, palette.secondary_label)
                                        .into_element()
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
                                MediaButton::new(tr!("play"))
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
                                    tr!("add-to-my-list"),
                                )
                                .selected(self.listed)
                                .on_press(move |_| on_list.call(())),
                            ),
                    ),
            )
            .maybe_child(paused.then(|| {
                edge(Symbol::ChevronLeft, tr!("previous-featured"), step(-1))
                    .position(Position::new_absolute().left(8.0).top(240.0))
            }))
            .maybe_child(paused.then(|| {
                edge(Symbol::ChevronRight, tr!("next-featured"), step(1))
                    .position(Position::new_absolute().right(8.0).top(240.0))
            }))
            .child(
                rect()
                    .position(Position::new_absolute().right(INSET).bottom(112.0))
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

use std::rc::Rc;
use std::time::Duration;

use barrier_kit::animation::{Spring, use_alive};
use barrier_kit::controls::{ScrollView, use_scroll_controller};
use barrier_kit::interaction::use_interaction;
use barrier_kit::media::{RoundButton, material, use_artwork_from};
use barrier_kit::platform::Timers;
use barrier_kit::style::{Palette, text};
use barrier_kit::symbol::{Symbol, symbol};
use barrier_kit::theme::color;
use barrier_kit::video::{VideoStream, VideoView};
use freya_animation::prelude::{AnimNum, Ease, Function, use_animation_transition};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Area, Direction, Gaps, Position, Size};
use vow_design::{CALLOUT, CAPTION, DURATION_BASE, DURATION_SLOW_MS, MEDIA, Rgba, TITLE_2};

use crate::art::{art_fill, logo};
use crate::assets;
use crate::catalog::{Entry, Genre, SHELVES, ShelfStyle, title};
use crate::localize::tr;
use crate::{INSET, WHITE};

const WIDE: (f32, f32) = (232.0, 130.5);
const TALL: (f32, f32) = (156.0, 234.0);
const GAP: f32 = 14.0;
const RADIUS: f32 = 10.0;
const LIFT_ROOM: Gaps = Gaps::new(16.0, INSET, 18.0, INSET);
const ABOVE_ART: i16 = 4;
const ABOVE_CARDS: i16 = 24;
const RISE: f32 = 24.0;

fn unit(on: bool) -> f32 {
    f32::from(u8::from(on))
}

#[derive(PartialEq)]
struct Card {
    entry: &'static Entry,
    style: ShelfStyle,
    rank: usize,
    on_open: Callback<(&'static str, Area), ()>,
}

impl Component for Card {
    fn render(&self) -> impl IntoElement {
        let title = title(self.entry.id);
        let interaction = use_interaction();
        let hovered = interaction.hovered();
        let hovering = use_reactive(&hovered);
        let lift = use_animation_transition(hovering, |from, to| {
            Spring::new(
                unit(from),
                unit(to),
                Duration::from_millis(DURATION_SLOW_MS),
                if to { 0.25 } else { 0.0 },
            )
        });
        let mut preview = use_state(|| None::<VideoStream>);
        let mut generation = use_state(|| 0u64);
        let alive = use_alive();
        let id = self.entry.id;
        use_side_effect_with_deps(&hovered, move |&hovered| {
            let next = *generation.peek() + 1;
            generation.set(next);
            if !hovered {
                preview.set(None);
                return;
            }
            let Some(timers) = try_consume_root_context::<Timers>() else {
                return;
            };
            let alive = alive.clone();
            timers.after(Duration::from_millis(MEDIA.preview_delay_ms), move || {
                if alive.get() && *generation.peek() == next {
                    let mut preview = preview;
                    let stream = assets::open_video(id, true);
                    if let Some(stream) = &stream {
                        stream.play();
                    }
                    preview.set(stream);
                }
            });
        });
        let showing = use_reactive(&preview.read().is_some());
        let fade = use_animation_transition(showing, |from, to| {
            AnimNum::new(unit(from), unit(to))
                .time(DURATION_BASE)
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        let still = use_artwork_from(assets::still(self.entry.id));
        let mut area = use_state(Area::default);

        let (width, height) = match self.style {
            ShelfStyle::Wide => WIDE,
            ShelfStyle::Ranked => TALL,
        };
        let scale = 1.0 + (MEDIA.hover_scale - 1.0) * lift.get().current();
        let shown = fade.get().value();
        let palette = Palette::current();
        let on_open = self.on_open.clone();
        let new = self
            .entry
            .caption
            .as_ref()
            .is_some_and(|caption| caption.contains(tr!("new")));

        let art = rect()
            .width(Size::px(width))
            .height(Size::px(height))
            .corner_radius(RADIUS)
            .overflow(Overflow::Clip)
            .background(art_fill(title))
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .maybe_child(
                (self.entry.watched.is_some() || assets::remote(id))
                    .then(|| still.read().clone())
                    .flatten()
                    .map(|handle| {
                        image(handle)
                            .width(Size::fill())
                            .height(Size::fill())
                            .aspect_ratio(AspectRatio::Max)
                            .image_cover(ImageCover::Center)
                    }),
            )
            .maybe_child(preview.read().clone().map(|stream| {
                rect()
                    .layer(ABOVE_ART)
                    .position(Position::new_absolute().top(0.0).left(0.0))
                    .width(Size::fill())
                    .height(Size::percent(100.0))
                    .opacity(shown)
                    .child(VideoView::new(stream))
                    .child(
                        rect()
                            .position(Position::new_absolute().right(8.0).bottom(8.0))
                            .width(Size::px(22.0))
                            .height(Size::px(22.0))
                            .corner_radius(11.0)
                            .center()
                            .background(color(Rgba::rgba8(0, 0, 0, 0.53)))
                            .child(symbol(Symbol::SpeakerWave, 12.0, WHITE)),
                    )
            }))
            .maybe_child((self.style == ShelfStyle::Ranked).then(|| {
                rect()
                    .layer(ABOVE_ART + 4)
                    .position(Position::new_absolute().left(8.0).top(6.0))
                    .child(
                        label()
                            .text(self.rank.to_string())
                            .font_family(vow_design::TEXT.family)
                            .font_weight(800)
                            .font_size(54.0)
                            .color(color(Rgba::rgba8(255, 255, 255, 0.88))),
                    )
            }))
            .maybe_child(new.then(|| {
                rect()
                    .layer(ABOVE_ART + 4)
                    .position(Position::new_absolute().right(8.0).top(8.0))
                    .padding(Gaps::new(2.0, 6.0, 2.0, 6.0))
                    .corner_radius(4.0)
                    .background(color(palette.destructive))
                    .child(text(tr!("new"), CAPTION, WHITE))
            }))
            .child(
                rect()
                    .layer(ABOVE_ART + 4)
                    .position(Position::new_absolute().left(12.0).bottom(10.0))
                    .width(Size::px(width - 24.0))
                    .child(logo(
                        title,
                        if self.style == ShelfStyle::Ranked {
                            0.36
                        } else {
                            0.5
                        },
                    )),
            );

        interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(title.name)
            .width(Size::px(width))
            .on_press(move |_| on_open.call((id, *area.peek())))
            .child(rect().scale(scale).child(art))
            .maybe_child(self.entry.watched.map(|watched| {
                rect()
                    .margin(Gaps::new(8.0, 0.0, 0.0, 0.0))
                    .width(Size::fill())
                    .height(Size::px(3.0))
                    .corner_radius(1.5)
                    .background(color(Rgba::rgba8(255, 255, 255, 0.2)))
                    .child(
                        rect()
                            .width(Size::percent(watched * 100.0))
                            .height(Size::fill())
                            .background(color(WHITE)),
                    )
            }))
            .maybe_child(self.entry.caption.as_deref().map(|caption| {
                rect().margin(Gaps::new(6.0, 0.0, 0.0, 0.0)).child(text(
                    caption,
                    CALLOUT,
                    palette.secondary_label,
                ))
            }))
    }
}

#[derive(PartialEq)]
pub struct ShelfRow {
    pub index: usize,
    pub genre: Option<Genre>,
    pub viewport: f32,
    pub on_open: Callback<(&'static str, Area), ()>,
}

impl Component for ShelfRow {
    fn render(&self) -> impl IntoElement {
        let shelf = &SHELVES[self.index];
        let controller = use_scroll_controller();
        let interaction = use_interaction();
        let mut entered = use_state(|| false);
        let mut top = use_state(|| f32::MAX);
        let mut width = use_state(|| 0.0f32);
        let target = use_state(|| (0.0f32, 0.0f32));
        let glide = use_animation_transition(target, |_, (from, to): (f32, f32)| {
            AnimNum::new(from, to)
                .time(DURATION_SLOW_MS)
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        use_side_effect(move || {
            let x = glide.get().value();
            if *glide.is_running().read() {
                // scroll_to_x reads and writes the scroll state; outside the effect it cannot re-trigger it.
                spawn(async move {
                    let mut controller = controller;
                    controller.scroll_to_x(x as i32);
                });
            }
        });
        let rise = use_animation_transition(entered, |from, to| {
            AnimNum::new(unit(from), unit(to))
                .time(DURATION_SLOW_MS)
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        let risen = rise.get().value();
        use_side_effect_with_deps(&self.viewport, move |&viewport| {
            if viewport > 0.0 && *top.read() < viewport - 40.0 && !*entered.peek() {
                entered.set(true);
            }
        });
        let visible: Vec<(usize, &'static Entry)> = shelf
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                self.genre
                    .is_none_or(|genre| title(entry.id).genre == genre)
            })
            .map(|(rank, entry)| (rank + 1, entry))
            .collect();
        let on_open = self.on_open.clone();
        let style = shelf.style;
        let height = match style {
            ShelfStyle::Wide => WIDE.1 + 44.0,
            ShelfStyle::Ranked => TALL.1,
        } + LIFT_ROOM.top()
            + LIFT_ROOM.bottom();
        let hovered = interaction.hovered();
        let (card_width, card_height) = match style {
            ShelfStyle::Wide => WIDE,
            ShelfStyle::Ranked => TALL,
        };
        let count = visible.len() as f32;
        let content = count * card_width
            + (count - 1.0).max(0.0) * GAP
            + LIFT_ROOM.left()
            + LIFT_ROOM.right();
        let scrolled = -(Into::<(i32, i32)>::into(controller).0 as f32);
        let rest = content - *width.read() - scrolled;
        let page = Rc::new(move |direction: f32| {
            let mut target = target;
            let (x, _): (i32, i32) = controller.into();
            let from = x as f32;
            let to = (from - direction * *width.peek() * 0.8)
                .clamp(-(content - *width.peek()).max(0.0), 0.0);
            target.set((from, to));
        });
        let arrow = |direction: f32| {
            let page = page.clone();
            let (symbol_kind, label, side) = if direction < 0.0 {
                (
                    Symbol::ChevronLeft,
                    tr!("previous"),
                    Position::new_absolute().left(INSET - MEDIA.round / 2.0),
                )
            } else {
                (
                    Symbol::ChevronRight,
                    tr!("next"),
                    Position::new_absolute().right(INSET - MEDIA.round / 2.0),
                )
            };
            rect()
                .layer(ABOVE_CARDS)
                .position(side.top(38.0 + LIFT_ROOM.top() + (card_height - MEDIA.round) / 2.0))
                .width(Size::px(MEDIA.round))
                .height(Size::px(MEDIA.round))
                .child(material(MEDIA.round / 2.0))
                .child(RoundButton::new(symbol_kind, label).on_press(move |_| page(direction)))
        };

        interaction
            .attach(rect(), true)
            .width(Size::fill())
            .padding(Gaps::new(0.0, 0.0, 18.0, 0.0))
            .opacity(risen)
            .offset_y(RISE * (1.0 - risen))
            .on_sized(move |event: Event<SizedEventData>| {
                width.set_if_modified(event.area.width());
                top.set_if_modified(event.area.min_y());
            })
            .child(
                rect()
                    .padding(Gaps::new(0.0, 0.0, 0.0, INSET))
                    .direction(Direction::Horizontal)
                    .cross_align(Alignment::Center)
                    .spacing(4.0)
                    .child(text(shelf.name, TITLE_2, Palette::current().label))
                    .child(symbol(Symbol::ChevronRight, 12.0, Palette::current().label)),
            )
            .child(
                rect().height(Size::px(height)).child(
                    ScrollView::new()
                        .direction(Direction::Horizontal)
                        .scroll_bar(false)
                        .controller(controller)
                        .height(Size::px(height))
                        .child(
                            rect()
                                .direction(Direction::Horizontal)
                                .spacing(GAP)
                                .padding(LIFT_ROOM)
                                .children(visible.into_iter().map(|(rank, entry)| {
                                    Card {
                                        entry,
                                        style,
                                        rank,
                                        on_open: on_open.clone(),
                                    }
                                    .into_element()
                                })),
                        ),
                ),
            )
            .maybe_child((hovered && scrolled > 1.0).then(|| arrow(-1.0)))
            .maybe_child((hovered && rest > 1.0).then(|| arrow(1.0)))
    }
}

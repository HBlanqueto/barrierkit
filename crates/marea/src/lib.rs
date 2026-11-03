//! Marea: a media app that pushes BarrierKit: video under glass, artwork shelves and a player.

mod art;
pub mod assets;
mod catalog;
mod chips;
mod hero;
mod player;
mod shelf;

use std::collections::HashSet;

use barrier_kit::backdrop::{Accent, Backdrop, BackdropSource};
use barrier_kit::controls::{ScrollController, ScrollView, SearchField, use_scroll_controller};
use barrier_kit::media::{ground, material, top_glass};
use barrier_kit::sidebar::{Sidebar, SidebarAccount, SidebarBackdrop, SidebarItem, SidebarSection};
use barrier_kit::style::{Palette, text};
use barrier_kit::symbol::Symbol;
use barrier_kit::theme::{color, use_appearance};
use barrier_kit::titlebar::window_drag_handler;
use freya_core::prelude::*;
use torin::prelude::{Area, Gaps, Position, Size};
use vow_appearance::ColorScheme;
use vow_design::{ACCENT_SWATCHES, Rgba, SIDEBAR, TITLE_2};

use catalog::{FEATURED, GENRES, SHELVES, title};
use chips::Chips;
use hero::{HeroBackground, HeroCopy, ease_out};
use player::Player;
use shelf::ShelfRow;

const ISLAND_INSET: f32 = 8.0;
const ISLAND_LAYER: i16 = 64;
const TOOLBAR_LAYER: i16 = 48;
const TOOLBAR: f32 = 64.0;
const TOOLBAR_FROM: f32 = 300.0;
const TOOLBAR_SPAN: f32 = 120.0;

const NIGHT: Rgba = Rgba::rgb8(0x0b, 0x0c, 0x0e);

fn tones(id: &str) -> [Rgba; 3] {
    let (_, from, to) = title(id).art;
    [
        from.mix(NIGHT, 0.72),
        to.mix(NIGHT, 0.6),
        from.mix(to, 0.5).mix(NIGHT, 0.8),
    ]
}

fn swatch(index: usize) -> Rgba {
    ACCENT_SWATCHES[index % ACCENT_SWATCHES.len()].1
}

#[derive(PartialEq)]
struct Island {
    height: f32,
}

impl Component for Island {
    fn render(&self) -> impl IntoElement {
        let query = use_state(String::new);
        let mut section = use_state(|| 0usize);
        let item = |index: usize, name: &'static str, icon: Symbol| {
            SidebarItem::new(name)
                .symbol(icon, swatch(index))
                .selected(*section.read() == index)
                .on_select(move |_| section.set(index))
        };
        rect()
            .position(
                Position::new_absolute()
                    .left(ISLAND_INSET)
                    .top(ISLAND_INSET)
                    .bottom(ISLAND_INSET),
            )
            .width(Size::px(SIDEBAR.width as f32))
            .height(Size::px((self.height - 2.0 * ISLAND_INSET).max(0.0)))
            .layer(ISLAND_LAYER)
            .child(material(SIDEBAR.radius as f32))
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::fill())
                    .layer(1)
                    .child(
                        Sidebar::new()
                            .traffic_lights(true)
                            .backdrop(SidebarBackdrop::Clear)
                            .child(
                                rect()
                                    .padding(Gaps::new(0.0, 0.0, 10.0, 0.0))
                                    .child(SearchField::new(query, "Buscar")),
                            )
                            .child(item(0, "Inicio", Symbol::House))
                            .child(item(1, "Series", Symbol::Tv))
                            .child(item(2, "Películas", Symbol::Film))
                            .child(item(3, "Estrenos", Symbol::Star))
                            .child(
                                SidebarSection::new("Biblioteca")
                                    .child(item(4, "Seguir viendo", Symbol::Clock))
                                    .child(item(5, "Mi lista", Symbol::Bookmark))
                                    .child(item(6, "Descargas", Symbol::ArrowDown)),
                            )
                            .child(SidebarAccount::new("HB", "H. Blanqueto", "Plan familiar")),
                    ),
            )
    }
}

#[derive(PartialEq)]
struct Toolbar {
    shown: f32,
    scroll: ScrollController,
}

impl Component for Toolbar {
    fn render(&self) -> impl IntoElement {
        let window = ground();
        let scroll = self.scroll;
        rect()
            .position(Position::new_absolute().top(0.0).left(0.0))
            .width(Size::fill())
            .height(Size::px(TOOLBAR))
            .layer(TOOLBAR_LAYER)
            .on_mouse_down(window_drag_handler(false))
            .on_wheel(move |event: Event<WheelEventData>| scroll.scroll_by_wheel(&event))
            .maybe_child((self.shown > 0.0).then(top_glass))
            .child(
                rect()
                    .position(Position::new_absolute().top(0.0).left(0.0))
                    .width(Size::fill())
                    .height(Size::percent(100.0))
                    .background(
                        LinearGradient::new()
                            .stop((
                                color(Rgba {
                                    a: 0.9 * self.shown,
                                    ..window
                                }),
                                0.0,
                            ))
                            .stop((color(Rgba { a: 0.0, ..window }), 100.0)),
                    ),
            )
            .child(
                rect()
                    .position(
                        Position::new_absolute()
                            .top(20.0 + 6.0 * (1.0 - self.shown))
                            .left(248.0),
                    )
                    .opacity(self.shown)
                    .child(text("Inicio", TITLE_2, Palette::current().label)),
            )
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Playing {
    id: &'static str,
    from: Area,
}

pub struct Marea;

impl App for Marea {
    fn render(&self) -> impl IntoElement {
        use_appearance(|config| config.color_scheme = ColorScheme::Dark);
        let mut featured = use_state(|| 0usize);
        let mut genre = use_state(|| 0usize);
        let mut listed = use_state(HashSet::<&'static str>::new);
        let mut playing = use_state(|| None::<Playing>);
        let mut window = use_state(Area::default);
        let controller = use_scroll_controller();
        let (_, offset): (i32, i32) = controller.into();
        let scrolled = (-offset).max(0) as f32;
        let toolbar = ease_out(((scrolled - TOOLBAR_FROM) / TOOLBAR_SPAN).clamp(0.0, 1.0));
        let current = FEATURED[*featured.read()];
        let width = window.read().width();
        let height = window.read().height();
        let open = Callback::new(move |(id, from): (&'static str, Area)| {
            playing.set(Some(Playing { id, from }))
        });

        let content = rect()
            .width(Size::fill())
            .height(Size::fill())
            .on_sized(move |event: Event<SizedEventData>| window.set_if_modified(event.area))
            .child(
                rect()
                    .key(current)
                    .position(Position::new_absolute().top(0.0).left(0.0))
                    .width(Size::fill())
                    .height(Size::percent(100.0))
                    .child(HeroBackground {
                        id: current,
                        scrolled,
                        width,
                    }),
            )
            .child(
                ScrollView::new().controller(controller).child(
                    rect()
                        .width(Size::fill())
                        .padding(Gaps::new(
                            0.0,
                            0.0,
                            24.0,
                            SIDEBAR.width as f32 + 2.0 * ISLAND_INSET + 4.0,
                        ))
                        .child(HeroCopy {
                            featured: *featured.read(),
                            scrolled,
                            listed: listed.read().contains(current),
                            on_select: Callback::new(move |index| featured.set(index)),
                            on_play: Callback::new(move |from| {
                                playing.set(Some(Playing {
                                    id: FEATURED[*featured.peek()],
                                    from,
                                }))
                            }),
                            on_list: Callback::new(move |()| {
                                let id = FEATURED[*featured.peek()];
                                let mut listed = listed.write();
                                if !listed.remove(id) {
                                    listed.insert(id);
                                }
                            }),
                        })
                        .child(Chips {
                            selected: *genre.read(),
                            on_select: Callback::new(move |index| genre.set(index)),
                        })
                        .children((0..SHELVES.len()).map(|index| {
                            ShelfRow {
                                index,
                                genre: GENRES[*genre.read()].0,
                                viewport: height,
                                on_open: open.clone(),
                            }
                            .into_element()
                        })),
                ),
            )
            .child(Toolbar {
                shown: toolbar,
                scroll: controller,
            })
            .child(Island { height })
            .maybe_child(playing.read().as_ref().map(|now| Player {
                id: now.id,
                from: now.from,
                window: *window.read(),
                on_close: Callback::new(move |()| playing.set(None)),
            }));

        Backdrop::new(BackdropSource::Colors(tones(current)), content).accent(Accent::FromBackdrop)
    }
}

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use barrier_kit::animation::Spring;
use barrier_kit::level::LevelSlider;
use barrier_kit::media::RoundButton;
use barrier_kit::platform::{Timers, WindowControls};
use barrier_kit::progress::ProgressRing;
use barrier_kit::style::text;
use barrier_kit::symbol::Symbol;
use barrier_kit::theme::color;
use barrier_kit::video::VideoView;
use freya_animation::prelude::{AnimDirection, OnCreation, use_animation};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Area, Content, Direction, Gaps, Position, Size};
use vow_design::{CALLOUT, CAPTION, DURATION_SLOW_MS, HEADLINE, Rgba, SPACE_2, TEXT};

use crate::art::art_fill;
use crate::assets;
use crate::catalog::title;

const CHROME_IDLE: Duration = Duration::from_millis(2500);
const TICK: Duration = Duration::from_millis(250);
const SEEK_STEP: f64 = 10.0;
const NEXT_UP: f64 = 4.0;
const THUMB: (f32, f32) = (168.0, 95.0);
const SUBTITLES: [&str; 3] = [
    "— ¿Has oído eso?",
    "— Es la radio. Lleva años apagada.",
    "— Entonces, ¿quién está hablando?",
];
const WHITE: Rgba = Rgba::rgb8(255, 255, 255);

fn clock(seconds: f64) -> String {
    let seconds = seconds.max(0.0) as u64;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

fn use_ticker() -> State<u64> {
    let tick = use_state(|| 0u64);
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    use_hook({
        let alive = alive.clone();
        move || {
            fn schedule(timers: Timers, tick: State<u64>, alive: Rc<Cell<bool>>) {
                timers.clone().after(TICK, move || {
                    if alive.get() {
                        let mut tick = tick;
                        *tick.write() += 1;
                        schedule(timers.clone(), tick, alive.clone());
                    }
                });
            }
            if let Some(timers) = try_consume_root_context::<Timers>() {
                schedule(timers, tick, alive);
            }
        }
    });
    use_drop(move || alive.set(false));
    tick
}

#[derive(PartialEq)]
pub struct Player {
    pub id: &'static str,
    pub from: Area,
    pub window: Area,
    pub on_close: Callback<(), ()>,
}

impl Component for Player {
    fn render(&self) -> impl IntoElement {
        let id = self.id;
        let stream = use_hook(move || {
            let stream = assets::open_video(id, false);
            if let Some(stream) = &stream {
                stream.play();
            }
            stream
        });
        let thumb = use_hook(move || assets::open_video(id, false));
        let tick = use_ticker();
        let _ = *tick.read();
        let mut closing = use_state(|| false);
        let mut last_move = use_state(Instant::now);
        let mut subtitles = use_state(|| false);
        let mut volume = use_state(|| 0.7f32);
        let audible = stream.clone();
        use_side_effect(move || {
            if let Some(stream) = &audible {
                stream.set_volume(f64::from(*volume.read()));
            }
        });
        let mut hover = use_state(|| None::<f32>);
        let mut track = use_state(Area::default);
        let open = use_animation(|conf| {
            conf.on_creation(OnCreation::Run);
            Spring::new(0.0, 1.0, Duration::from_millis(DURATION_SLOW_MS + 100), 0.0)
        });
        let on_close = self.on_close.clone();
        use_side_effect(move || {
            if *closing.read() && !*open.is_running().read() {
                on_close.call(());
            }
        });

        let opened = open.get().current();
        let lerp = |a: f32, b: f32| a + (b - a) * opened;
        let (from, to) = (self.from, self.window);
        let frame = (
            lerp(from.min_x() - to.min_x(), 0.0),
            lerp(from.min_y() - to.min_y(), 0.0),
            lerp(from.width(), to.width()),
            lerp(from.height(), to.height()),
        );
        let settled = opened >= 1.0 && !*closing.read();

        let (position, duration, playing) = stream.as_ref().map_or((0.0, 0.0, false), |stream| {
            (stream.position(), stream.duration(), stream.playing())
        });
        let idle = last_move.read().elapsed() > CHROME_IDLE;
        let chrome = settled && !(playing && idle);
        let done = if duration > 0.0 {
            (position / duration) as f32
        } else {
            0.0
        };
        let next_up = duration > 0.0 && duration - position < NEXT_UP;

        let mut close = move || {
            if !*closing.peek() {
                closing.set(true);
                open.run(AnimDirection::Reverse);
            }
        };
        let toggle = {
            let stream = stream.clone();
            move || {
                if let Some(stream) = &stream {
                    if stream.playing() {
                        stream.pause();
                    } else {
                        if stream.duration() - stream.position() < 0.1 {
                            stream.seek(0.0);
                        }
                        stream.play();
                    }
                }
            }
        };
        let seek_by = {
            let stream = stream.clone();
            move |delta: f64| {
                if let Some(stream) = &stream {
                    stream.seek((stream.position() + delta).clamp(0.0, stream.duration()));
                }
            }
        };
        let title = title(self.id);

        let keys = {
            let toggle = toggle.clone();
            let seek_by = seek_by.clone();
            let mut close = close;
            move |event: Event<KeyboardEventData>| {
                last_move.set(Instant::now());
                match &event.key {
                    Key::Named(NamedKey::Escape) => close(),
                    Key::Named(NamedKey::ArrowLeft) => seek_by(-SEEK_STEP),
                    Key::Named(NamedKey::ArrowRight) => seek_by(SEEK_STEP),
                    Key::Character(c) if c == " " => toggle(),
                    Key::Character(c) if c.eq_ignore_ascii_case("c") => subtitles.toggle(),
                    _ => {}
                }
            }
        };

        let scrub_stream = stream.clone();
        let thumb_seek = thumb.clone();
        let line = (*subtitles.read() && playing)
            .then(|| SUBTITLES[(position / 4.0) as usize % SUBTITLES.len()]);

        rect()
            .position(Position::new_absolute().left(frame.0).top(frame.1))
            .width(Size::px(frame.2))
            .height(Size::px(frame.3))
            .corner_radius(lerp(10.0, 0.0))
            .overflow(Overflow::Clip)
            .background(art_fill(title))
            .on_global_key_down(keys)
            .on_pointer_move(move |_| last_move.set(Instant::now()))
            .maybe_child(stream.clone().map(VideoView::new))
            .maybe_child(line.map(|line| {
                rect()
                    .position(Position::new_absolute().bottom(120.0).left(0.0))
                    .width(Size::fill())
                    .center()
                    .child(
                        rect()
                            .padding(Gaps::new(4.0, 10.0, 4.0, 10.0))
                            .corner_radius(6.0)
                            .background(color(Rgba::rgba8(0, 0, 0, 0.66)))
                            .child(
                                label()
                                    .text(line)
                                    .font_family(TEXT.family)
                                    .font_size(18.0)
                                    .font_weight(500)
                                    .color(color(WHITE)),
                            ),
                    )
            }))
            .maybe_child(chrome.then(|| {
                rect()
                    .position(Position::new_absolute().top(0.0).left(0.0))
                    .width(Size::fill())
                    .padding(Gaps::new(18.0, 20.0, 60.0, 20.0))
                    .direction(Direction::Horizontal)
                    .cross_align(Alignment::Center)
                    .spacing(12.0)
                    .background(
                        LinearGradient::new()
                            .stop((color(Rgba::rgba8(0, 0, 0, 0.66)), 0.0))
                            .stop((color(Rgba::rgba8(0, 0, 0, 0.0)), 100.0)),
                    )
                    .child(
                        RoundButton::new(Symbol::ChevronLeft, "Cerrar el reproductor")
                            .on_press(move |_| close()),
                    )
                    .child(rect().child(text(title.name, HEADLINE, WHITE)).child(text(
                        if title.series {
                            "Temporada 1 · Episodio 1"
                        } else {
                            title.meta[0]
                        },
                        CALLOUT,
                        Rgba::rgba8(255, 255, 255, 0.7),
                    )))
            }))
            .maybe_child((chrome && next_up && title.series).then(|| {
                rect()
                    .position(Position::new_absolute().right(24.0).bottom(110.0))
                    .width(Size::px(260.0))
                    .padding(Gaps::new_all(10.0))
                    .corner_radius(12.0)
                    .content(Content::Flex)
                    .direction(Direction::Horizontal)
                    .cross_align(Alignment::Center)
                    .spacing(10.0)
                    .background(color(Rgba::rgba8(0x1c, 0x1d, 0x20, 0.91)))
                    .child(
                        rect()
                            .width(Size::px(90.0))
                            .height(Size::px(51.0))
                            .corner_radius(6.0)
                            .background(art_fill(title)),
                    )
                    .child(
                        rect()
                            .width(Size::flex(1.0))
                            .child(text(
                                "Siguiente episodio",
                                CAPTION,
                                Rgba::rgba8(255, 255, 255, 0.6),
                            ))
                            .child(text("Episodio 2", HEADLINE, WHITE)),
                    )
                    .child(ProgressRing::new(
                        "Siguiente episodio",
                        (1.0 - (duration - position) / NEXT_UP).clamp(0.0, 1.0) as f32,
                    ))
            }))
            .maybe_child(chrome.then(|| {
                let toggle = toggle.clone();
                let back = seek_by.clone();
                let forward = seek_by.clone();
                let hover_at = *hover.read();
                rect()
                    .position(Position::new_absolute().bottom(0.0).left(0.0))
                    .width(Size::fill())
                    .padding(Gaps::new(60.0, 24.0, 18.0, 24.0))
                    .spacing(10.0)
                    .background(
                        LinearGradient::new()
                            .angle(180.0)
                            .stop((color(Rgba::rgba8(0, 0, 0, 0.8)), 0.0))
                            .stop((color(Rgba::rgba8(0, 0, 0, 0.0)), 100.0)),
                    )
                    .child(
                        rect()
                            .width(Size::fill())
                            .height(Size::px(18.0))
                            .cross_align(Alignment::Center)
                            .main_align(Alignment::Center)
                            .a11y_role(AccessibilityRole::Slider)
                            .a11y_alt("Tiempo")
                            .on_sized(move |event: Event<SizedEventData>| {
                                track.set_if_modified(event.area)
                            })
                            .on_pointer_move(move |event: Event<PointerEventData>| {
                                let area = *track.peek();
                                let at = ((event.global_location().x as f32 - area.min_x())
                                    / area.width().max(1.0))
                                .clamp(0.0, 1.0);
                                hover.set(Some(at));
                                if let Some(thumb) = &thumb_seek {
                                    thumb.seek(f64::from(at) * thumb.duration());
                                }
                            })
                            .on_pointer_leave(move |_| hover.set(None))
                            .on_press(move |_| {
                                if let (Some(stream), Some(at)) = (&scrub_stream, *hover.peek()) {
                                    stream.seek(f64::from(at) * stream.duration());
                                }
                            })
                            .child(
                                rect()
                                    .width(Size::fill())
                                    .height(Size::px(if hover_at.is_some() { 7.0 } else { 4.0 }))
                                    .corner_radius(3.5)
                                    .background(color(Rgba::rgba8(255, 255, 255, 0.25)))
                                    .child(
                                        rect()
                                            .width(Size::percent(done * 100.0))
                                            .height(Size::fill())
                                            .corner_radius(3.5)
                                            .background(color(WHITE)),
                                    ),
                            )
                            .maybe_child(hover_at.zip(thumb.clone()).map(|(at, thumb)| {
                                let left = (track.read().width() * at - THUMB.0 / 2.0)
                                    .clamp(0.0, (track.read().width() - THUMB.0).max(0.0));
                                rect()
                                    .position(Position::new_absolute().left(left).bottom(26.0))
                                    .width(Size::px(THUMB.0))
                                    .spacing(4.0)
                                    .cross_align(Alignment::Center)
                                    .child(
                                        rect()
                                            .width(Size::px(THUMB.0))
                                            .height(Size::px(THUMB.1))
                                            .corner_radius(8.0)
                                            .overflow(Overflow::Clip)
                                            .border(
                                                Border::new()
                                                    .width(1.0)
                                                    .fill(color(Rgba::rgba8(255, 255, 255, 0.25))),
                                            )
                                            .child(VideoView::new(thumb).corner_radius(8.0)),
                                    )
                                    .child(text(clock(f64::from(at) * duration), CAPTION, WHITE))
                            })),
                    )
                    .child(
                        rect()
                            .width(Size::fill())
                            .content(Content::Flex)
                            .direction(Direction::Horizontal)
                            .cross_align(Alignment::Center)
                            .spacing(SPACE_2)
                            .child(
                                RoundButton::new(
                                    if playing { Symbol::Pause } else { Symbol::Play },
                                    if playing { "Pausar" } else { "Reproducir" },
                                )
                                .selected(true)
                                .on_press(move |_| toggle()),
                            )
                            .child(
                                RoundButton::new(Symbol::GoBackward, "Retroceder 10 segundos")
                                    .on_press(move |_| back(-SEEK_STEP)),
                            )
                            .child(
                                RoundButton::new(Symbol::GoForward, "Avanzar 10 segundos")
                                    .on_press(move |_| forward(SEEK_STEP)),
                            )
                            .child(text(
                                format!("{} / {}", clock(position), clock(duration)),
                                CALLOUT,
                                Rgba::rgba8(255, 255, 255, 0.8),
                            ))
                            .child(rect().width(Size::flex(1.0)))
                            .child(
                                rect().width(Size::px(110.0)).child(
                                    LevelSlider::new("Volumen")
                                        .value(*volume.read())
                                        .on_change(move |value: f32| volume.set(value)),
                                ),
                            )
                            .child(
                                RoundButton::new(Symbol::Captions, "Subtítulos")
                                    .selected(*subtitles.read())
                                    .on_press(move |_| subtitles.toggle()),
                            )
                            .child(
                                RoundButton::new(Symbol::Expand, "Pantalla completa")
                                    .on_press(move |_| WindowControls::get().toggle_fullscreen()),
                            ),
                    )
            }))
            .layer(Layer::Overlay)
    }
}

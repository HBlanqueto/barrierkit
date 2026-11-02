//! The four panes, the queue and the navigation, drawn with the palette the
//! backdrop provides.

use barrier_kit::backdrop::Tint;
use barrier_kit::choices::{ChoiceCards, ColorWell, RadioGroup};
use barrier_kit::compact::IconButton;
use barrier_kit::controls::{Button, Checkbox, SegmentedControl, Slider, Switch};
use barrier_kit::level::LevelSlider;
use barrier_kit::progress::ProgressBar;
use barrier_kit::style::{Palette, text};
use barrier_kit::symbol::{Symbol, symbol};
use barrier_kit::theme::color;
use barrier_kit::views::form::{Form, Row, Section};
use freya_components::image_viewer::ImageViewer;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Size};
use vow_design::{
    ACCENT_SWATCHES, CAPTION, HEADLINE, Rgba, SPACE_2, SPACE_3, SPACE_4, TEXT, TITLE_2,
};

use crate::albums::Album;
use crate::{Background, Mood, PRESETS, Player, Tab};

const WAVE_WIDTH: f32 = 380.0;
const WAVE_BARS: usize = 64;

fn clock(seconds: f32) -> String {
    format!("{}:{:02}", seconds as u32 / 60, seconds as u32 % 60)
}

pub fn now_playing(albums: &[Album], player: Player, tint: Tint) -> Element {
    let palette = Palette::current();
    let album = &albums[*player.album.read()];
    let track = *player.track.read();
    let length = album.seconds(track) as f32;
    let position = *player.position.read();
    let playing = *player.playing.read();
    let mut seek = player.position;
    let mut volume = player.volume;

    let bars = (0..WAVE_BARS).map(|i| {
        let wave = ((i * 7 + track * 13 + album.title.len()) % 17) as f32 / 17.0;
        let envelope = (std::f32::consts::PI * i as f32 / WAVE_BARS as f32)
            .sin()
            .sqrt();
        let height = 6.0 + (0.3 + 0.7 * wave) * envelope * 40.0;
        let played = (i as f32) / (WAVE_BARS as f32) <= position;
        rect()
            .width(Size::px(WAVE_WIDTH / WAVE_BARS as f32 - 2.0))
            .height(Size::px(height))
            .corner_radius(1.5)
            .background(color(if played {
                palette.label
            } else {
                palette.tertiary_label
            }))
            .into_element()
    });

    rect()
        .width(Size::fill())
        .cross_align(Alignment::Center)
        .spacing(SPACE_3)
        .padding(Gaps::new(SPACE_2, SPACE_4, SPACE_4, SPACE_4))
        .child(
            ImageViewer::new((album.title, album.cover.clone()))
                .width(Size::px(220.0))
                .height(Size::px(220.0))
                .corner_radius(14.0),
        )
        .child(
            rect()
                .cross_align(Alignment::Center)
                .child(text(album.tracks[track], TITLE_2, palette.label))
                .child(text(album.artist, TEXT, palette.secondary_label))
                .child(text(album.title, TEXT, palette.secondary_label)),
        )
        .child(
            rect()
                .a11y_role(AccessibilityRole::Slider)
                .a11y_alt("Posición")
                .width(Size::px(WAVE_WIDTH))
                .height(Size::px(48.0))
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center)
                .spacing(2.0)
                .on_mouse_down(move |event: Event<MouseEventData>| {
                    seek.set((event.element_location.x as f32 / WAVE_WIDTH).clamp(0.0, 1.0))
                })
                .children(bars),
        )
        .child(
            rect()
                .width(Size::px(WAVE_WIDTH))
                .direction(Direction::Horizontal)
                .child(rect().width(Size::flex(1.0)).child(text(
                    clock(position * length),
                    CAPTION,
                    palette.secondary_label,
                )))
                .child(text(
                    format!("−{}", clock((1.0 - position) * length)),
                    CAPTION,
                    palette.secondary_label,
                ))
                .content(Content::flex()),
        )
        .child(
            rect()
                .direction(Direction::Horizontal)
                .spacing(SPACE_4)
                .cross_align(Alignment::Center)
                .child(round(
                    40.0,
                    palette.control_background,
                    IconButton::new(Symbol::Backward, "Anterior")
                        .on_press(move |_| player.skip(-1)),
                ))
                .child(round(
                    52.0,
                    palette.control_hover,
                    IconButton::new(
                        if playing { Symbol::Pause } else { Symbol::Play },
                        if playing { "Pausar" } else { "Reproducir" },
                    )
                    .on_press(move |_| player.toggle()),
                ))
                .child(round(
                    40.0,
                    palette.control_background,
                    IconButton::new(Symbol::Forward, "Siguiente").on_press(move |_| player.skip(1)),
                )),
        )
        .child(
            rect()
                .width(Size::px(WAVE_WIDTH))
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center)
                .spacing(SPACE_2)
                .child(symbol(Symbol::SpeakerWave, 16.0, palette.secondary_label))
                .child(
                    LevelSlider::new("Volumen")
                        .value(*player.volume.read())
                        .tint(tint)
                        .on_change(move |v| volume.set(v)),
                ),
        )
        .into_element()
}

fn round(diameter: f32, fill: Rgba, button: IconButton) -> Rect {
    rect()
        .width(Size::px(diameter))
        .height(Size::px(diameter))
        .corner_radius(diameter / 2.0)
        .center()
        .background(color(fill))
        .child(button)
}

pub fn library(albums: &[Album], player: Player) -> Element {
    let palette = Palette::current();
    let current = *player.album.read();
    let tiles = albums.iter().enumerate().map(|(index, album)| {
        let mut chosen = player.album;
        let (mut track, mut position) = (player.track, player.position);
        let selected = index == current;
        rect()
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(album.title)
            .width(Size::px(140.0))
            .spacing(4.0)
            .on_press(move |_| {
                chosen.set(index);
                track.set(0);
                position.set(0.0);
            })
            .child(
                rect()
                    .corner_radius(12.0)
                    .padding(3.0)
                    .maybe(selected, |el| el.background(color(palette.accent)))
                    .child(
                        ImageViewer::new((album.title, album.cover.clone()))
                            .width(Size::px(134.0))
                            .height(Size::px(134.0))
                            .corner_radius(10.0),
                    ),
            )
            .child(text(album.title, TEXT, palette.label))
            .child(text(
                format!("{} · {}", album.artist, album.year),
                CAPTION,
                palette.secondary_label,
            ))
            .into_element()
    });
    rect()
        .width(Size::fill())
        .padding(Gaps::new(SPACE_2, SPACE_4, SPACE_4, SPACE_4))
        .spacing(SPACE_3)
        .child(text("Biblioteca", TITLE_2, palette.label))
        .child(text(
            "El fondo sigue al disco que elijas.",
            TEXT,
            palette.secondary_label,
        ))
        .child(
            rect()
                .width(Size::fill())
                .direction(Direction::Horizontal)
                .content(Content::wrap())
                .spacing(SPACE_4)
                .children(tiles),
        )
        .into_element()
}

pub fn mixer(mood: Mood, album: &Album) -> Element {
    let tint = mood.tint();
    let (mut controls, mut own) = (mood.controls, mood.own_controls);
    let (mut eq, mut switches, mut checks, mut quality, mut export) = (
        mood.eq,
        mood.switches,
        mood.checks,
        mood.quality,
        mood.export,
    );
    let bands = ["32 Hz", "250 Hz", "1 kHz", "4 kHz", "16 kHz"];
    let mut colour = Section::titled("Color de los controles").row(
        Row::new("Relleno").control(
            SegmentedControl::new("Relleno")
                .segment("Acento")
                .segment("Gradiente del fondo")
                .segment("Propio")
                .selected(*controls.read())
                .on_change(move |i| controls.set(i)),
        ),
    );
    if *controls.read() == 2 {
        colour = colour.row(
            Row::new("Color").control(
                ColorWell::new("Color propio de los controles", &ACCENT_SWATCHES)
                    .selected(Some(*own.read()))
                    .on_change(move |i| own.set(i)),
            ),
        );
    }
    let mut equaliser = Section::titled("Ecualizador");
    for (index, band) in bands.iter().enumerate() {
        equaliser = equaliser.row(
            Row::new(band).control(
                rect().width(Size::px(200.0)).child(
                    LevelSlider::new(band)
                        .value(eq.read()[index])
                        .tint(tint)
                        .on_change(move |v| eq.write()[index] = v),
                ),
            ),
        );
    }
    let switch = |index: usize, title: &'static str| {
        Switch::new(title)
            .on(switches.read()[index])
            .tint(tint)
            .on_toggle(move |on| switches.write()[index] = on)
    };
    let check = |index: usize, title: &'static str| {
        Checkbox::new(title)
            .checked(checks.read()[index])
            .tint(tint)
            .on_toggle(move |on| checks.write()[index] = on)
    };
    let done = *export.read();
    Form::new()
        .section(colour)
        .section(equaliser)
        .section(
            Section::titled("Reproducción")
                .row(
                    Row::new("Normalizar volumen")
                        .detail("Mismo nivel entre discos.")
                        .control(switch(0, "Normalizar volumen")),
                )
                .row(
                    Row::new("Fundido entre canciones")
                        .control(switch(1, "Fundido entre canciones")),
                )
                .row(Row::new("Sin silencios").control(switch(2, "Sin silencios")))
                .row(
                    Row::new("Graves").control(
                        rect()
                            .width(Size::px(200.0))
                            .child(Slider::stepped(5).value(2).tint(tint)),
                    ),
                ),
        )
        .section(
            Section::titled("Calidad")
                .row(
                    Row::new("Transmisión").control(
                        RadioGroup::new("Calidad")
                            .option("Ahorro")
                            .option("Alta")
                            .option("Sin pérdida")
                            .selected(*quality.read())
                            .tint(tint)
                            .on_change(move |i| quality.set(i)),
                    ),
                )
                .row(Row::new("Letras sincronizadas").control(check(0, "Letras sincronizadas")))
                .row(Row::new("Enviar escuchas").control(check(1, "Enviar escuchas"))),
        )
        .section(
            Section::titled("Exportación")
                .row(
                    Row::new(album.title).control(
                        rect()
                            .width(Size::px(260.0))
                            .direction(Direction::Horizontal)
                            .cross_align(Alignment::Center)
                            .spacing(SPACE_2)
                            .content(Content::flex())
                            .child(
                                rect().width(Size::flex(1.0)).child(
                                    ProgressBar::determinate("Exportación", done).tint(tint),
                                ),
                            )
                            .child(text(
                                format!("{} %", (done * 100.0).round()),
                                CAPTION,
                                Palette::current().secondary_label,
                            )),
                    ),
                )
                .row(
                    Row::new("Siguiente pista").control(Button::new("Avanzar").on_press(
                        move |_| {
                            export.set(if done >= 1.0 {
                                0.0
                            } else {
                                (done + 0.125).min(1.0)
                            })
                        },
                    )),
                ),
        )
        .into_element()
}

pub fn appearance(mood: Mood) -> Element {
    let (mut background, mut preset, mut accent, mut own) =
        (mood.background, mood.preset, mood.accent, mood.own_accent);
    let mut fondo = Section::titled("Fondo").row(
        Row::new("Origen")
            .detail("Mezcla conserva siempre el suyo.")
            .control(
                SegmentedControl::new("Origen del fondo")
                    .segment("Portada")
                    .segment("Paleta")
                    .segment("Sin tinte")
                    .selected(background.read().index())
                    .on_change(move |i| background.set(Background::from_index(i))),
            ),
    );
    if *background.read() == Background::Palette {
        let mut cards = ChoiceCards::new("Paleta del fondo")
            .selected(*preset.read())
            .on_change(move |i| preset.set(i));
        for (name, colours) in PRESETS {
            cards = cards.option(name, swatch(colours));
        }
        fondo = fondo.row(Row::new("Paleta").control(cards));
    }
    let mut acento = Section::titled("Acento").row(
        Row::new("Color de acento")
            .detail("Del sistema, sacado del fondo o elegido aquí.")
            .control(
                SegmentedControl::new("Acento")
                    .segment("Sistema")
                    .segment("Del fondo")
                    .segment("Propio")
                    .selected(*accent.read())
                    .on_change(move |i| accent.set(i)),
            ),
    );
    if *accent.read() == 2 {
        acento = acento.row(
            Row::new("Color").control(
                ColorWell::new("Acento propio", &ACCENT_SWATCHES)
                    .selected(Some(*own.read()))
                    .on_change(move |i| own.set(i)),
            ),
        );
    }
    Form::new().section(fondo).section(acento).into_element()
}

fn swatch(colours: [Rgba; 3]) -> Element {
    rect()
        .width(Size::px(72.0))
        .height(Size::px(44.0))
        .corner_radius(8.0)
        .direction(Direction::Horizontal)
        .overflow(Overflow::Clip)
        .children(colours.map(|c| {
            rect()
                .width(Size::percent(100.0 / 3.0))
                .height(Size::fill())
                .background(color(c))
                .into_element()
        }))
        .into_element()
}

pub fn queue(albums: &[Album], player: Player) -> Element {
    let palette = Palette::current();
    let (album, track) = (*player.album.read(), *player.track.read());
    let upcoming: Vec<(usize, usize)> = (0..12)
        .scan((album, track), |(a, t), _| {
            let current = (*a, *t);
            *t += 1;
            if *t >= albums[*a].tracks.len() {
                *t = 0;
                *a = (*a + 1) % albums.len();
            }
            Some(current)
        })
        .collect();
    let minutes: u32 = upcoming
        .iter()
        .map(|(a, t)| albums[*a].seconds(*t))
        .sum::<u32>()
        / 60;
    let rows = upcoming.into_iter().enumerate().map(|(index, (a, t))| {
        let (mut chosen_album, mut chosen_track, mut position) =
            (player.album, player.track, player.position);
        rect()
            .a11y_role(AccessibilityRole::ListItem)
            .a11y_alt(albums[a].tracks[t])
            .width(Size::fill())
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(SPACE_2)
            .padding(Gaps::new(5.0, 8.0, 5.0, 8.0))
            .corner_radius(8.0)
            .maybe(index == 0, |el| el.background(color(palette.control_hover)))
            .on_press(move |_| {
                chosen_album.set(a);
                chosen_track.set(t);
                position.set(0.0);
            })
            .child(
                ImageViewer::new((albums[a].title, albums[a].cover.clone()))
                    .width(Size::px(34.0))
                    .height(Size::px(34.0))
                    .corner_radius(6.0),
            )
            .child(
                rect()
                    .width(Size::flex(1.0))
                    .child(text(albums[a].tracks[t], TEXT, palette.label))
                    .child(text(albums[a].artist, CAPTION, palette.secondary_label)),
            )
            .child(text(
                clock(albums[a].seconds(t) as f32),
                CAPTION,
                palette.secondary_label,
            ))
            .content(Content::flex())
            .into_element()
    });
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .padding(Gaps::new(0.0, SPACE_2, SPACE_3, SPACE_2))
        .child(
            rect()
                .width(Size::fill())
                .direction(Direction::Horizontal)
                .padding(Gaps::new(0.0, 8.0, SPACE_2, 8.0))
                .content(Content::flex())
                .child(
                    rect()
                        .width(Size::flex(1.0))
                        .child(text("Cola", HEADLINE, palette.label)),
                )
                .child(text(
                    format!("{minutes} min restantes"),
                    CAPTION,
                    palette.secondary_label,
                )),
        )
        .child(
            barrier_kit::controls::ScrollView::new()
                .child(rect().width(Size::fill()).children(rows)),
        )
        .into_element()
}

pub fn navigation(current: Tab, mut tab: State<Tab>, labels: bool) -> Element {
    let palette = Palette::current();
    let items = Tab::ALL.iter().map(|&item| {
        let selected = item == current;
        rect()
            .a11y_role(AccessibilityRole::Tab)
            .a11y_alt(item.title())
            .a11y_builder(move |node| node.set_selected(selected))
            .width(if labels { Size::fill() } else { Size::px(44.0) })
            .height(Size::px(34.0))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .main_align(if labels {
                Alignment::Start
            } else {
                Alignment::Center
            })
            .spacing(SPACE_3)
            .padding(Gaps::new(0.0, 10.0, 0.0, 10.0))
            .corner_radius(8.0)
            .maybe(selected, |el| el.background(color(palette.control_hover)))
            .on_press(move |_| tab.set(item))
            .child(symbol(
                item.symbol(),
                16.0,
                if selected {
                    palette.label
                } else {
                    palette.secondary_label
                },
            ))
            .maybe_child(labels.then(|| {
                text(
                    item.title(),
                    TEXT,
                    if selected {
                        palette.label
                    } else {
                        palette.secondary_label
                    },
                )
            }))
            .into_element()
    });
    rect()
        .spacing(2.0)
        .padding(Gaps::new(SPACE_2, SPACE_2, SPACE_2, SPACE_2))
        .children(items)
        .into_element()
}

pub fn tab_bar(current: Tab, mut tab: State<Tab>) -> Element {
    let palette = Palette::current();
    let items = Tab::ALL.iter().map(|&item| {
        let ink = if item == current {
            palette.label
        } else {
            palette.secondary_label
        };
        rect()
            .a11y_role(AccessibilityRole::Tab)
            .a11y_alt(item.title())
            .width(Size::flex(1.0))
            .cross_align(Alignment::Center)
            .spacing(3.0)
            .padding(Gaps::new(6.0, 0.0, 6.0, 0.0))
            .corner_radius(8.0)
            .maybe(item == current, |el| {
                el.background(color(palette.control_background))
            })
            .on_press(move |_| tab.set(item))
            .child(symbol(item.symbol(), 18.0, ink))
            .child(text(item.title(), CAPTION, ink))
            .into_element()
    });
    rect()
        .width(Size::fill())
        .direction(Direction::Horizontal)
        .content(Content::flex())
        .spacing(4.0)
        .padding(Gaps::new(4.0, SPACE_2, SPACE_2, SPACE_2))
        .children(items)
        .into_element()
}

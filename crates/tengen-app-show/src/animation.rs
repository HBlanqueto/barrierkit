use barrier_kit::backdrop::{Accent, Tint};
use barrier_kit::controls::{Button, ButtonStyle, SegmentedControl, Switch};
use barrier_kit::patterns::preferences::{Pane, PaneGroup};
use barrier_kit::progress::{ProgressBar, Spinner};
use barrier_kit::symbol::Symbol;
use barrier_kit::views::form::{Form, Row, Section};
use freya_animation::prelude::Function;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Size};
use vow_design::{DURATION_BASE, DURATION_SLOW_MS, SPACE_2, SPLIT_VIEW, SWITCH};

use crate::chrome::segmented;
use crate::events::{
    Arrivals, CopyToast, HoverCard, Inbox, PlusToggle, Popover, PressCard, ShakeField,
};
use crate::kit::{Download, SCENES, sidebar_section};
use crate::motion::{EasedTrack, Property, PropertyTile, SpringTrack};
use crate::slides::{Stage, TRANSITIONS, pager};
use crate::{preview, tint};

const CURVES: [(&str, Function); 7] = [
    ("Lineal", Function::Linear),
    ("Seno", Function::Sine),
    ("Cúbica", Function::Cubic),
    ("Exponencial", Function::Expo),
    ("Atrás", Function::Back),
    ("Elástica", Function::Elastic),
    ("Rebote", Function::Bounce),
];

const SPRINGS: [(&str, f32); 3] = [("Rígido", 0.0), ("Suave", 0.3), ("Vivo", 0.7)];

const PROPERTIES: [(&str, Property); 7] = [
    ("Opacidad", Property::Opacity),
    ("Escala", Property::Scale),
    ("Rotación", Property::Rotation),
    ("Desplazamiento", Property::Offset),
    ("Color", Property::Tint),
    ("Radio", Property::Radius),
    ("Anchura", Property::Width),
];

fn curves(mut ahead: State<bool>) -> Form {
    let at_end = *ahead.read();
    let play = Row::new("Reproducir").control(
        Button::new(if at_end { "Volver" } else { "Reproducir" })
            .style(ButtonStyle::Primary)
            .on_press(move |_| ahead.toggle()),
    );
    let durations = [
        ("Rápida", SWITCH.transition_ms),
        ("Base", DURATION_BASE),
        ("Lenta", DURATION_SLOW_MS),
    ];
    Form::new()
        .section(CURVES.iter().fold(
            Section::titled("Curvas de Freya").row(play),
            |section, &(name, function)| {
                section.row(Row::new(name).control(EasedTrack {
                    ahead: at_end,
                    function,
                    ms: DURATION_SLOW_MS,
                }))
            },
        ))
        .section(SPRINGS.iter().fold(
            Section::titled("Resorte de Grade"),
            |section, &(name, bounce)| {
                section.row(Row::new(name).control(SpringTrack {
                    ahead: at_end,
                    bounce,
                }))
            },
        ))
        .section(durations.into_iter().fold(
            Section::titled("Duraciones de Tengen"),
            |section, (name, ms)| {
                section.row(Row::new(name).control(EasedTrack {
                    ahead: at_end,
                    function: Function::Cubic,
                    ms,
                }))
            },
        ))
}

fn properties(mut on: State<bool>) -> Form {
    let animated = *on.read();
    let toggle = Row::new("Animar").control(
        Switch::new("Animar")
            .on(animated)
            .on_toggle(move |value: bool| on.set(value)),
    );
    Form::new().section(PROPERTIES.iter().fold(
        Section::titled("Propiedades").row(toggle),
        |section, &(name, property)| {
            section.row(Row::new(name).control(PropertyTile {
                on: animated,
                property,
            }))
        },
    ))
}

fn kit_controls(mut backdrop: State<usize>) -> Form {
    let selected = *backdrop.read();
    Form::new()
        .section(
            Section::titled("Transiciones del kit")
                .row(Row::new("Interruptor").control(Switch::new("Animado")))
                .row(Row::new("Sección lateral").control(sidebar_section()))
                .row(
                    Row::new("Indeterminado").control(
                        rect()
                            .width(Size::px(SPLIT_VIEW.inspector))
                            .child(ProgressBar::indeterminate("Buscando")),
                    ),
                )
                .row(Row::new("Actividad").control(Spinner::new("Cargando")))
                .row(Row::new("Descarga").control(Download)),
        )
        .section(
            Section::titled("Fondo tintado")
                .row(Row::new("Escena").control(segmented(
                    "Escena",
                    &SCENES.map(|(name, _)| name),
                    selected,
                    move |index| backdrop.set(index),
                )))
                .row(
                    Row::new("Fundido")
                        .detail("El fondo y su acento funden entre escenas.")
                        .control(preview(
                            SCENES[selected].1,
                            Accent::FromBackdrop,
                            Tint::Accent,
                        )),
                ),
        )
}

fn appearing() -> Form {
    Form::new().section(
        Section::titled("Entrar y salir")
            .row(Row::new("Lista").control(Arrivals))
            .row(Row::new("Tarjeta").control(Popover)),
    )
}

fn navigation(page: State<usize>, mut transition: State<usize>, mut period: State<usize>) -> Form {
    let style = *transition.read();
    let shown_period = *period.read();
    Form::new()
        .section(
            Section::titled("Diapositivas")
                .row(Row::new("Transición").control(segmented(
                    "Transición",
                    &TRANSITIONS,
                    style,
                    move |index| transition.set(index),
                )))
                .row(
                    Row::new("Carrusel").control(
                        rect()
                            .cross_align(Alignment::Center)
                            .spacing(SPACE_2)
                            .child(Stage {
                                page: *page.read(),
                                transition: style,
                            })
                            .child(pager(page)),
                    ),
                ),
        )
        .section(
            Section::titled("Pestañas").row(
                Row::new("Con segmentos")
                    .detail("El contenido se desplaza hacia el segmento elegido.")
                    .control(
                        rect()
                            .cross_align(Alignment::Center)
                            .spacing(SPACE_2)
                            .child(
                                SegmentedControl::new("Periodo")
                                    .segment("Día")
                                    .segment("Semana")
                                    .segment("Mes")
                                    .selected(shown_period)
                                    .on_change(move |index: usize| period.set(index)),
                            )
                            .child(Stage {
                                page: shown_period,
                                transition: 0,
                            }),
                    ),
            ),
        )
}

fn feedback() -> Form {
    Form::new()
        .section(
            Section::titled("Puntero")
                .row(Row::new("Pasar por encima").control(HoverCard))
                .row(
                    Row::new("Pulsar")
                        .detail("Se hunde al pulsar y rebota al soltar.")
                        .control(PressCard),
                )
                .row(Row::new("Alternar").control(PlusToggle)),
        )
        .section(
            Section::titled("Respuesta")
                .row(
                    Row::new("Error")
                        .detail("La contraseña es «tengen».")
                        .control(ShakeField),
                )
                .row(Row::new("Notificación").control(Inbox))
                .row(Row::new("Aviso").control(CopyToast)),
        )
}

fn accessibility(mut reduced: State<bool>) -> Form {
    Form::new().section(
        Section::titled("Accesibilidad").row(
            Row::new("Reducir movimiento")
                .detail("Cada animación salta a su estado final.")
                .control(
                    Switch::new("Reducir movimiento")
                        .on(*reduced.read())
                        .on_toggle(move |on: bool| reduced.set(on)),
                ),
        ),
    )
}

pub fn group(reduced: State<bool>) -> PaneGroup {
    let ahead = use_state(|| false);
    let animated = use_state(|| false);
    let backdrop = use_state(|| 0);
    let page = use_state(|| 0);
    let transition = use_state(|| 0);
    let period = use_state(|| 0);
    PaneGroup::new(Some("Animación"))
        .pane(Pane::new(
            "curvas",
            "Curvas",
            Symbol::SlidersVertical,
            tint(0),
            curves(ahead),
        ))
        .pane(Pane::new(
            "propiedades",
            "Propiedades",
            Symbol::SquareGrid,
            tint(1),
            properties(animated),
        ))
        .pane(Pane::new(
            "controles",
            "Kit en movimiento",
            Symbol::ChevronRight,
            tint(6),
            kit_controls(backdrop),
        ))
        .pane(Pane::new(
            "aparicion",
            "Aparición",
            Symbol::Plus,
            tint(4),
            appearing(),
        ))
        .pane(Pane::new(
            "diapositivas",
            "Diapositivas",
            Symbol::Play,
            tint(7),
            navigation(page, transition, period),
        ))
        .pane(Pane::new(
            "eventos",
            "Puntero y respuesta",
            Symbol::CircleHalf,
            tint(5),
            feedback(),
        ))
        .pane(Pane::new(
            "movimiento",
            "Movimiento",
            Symbol::Pause,
            tint(2),
            accessibility(reduced),
        ))
}

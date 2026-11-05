use barrier_kit::app::save_defaults;
use barrier_kit::chrome::{
    ChromeStyle, RailStyle, RestingEdge, Reveal, TitlebarLayout, WindowChrome,
};
use barrier_kit::compact::IconButton;
use barrier_kit::controls::{SegmentedControl, Switch};
use barrier_kit::patterns::preferences::{Pane, PaneGroup};
use barrier_kit::platform::FullscreenBar;
use barrier_kit::popup_button::PopUpButton;
use barrier_kit::symbol::Symbol;
use barrier_kit::views::form::{Form, Row, Section};
use barrier_kit::views::split::{NarrowSidebar, SidebarToggle};
use freya_core::prelude::*;
use serde::{Deserialize, Serialize};
use torin::prelude::Direction;
use vow_appearance::SidebarStyle;
use vow_design::TITLEBAR_RAIL;

use crate::{APP, tint};

const LAYOUTS: [(TitlebarLayout, &str); 2] = [
    (TitlebarLayout::Top, "Arriba"),
    (TitlebarLayout::Rail, "Rail"),
];

const REVEALS: [(Reveal, &str); 5] = [
    (Reveal::Always, "Siempre"),
    (Reveal::Drawer, "Cajón"),
    (Reveal::Push, "Empuja"),
    (Reveal::Corner, "Esquina"),
    (Reveal::Popup, "Píldora emergente"),
];

const EDGES: [RestingEdge; 3] = [RestingEdge::None, RestingEdge::Line, RestingEdge::Accent];

const BARS: [(FullscreenBar, &str); 3] = [
    (FullscreenBar::Solid, "A · Lisa"),
    (FullscreenBar::Glass, "B1 · Vidrio en banda"),
    (FullscreenBar::ProgressiveGlass, "B2 · Vidrio progresivo"),
];

const SIDEBAR_LIMITS: [(&str, Option<(f32, f32)>); 3] = [
    ("Tengen", None),
    ("Estrecha", Some((160.0, 220.0))),
    ("Ancha", Some((240.0, 420.0))),
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub layout: TitlebarLayout,
    pub wide_rail: bool,
    pub round_lights: bool,
    pub rail_unified: bool,
    pub rail_top_items: bool,
    pub rail_bottom_items: bool,
    pub reveal: Reveal,
    pub resting_edge: RestingEdge,
    pub flat_sidebar: bool,
    pub sidebar_fixed: bool,
    pub sidebar_limits: usize,
    pub narrow_sidebar: NarrowSidebar,
    pub fullscreen_bar: FullscreenBar,
    pub fullscreen_push: bool,
}

impl Prefs {
    fn style(self) -> ChromeStyle {
        ChromeStyle {
            layout: self.layout,
            rail_width: if self.wide_rail {
                TITLEBAR_RAIL.width_wide
            } else {
                TITLEBAR_RAIL.width
            },
            round_lights: self.round_lights,
            rail_style: if self.rail_unified {
                RailStyle::Unified
            } else {
                RailStyle::Normal
            },
            reveal: self.reveal,
            resting_edge: self.resting_edge,
        }
    }

    pub fn sidebar_style(self) -> SidebarStyle {
        if self.flat_sidebar {
            SidebarStyle::Flat
        } else {
            SidebarStyle::Island
        }
    }

    pub fn sidebar_resizable(self) -> bool {
        !self.sidebar_fixed
    }

    pub fn sidebar_range(self) -> Option<(f32, f32)> {
        SIDEBAR_LIMITS[self.sidebar_limits.min(SIDEBAR_LIMITS.len() - 1)].1
    }
}

fn update(mut prefs: State<Prefs>, change: impl FnOnce(&mut Prefs)) {
    let mut next = *prefs.peek();
    change(&mut next);
    prefs.set(next);
    if let Err(error) = save_defaults(APP, &next) {
        tracing::warn!(%error, "tengen-app-show: no se pudieron guardar los ajustes");
    }
}

pub(crate) fn segmented(
    label: &'static str,
    segments: &[&'static str],
    selected: usize,
    on_change: impl FnMut(usize) + 'static,
) -> SegmentedControl {
    segments
        .iter()
        .fold(SegmentedControl::new(label), |control, segment| {
            control.segment(segment)
        })
        .selected(selected)
        .on_change(on_change)
}

fn popup<T: PartialEq + Copy + 'static>(
    label: &'static str,
    options: &'static [(T, &'static str)],
    current: T,
    on_change: impl Fn(T) + 'static,
) -> PopUpButton {
    options
        .iter()
        .fold(PopUpButton::new(label), |button, (_, name)| {
            button.item(name)
        })
        .selected(
            options
                .iter()
                .position(|(value, _)| *value == current)
                .unwrap_or_default(),
        )
        .on_change(move |i: usize| on_change(options[i].0))
}

fn yes_no(label: &'static str, on: bool, on_change: impl Fn(bool) + 'static) -> SegmentedControl {
    segmented(label, &["No", "Sí"], usize::from(on), move |i| {
        on_change(i == 1)
    })
}

fn titlebar(prefs: State<Prefs>) -> Form {
    let current = *prefs.read();
    let mut titlebar = Section::titled("Titlebar")
        .row(
            Row::new("Diseño").control(popup("Diseño", &LAYOUTS, current.layout, move |layout| {
                update(prefs, |p| p.layout = layout)
            })),
        )
        .row(Row::new("Ancho del rail").control(segmented(
            "Ancho del rail",
            &["Normal", "Ancho"],
            usize::from(current.wide_rail),
            move |i| update(prefs, |p| p.wide_rail = i == 1),
        )))
        .row(Row::new("Semáforos").control(segmented(
            "Semáforos",
            &["Cuadrados", "Redondos"],
            usize::from(current.round_lights),
            move |i| update(prefs, |p| p.round_lights = i == 1),
        )));
    if current.layout == TitlebarLayout::Rail {
        titlebar = titlebar
            .row(Row::new("Estilo del rail").control(segmented(
                "Estilo del rail",
                &["Normal", "Unificado"],
                usize::from(current.rail_unified),
                move |i| update(prefs, |p| p.rail_unified = i == 1),
            )))
            .row(Row::new("Elementos arriba").control(yes_no(
                "Elementos arriba",
                current.rail_top_items,
                move |on| update(prefs, |p| p.rail_top_items = on),
            )))
            .row(Row::new("Elementos abajo").control(yes_no(
                "Elementos abajo",
                current.rail_bottom_items,
                move |on| update(prefs, |p| p.rail_bottom_items = on),
            )));
    }
    titlebar = titlebar.row(Row::new("Aparición").control(popup(
        "Aparición",
        &REVEALS,
        current.reveal,
        move |reveal| update(prefs, |p| p.reveal = reveal),
    )));
    if matches!(
        current.reveal,
        Reveal::Drawer | Reveal::Push | Reveal::Popup
    ) {
        titlebar = titlebar.row(
            Row::new("Borde en reposo").control(segmented(
                "Borde en reposo",
                &["Nada", "Línea fina", "Franja de acento"],
                EDGES
                    .iter()
                    .position(|edge| *edge == current.resting_edge)
                    .unwrap_or_default(),
                move |i| update(prefs, |p| p.resting_edge = EDGES[i]),
            )),
        );
    }
    Form::new().section(titlebar)
}

fn fullscreen(prefs: State<Prefs>) -> Form {
    let current = *prefs.read();
    Form::new().section(
        Section::titled("Pantalla completa")
            .row(Row::new("Barra").control(popup(
                "Barra",
                &BARS,
                current.fullscreen_bar,
                move |bar| update(prefs, |p| p.fullscreen_bar = bar),
            )))
            .row(
                Row::new("Empujar el contenido").control(
                    Switch::new("Empujar el contenido")
                        .on(current.fullscreen_push)
                        .on_toggle(move |on: bool| update(prefs, |p| p.fullscreen_push = on)),
                ),
            ),
    )
}

fn sidebar(prefs: State<Prefs>) -> Form {
    let current = *prefs.read();
    Form::new().section(
        Section::titled("Barra lateral")
            .row(Row::new("Estilo").control(segmented(
                "Estilo de la barra lateral",
                &["Isla", "Plana"],
                usize::from(current.flat_sidebar),
                move |i| update(prefs, |p| p.flat_sidebar = i == 1),
            )))
            .row(Row::new("Mostrar u ocultar").control(SidebarToggle))
            .row(
                Row::new("Redimensionable").control(
                    Switch::new("Redimensionable")
                        .on(current.sidebar_resizable())
                        .on_toggle(move |on: bool| update(prefs, |p| p.sidebar_fixed = !on)),
                ),
            )
            .row(Row::new("Límites").control(segmented(
                "Límites",
                &SIDEBAR_LIMITS.map(|(name, _)| name),
                current.sidebar_limits,
                move |i| update(prefs, |p| p.sidebar_limits = i),
            )))
            .row(Row::new("En ventana estrecha").control(segmented(
                "Barra lateral en ventana estrecha",
                &["Superpuesta", "Forzada"],
                usize::from(current.narrow_sidebar == NarrowSidebar::Push),
                move |i| {
                    update(prefs, |p| {
                        p.narrow_sidebar = if i == 1 {
                            NarrowSidebar::Push
                        } else {
                            NarrowSidebar::Overlay
                        }
                    })
                },
            ))),
    )
}

pub fn group(prefs: State<Prefs>) -> PaneGroup {
    PaneGroup::new(Some("Ventana"))
        .pane(Pane::new(
            "titlebar",
            "Titlebar",
            Symbol::SidebarLeft,
            tint(0),
            titlebar(prefs),
        ))
        .pane(Pane::new(
            "pantalla-completa",
            "Pantalla completa",
            Symbol::SquareGrid,
            tint(5),
            fullscreen(prefs),
        ))
        .pane(Pane::new(
            "barra-lateral",
            "Barra lateral",
            Symbol::SidebarLeft,
            tint(3),
            sidebar(prefs),
        ))
}

fn rail_items(first: (Symbol, &'static str), second: (Symbol, &'static str)) -> Rect {
    rect()
        .direction(Direction::Vertical)
        .spacing(TITLEBAR_RAIL.padding_bottom)
        .child(IconButton::new(first.0, first.1))
        .child(IconButton::new(second.0, second.1))
}

pub fn window(prefs: Prefs, content: impl IntoElement) -> WindowChrome {
    let mut chrome = WindowChrome::new(prefs.style(), content);
    if prefs.rail_top_items {
        chrome = chrome.rail_top(rail_items(
            (Symbol::Plus, "Nuevo"),
            (Symbol::MagnifyingGlass, "Buscar"),
        ));
    }
    if prefs.rail_bottom_items {
        chrome = chrome.rail_bottom(rail_items(
            (Symbol::SquareGrid, "Diseño"),
            (Symbol::SlidersVertical, "Medidas"),
        ));
    }
    chrome
}

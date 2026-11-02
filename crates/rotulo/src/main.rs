//! Rótulo: a test app whose own window switches between the top titlebar and
//! the vertical ones, on either side, with a flat or island sidebar.

use barrier_kit::app::{load_defaults, save_defaults};
use barrier_kit::chrome::{ChromeStyle, RestingEdge, Reveal, Side, TitlebarLayout, WindowChrome};
use barrier_kit::compact::IconButton;
use barrier_kit::controls::{ScrollView, SegmentedControl};
use barrier_kit::platform::{Decorations, WindowOptions, run_window};
use barrier_kit::popup_button::PopUpButton;
use barrier_kit::sidebar::{Sidebar, SidebarItem};
use barrier_kit::symbol::Symbol;
use barrier_kit::theme::use_appearance;
use barrier_kit::titlebar::Titlebar;
use barrier_kit::views::form::{Form, Row, Section};
use barrier_kit::views::split::{SidebarToggle, SplitView};
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use serde::{Deserialize, Serialize};
use torin::prelude::{Content, Direction, Size};
use vow_appearance::SidebarStyle;
use vow_design::{ACCENT_SWATCHES, TITLEBAR_RAIL, WINDOW_MIN_SIZE};

const APP: &str = "rotulo";

const LAYOUTS: [(TitlebarLayout, &str); 5] = [
    (TitlebarLayout::Top, "Arriba"),
    (TitlebarLayout::Rail, "Rail"),
    (TitlebarLayout::Unified, "Unificado"),
    (TitlebarLayout::Pill, "Píldora"),
    (TitlebarLayout::Actions, "Acciones"),
];

const REVEALS: [(Reveal, &str); 5] = [
    (Reveal::Always, "Siempre"),
    (Reveal::Drawer, "Cajón"),
    (Reveal::Push, "Empuja"),
    (Reveal::Corner, "Esquina"),
    (Reveal::Popup, "Píldora emergente"),
];

const EDGES: [RestingEdge; 3] = [RestingEdge::None, RestingEdge::Line, RestingEdge::Accent];

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct Prefs {
    layout: TitlebarLayout,
    side: Side,
    wide_rail: bool,
    round_lights: bool,
    reveal: Reveal,
    resting_edge: RestingEdge,
    flat_sidebar: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            layout: TitlebarLayout::Rail,
            side: Side::Left,
            wide_rail: false,
            round_lights: false,
            reveal: Reveal::Always,
            resting_edge: RestingEdge::None,
            flat_sidebar: false,
        }
    }
}

impl Prefs {
    fn chrome(self) -> ChromeStyle {
        ChromeStyle {
            layout: self.layout,
            side: self.side,
            rail_width: if self.wide_rail {
                TITLEBAR_RAIL.width_wide
            } else {
                TITLEBAR_RAIL.width
            },
            round_lights: self.round_lights,
            reveal: self.reveal,
            resting_edge: self.resting_edge,
        }
    }

    fn layout_name(self) -> &'static str {
        LAYOUTS
            .iter()
            .find(|(layout, _)| *layout == self.layout)
            .map_or("", |(_, name)| name)
    }
}

fn update(mut prefs: State<Prefs>, change: impl FnOnce(&mut Prefs)) {
    let mut next = *prefs.peek();
    change(&mut next);
    prefs.set(next);
    if let Err(error) = save_defaults(APP, &next) {
        tracing::warn!(%error, "rótulo: no se pudieron guardar los ajustes");
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Design,
    Measures,
}

impl Page {
    fn title(self) -> &'static str {
        match self {
            Page::Design => "Diseño",
            Page::Measures => "Medidas",
        }
    }
}

fn segmented(
    label: &'static str,
    segments: &[&'static str],
    selected: usize,
    on_change: impl Fn(usize) + 'static,
) -> SegmentedControl {
    segments
        .iter()
        .fold(SegmentedControl::new(label), |control, segment| {
            control.segment(segment)
        })
        .selected(selected)
        .on_change(on_change)
}

fn design(prefs: State<Prefs>) -> Form {
    let current = *prefs.read();
    let layout_index = LAYOUTS
        .iter()
        .position(|(layout, _)| *layout == current.layout)
        .unwrap_or_default();
    let mut titlebar = Section::titled("Titlebar")
        .row(
            Row::new("Diseño").control(
                LAYOUTS
                    .iter()
                    .fold(PopUpButton::new("Diseño"), |button, (_, name)| {
                        button.item(name)
                    })
                    .selected(layout_index)
                    .on_change(move |i: usize| update(prefs, |p| p.layout = LAYOUTS[i].0)),
            ),
        )
        .row(Row::new("Lado").control(segmented(
            "Lado",
            &["Izquierda", "Derecha"],
            usize::from(current.side == Side::Right),
            move |i| {
                update(prefs, |p| {
                    p.side = if i == 0 { Side::Left } else { Side::Right }
                })
            },
        )))
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
    titlebar = titlebar.row(
        Row::new("Aparición").control(
            REVEALS
                .iter()
                .fold(PopUpButton::new("Aparición"), |button, (_, name)| {
                    button.item(name)
                })
                .selected(
                    REVEALS
                        .iter()
                        .position(|(reveal, _)| *reveal == current.reveal)
                        .unwrap_or_default(),
                )
                .on_change(move |i: usize| update(prefs, |p| p.reveal = REVEALS[i].0)),
        ),
    );
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
    Form::new().section(titlebar).section(
        Section::titled("Barra lateral")
            .row(Row::new("Estilo").control(segmented(
                "Estilo de la barra lateral",
                &["Isla", "Plana"],
                usize::from(current.flat_sidebar),
                move |i| update(prefs, |p| p.flat_sidebar = i == 1),
            )))
            .row(Row::new("Mostrar u ocultar").control(SidebarToggle)),
    )
}

fn measures(prefs: Prefs) -> Form {
    let chrome = prefs.chrome();
    let width = match prefs.layout {
        TitlebarLayout::Top => String::from("—"),
        TitlebarLayout::Actions => format!("{} px", TITLEBAR_RAIL.actions_width),
        _ => format!("{} px", chrome.rail_width),
    };
    Form::new().section(
        Section::titled("Diseño elegido")
            .row(Row::value("Diseño", prefs.layout_name()))
            .row(Row::value(
                "Lado",
                match prefs.side {
                    Side::Left => "Izquierda",
                    Side::Right => "Derecha",
                },
            ))
            .row(Row::value("Ancho del rail", width))
            .row(Row::value(
                "Semáforos",
                if prefs.round_lights {
                    "Redondos"
                } else {
                    "Cuadrados"
                },
            ))
            .row(Row::value(
                "Barra lateral",
                if prefs.flat_sidebar { "Plana" } else { "Isla" },
            )),
    )
}

#[derive(PartialEq)]
struct Rotulo {
    prefs: Prefs,
}

impl App for Rotulo {
    fn render(&self) -> impl IntoElement {
        let initial = self.prefs;
        let prefs = use_state(|| initial);
        let mut page = use_state(|| Page::Design);
        use_appearance(move |config| {
            config.sidebar_style = if prefs.read().flat_sidebar {
                SidebarStyle::Flat
            } else {
                SidebarStyle::Island
            };
        });

        let current = *page.read();
        let item = |target: Page, symbol: Symbol, tint| {
            SidebarItem::new(target.title())
                .symbol(symbol, tint)
                .selected(current == target)
                .on_select(move |_| page.set(target))
        };
        let sidebar = Sidebar::new()
            .traffic_lights(true)
            .child(item(Page::Design, Symbol::SquareGrid, ACCENT_SWATCHES[0].1))
            .child(item(
                Page::Measures,
                Symbol::SlidersVertical,
                ACCENT_SWATCHES[4].1,
            ));
        let form = match current {
            Page::Design => design(prefs),
            Page::Measures => measures(*prefs.read()),
        };
        let detail = rect()
            .width(Size::fill())
            .height(Size::fill())
            .direction(Direction::Vertical)
            .content(Content::flex())
            .child(
                Titlebar::new()
                    .title(current.title())
                    .with_sidebar(true)
                    .left(SidebarToggle),
            )
            .child(
                ScrollView::new()
                    .width(Size::fill())
                    .height(Size::flex(1.0))
                    .child(form),
            );
        let footer = rect()
            .direction(Direction::Vertical)
            .spacing(TITLEBAR_RAIL.padding_bottom)
            .child(
                IconButton::new(Symbol::SquareGrid, Page::Design.title())
                    .on_press(move |_| page.set(Page::Design)),
            )
            .child(
                IconButton::new(Symbol::SlidersVertical, Page::Measures.title())
                    .on_press(move |_| page.set(Page::Measures)),
            );
        WindowChrome::new(prefs.read().chrome(), SplitView::new(sidebar, detail)).footer(footer)
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    run_window(
        AppComponent::new(Rotulo {
            prefs: load_defaults(APP),
        }),
        WindowOptions::new("Rótulo", APP)
            .decorations(Decorations::Client)
            .size((880, 560))
            .min_size(Some(WINDOW_MIN_SIZE)),
    )
    .expect("rotulo");
}

#[cfg(test)]
mod tests {
    use super::*;
    use freya_testing::TestingRunner;
    use torin::prelude::Size2D;
    use vow_design::TITLEBAR;

    fn light_groups(test: &TestingRunner) -> usize {
        let diameter = TITLEBAR.buttons.diameter as f32;
        let length = 3.0 * diameter + 2.0 * TITLEBAR.buttons.spacing as f32;
        let mut groups = test.find_many(|node, _| {
            let area = node.layout().area;
            let size = (area.width(), area.height());
            (size == (length, diameter) || size == (diameter, length))
                .then_some((area.min_x() as i32, area.min_y() as i32))
        });
        groups.sort_unstable();
        groups.dedup();
        groups.len()
    }

    fn render(prefs: Prefs) -> TestingRunner {
        let (mut test, _) = TestingRunner::new(
            AppComponent::new(Rotulo { prefs }),
            Size2D::new(900.0, 560.0),
            |_| {},
            1.0,
        );
        test.sync_and_update();
        test
    }

    #[test]
    fn every_layout_shows_the_traffic_lights_once() {
        for (layout, name) in LAYOUTS {
            for side in [Side::Left, Side::Right] {
                for flat_sidebar in [false, true] {
                    let prefs = Prefs {
                        layout,
                        side,
                        flat_sidebar,
                        ..Default::default()
                    };
                    assert_eq!(light_groups(&render(prefs)), 1, "{name} {prefs:?}");
                }
            }
        }
    }

    #[test]
    fn the_popup_hides_the_lights_until_the_pointer_reaches_the_edge() {
        for (layout, edge) in [
            (TitlebarLayout::Top, (450.0, 2.0)),
            (TitlebarLayout::Rail, (2.0, 280.0)),
        ] {
            let mut test = render(Prefs {
                layout,
                reveal: Reveal::Popup,
                ..Default::default()
            });
            assert_eq!(light_groups(&test), 0, "{layout:?} at rest");
            test.move_cursor(edge);
            test.poll(
                std::time::Duration::from_millis(16),
                std::time::Duration::from_millis(400),
            );
            assert_eq!(light_groups(&test), 1, "{layout:?} revealed");
        }
    }
}

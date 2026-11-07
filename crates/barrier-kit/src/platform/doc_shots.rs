use crate::chrome::{ChromeStyle, RailStyle, TitlebarLayout, WindowChrome};
use crate::compact::IconButton;
use crate::controls::Switch;
use crate::patterns::preferences::{Pane, PaneGroup, PreferencesWindow};
use crate::platform::FullscreenBar;
use crate::sidebar::{Sidebar, SidebarItem};
use crate::symbol::Symbol;
use crate::titlebar::Titlebar;
use crate::views::form::{Form, Row, Section};
use crate::views::split::{SidebarToggle, SplitView};
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Direction, Position, Size, Size2D};
use vow_appearance::{AppearanceConfig, SidebarStyle, resolve};
use vow_design::ACCENT_SWATCHES;

use super::tests::{REVEAL_STEP, REVEAL_TIME, window_controls};
use crate::platform::window::{WindowControls, WindowStatus};

#[derive(Clone, Copy, PartialEq)]
enum Scene {
    Documents { sidebar: bool },
    Design { state: &'static str },
    Buttons { stripe: bool, state: &'static str },
    Prefs,
}

#[derive(Clone, Copy)]
struct Shot {
    name: &'static str,
    size: (f32, f32),
    scene: Scene,
    style: ChromeStyle,
    items: bool,
    status: WindowStatus,
    overlay: bool,
    bar_down: bool,
    bar: FullscreenBar,
    push: bool,
    flat: bool,
    compositor_bar: bool,
}

fn tint(index: usize) -> vow_design::Rgba {
    ACCENT_SWATCHES[index % ACCENT_SWATCHES.len()].1
}

fn bar_left() -> Rect {
    rect().direction(Direction::Horizontal).child(SidebarToggle)
}

struct Case(Shot);

impl App for Case {
    fn render(&self) -> impl IntoElement {
        let shot = self.0;
        let content: Element = match shot.scene {
            Scene::Documents { sidebar } => {
                let form = Form::new().section(
                    Section::titled("Documento")
                        .row(Row::value("Tamaño", "248 KB"))
                        .row(Row::value("Modificado", "Hoy")),
                );
                let plus = IconButton::new(Symbol::Plus, "Nuevo");
                if sidebar {
                    let detail = rect()
                        .expanded()
                        .direction(Direction::Vertical)
                        .child(
                            Titlebar::new()
                                .title("Documentos")
                                .with_sidebar(true)
                                .left(bar_left())
                                .right(plus),
                        )
                        .child(form);
                    SplitView::new(
                        Sidebar::new()
                            .traffic_lights(true)
                            .child(
                                SidebarItem::new("Documentos")
                                    .symbol(Symbol::SquareGrid, tint(0))
                                    .selected(true),
                            )
                            .child(SidebarItem::new("Recientes").symbol(Symbol::Clock, tint(4))),
                        detail,
                    )
                    .into_element()
                } else {
                    rect()
                        .expanded()
                        .direction(Direction::Vertical)
                        .child(Titlebar::new().title("notas.txt").right(plus))
                        .child(form)
                        .into_element()
                }
            }
            Scene::Design { state } => {
                let detail =
                    rect()
                        .expanded()
                        .direction(Direction::Vertical)
                        .child(
                            Titlebar::new()
                                .title("Diseño")
                                .with_sidebar(true)
                                .left(bar_left()),
                        )
                        .child(
                            Form::new().section(
                                Section::titled("Ventana")
                                    .row(Row::new("Contenido que llega arriba").control(
                                        Switch::new("Contenido que llega arriba").on(true),
                                    ))
                                    .row(Row::value("Estado", state))
                                    .row(Row::navigation("Más", |_| {})),
                            ),
                        );
                SplitView::new(
                    Sidebar::new()
                        .traffic_lights(true)
                        .child(
                            SidebarItem::new("Diseño")
                                .symbol(Symbol::SquareGrid, tint(0))
                                .selected(true),
                        )
                        .child(
                            SidebarItem::new("Medidas").symbol(Symbol::SlidersVertical, tint(4)),
                        ),
                    detail,
                )
                .into_element()
            }
            Scene::Buttons { stripe, state } => {
                let label = if stripe {
                    "Contenido de la ventana"
                } else {
                    "Contenido que llega arriba"
                };
                let swatches = rect()
                    .width(Size::fill())
                    .height(Size::px(28.0))
                    .direction(Direction::Horizontal)
                    .content(torin::prelude::Content::flex())
                    .children(ACCENT_SWATCHES.iter().map(|(_, color)| {
                        rect()
                            .width(Size::flex(1.0))
                            .height(Size::fill())
                            .background(crate::theme::color(*color))
                            .into_element()
                    }));
                let detail = rect()
                    .expanded()
                    .direction(Direction::Vertical)
                    .child(
                        Titlebar::new()
                            .title("Botones")
                            .with_sidebar(true)
                            .left(bar_left()),
                    )
                    .maybe_child(stripe.then_some(swatches))
                    .child(
                        Form::new().section(
                            Section::titled("Ventana")
                                .row(Row::new(label).control(Switch::new(label).on(true)))
                                .row(Row::value("Estado", state)),
                        ),
                    );
                let names = [
                    "Botones",
                    "Interruptores",
                    "Elección",
                    "Deslizadores",
                    "Campos",
                    "Progreso",
                ];
                SplitView::new(
                    Sidebar::new()
                        .traffic_lights(true)
                        .children(names.iter().enumerate().map(|(i, name)| {
                            SidebarItem::new(*name)
                                .symbol(Symbol::SquareGrid, tint(i))
                                .selected(i == 0)
                        })),
                    detail,
                )
                .into_element()
            }
            Scene::Prefs => PreferencesWindow::new()
                .group(
                    PaneGroup::new(Some("Capa 1")).pane(Pane::new(
                        "botones",
                        "Botones",
                        Symbol::Plus,
                        tint(0),
                        Form::new()
                            .section(Section::titled("Botones").row(Row::value("Variantes", "—"))),
                    )),
                )
                .into_element(),
        };
        let mut chrome = WindowChrome::new(shot.style, content);
        if shot.items {
            chrome = chrome
                .rail_top(
                    rect()
                        .direction(Direction::Vertical)
                        .spacing(vow_design::TITLEBAR_RAIL.padding_bottom)
                        .child(IconButton::new(Symbol::Plus, "Nuevo"))
                        .child(IconButton::new(Symbol::MagnifyingGlass, "Buscar")),
                )
                .rail_bottom(
                    rect()
                        .direction(Direction::Vertical)
                        .spacing(vow_design::TITLEBAR_RAIL.padding_bottom)
                        .child(IconButton::new(Symbol::SquareGrid, "Diseño"))
                        .child(IconButton::new(Symbol::SlidersVertical, "Medidas")),
                );
        }
        let compositor_bar = shot.compositor_bar.then(|| {
            rect()
                .position(Position::new_absolute().top(0.0).left(0.0))
                .width(Size::percent(100.0))
                .height(Size::px(
                    *window_controls(shot.status).focus_bar_height.peek(),
                ))
                .background(crate::theme::color(vow_design::FOCUS_BAR_BACKGROUND))
                .layer(i16::MAX)
        });
        rect()
            .expanded()
            .child(crate::chrome::window_root(chrome))
            .maybe_child(compositor_bar)
    }
}

fn render(shot: Shot, dir: &std::path::Path) {
    let resolved = resolve(
        &AppearanceConfig {
            sidebar_style: if shot.flat {
                SidebarStyle::Flat
            } else {
                SidebarStyle::Island
            },
            ..Default::default()
        },
        None,
    );
    let bar = std::rc::Rc::new(std::cell::RefCell::new(None));
    let shared = bar.clone();
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Case(shot)),
        Size2D::new(shot.size.0, shot.size.1),
        move |runner| {
            let shared = shared.clone();
            runner.provide_root_context(move || {
                let controls = WindowControls {
                    focus_overlay: shot.overlay,
                    fullscreen_bar: State::create(shot.bar),
                    fullscreen_push: State::create(shot.push),
                    ..window_controls(shot.status)
                };
                *shared.borrow_mut() = Some(controls.focus_bar);
                controls
            });
            runner.provide_root_context(move || State::create(resolved));
        },
        1.0,
    );
    test.sync_and_update();
    test.poll(REVEAL_STEP, REVEAL_TIME);
    if shot.bar_down {
        bar.borrow_mut().as_mut().unwrap().set(true);
        test.sync_and_update();
        test.poll(REVEAL_STEP, REVEAL_TIME);
    }
    test.sync_and_update();
    test.render_to_file(dir.join(format!("{}.png", shot.name)));
}

#[test]
fn doc_shots() {
    let Some(dir) = std::env::var_os("BARRIER_KIT_SHOTS") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    let normal = WindowStatus::default();
    let inactive = WindowStatus {
        activated: false,
        ..normal
    };
    let maximized = WindowStatus {
        maximized: true,
        ..normal
    };
    let full = WindowStatus {
        fullscreen: true,
        ..normal
    };
    let top = ChromeStyle::default();
    let rail = ChromeStyle {
        layout: TitlebarLayout::Rail,
        ..top
    };
    let unified = ChromeStyle {
        rail_style: RailStyle::Unified,
        ..rail
    };
    let base = Shot {
        name: "",
        size: (880.0, 440.0),
        scene: Scene::Buttons {
            stripe: true,
            state: "Pantalla completa",
        },
        style: top,
        items: false,
        status: full,
        overlay: true,
        bar_down: true,
        bar: FullscreenBar::Solid,
        push: false,
        flat: false,
        compositor_bar: false,
    };
    let docs = Shot {
        size: (780.0, 260.0),
        scene: Scene::Documents { sidebar: true },
        status: normal,
        overlay: false,
        bar_down: false,
        ..base
    };
    let design = Shot {
        size: (880.0, 560.0),
        scene: Scene::Design { state: "Normal" },
        status: normal,
        overlay: false,
        bar_down: false,
        ..base
    };
    let shots = [
        Shot {
            name: "titlebar-island",
            ..docs
        },
        Shot {
            name: "titlebar-island-inactive",
            status: inactive,
            ..docs
        },
        Shot {
            name: "titlebar-flat",
            flat: true,
            ..docs
        },
        Shot {
            name: "titlebar-no-bar",
            scene: Scene::Documents { sidebar: false },
            ..docs
        },
        Shot {
            name: "titlebar-no-bar-inactive",
            scene: Scene::Documents { sidebar: false },
            status: inactive,
            ..docs
        },
        Shot {
            name: "state-normal",
            ..design
        },
        Shot {
            name: "state-maximized",
            scene: Scene::Design {
                state: "Maximizada",
            },
            status: maximized,
            ..design
        },
        Shot {
            name: "state-fullscreen-no-protocol",
            scene: Scene::Design {
                state: "Pantalla completa",
            },
            status: full,
            ..design
        },
        Shot {
            name: "state-focus-bar-top",
            scene: Scene::Design {
                state: "Pantalla completa",
            },
            status: full,
            overlay: true,
            ..design
        },
        Shot {
            name: "focus-top",
            compositor_bar: true,
            scene: Scene::Design {
                state: "Pantalla completa",
            },
            status: full,
            overlay: true,
            bar_down: true,
            ..design
        },
        Shot {
            name: "rail-items",
            style: rail,
            items: true,
            ..design
        },
        Shot {
            name: "rail-unified-items",
            style: unified,
            items: true,
            ..design
        },
        Shot {
            name: "bar-solid",
            ..base
        },
        Shot {
            name: "bar-glass-band",
            bar: FullscreenBar::Glass,
            ..base
        },
        Shot {
            name: "bar-progressive-glass",
            bar: FullscreenBar::ProgressiveGlass,
            ..base
        },
        Shot {
            name: "bar-push",
            push: true,
            ..base
        },
        Shot {
            name: "bar-push-progressive-glass",
            push: true,
            bar: FullscreenBar::ProgressiveGlass,
            ..base
        },
        Shot {
            name: "bar-push-top",
            push: true,
            bar_down: false,
            ..base
        },
        Shot {
            name: "rail-bar-solid",
            style: rail,
            ..base
        },
        Shot {
            name: "rail-bar-glass-band",
            style: rail,
            bar: FullscreenBar::Glass,
            ..base
        },
        Shot {
            name: "rail-bar-progressive-glass",
            style: rail,
            bar: FullscreenBar::ProgressiveGlass,
            ..base
        },
        Shot {
            name: "rail-bar-push",
            style: rail,
            push: true,
            ..base
        },
        Shot {
            name: "focus-unified",
            compositor_bar: true,
            style: unified,
            ..base
        },
        Shot {
            name: "focus-rail-items",
            compositor_bar: true,
            style: rail,
            items: true,
            ..base
        },
        Shot {
            name: "island-rail-focus",
            compositor_bar: true,
            style: rail,
            ..base
        },
        Shot {
            name: "island-rail-maximized",
            size: (880.0, 420.0),
            style: rail,
            scene: Scene::Buttons {
                stripe: false,
                state: "Maximizada",
            },
            status: maximized,
            overlay: false,
            bar_down: false,
            ..base
        },
        Shot {
            name: "sidebar-top",
            size: (980.0, 200.0),
            scene: Scene::Prefs,
            status: normal,
            overlay: false,
            bar_down: false,
            ..base
        },
        Shot {
            name: "sidebar-rail",
            size: (980.0, 200.0),
            scene: Scene::Prefs,
            style: rail,
            status: normal,
            overlay: false,
            bar_down: false,
            ..base
        },
    ];
    for shot in shots {
        render(shot, &dir);
    }
}

//! Master Tengen App Show: every BarrierKit component on its own page (ADR 0047 §6).

use barrier_kit::adornments::{Link, avatar, badge, divider};
use barrier_kit::app::{Launch, launch};
use barrier_kit::choices::{ColorWell, RadioGroup};
use barrier_kit::compact::{IconButton, Stepper, ToggleGroup};
use barrier_kit::controls::{
    Button, ButtonStyle, Checkbox, SearchField, SegmentedControl, Slider, Switch, TextField,
};
use barrier_kit::level::{LevelSlider, LevelStyle};
use barrier_kit::patterns::about::about_window;
use barrier_kit::patterns::preferences::{Pane, PaneGroup, PreferencesWindow};
use barrier_kit::platform::{Decorations, WindowOptions, run_window};
use barrier_kit::popup_button::PopUpButton;
use barrier_kit::progress::{ProgressBar, ProgressRing};
use barrier_kit::symbol::Symbol;
use barrier_kit::views::collection::CollectionView;
use barrier_kit::views::feedback::{Alert, empty_state};
use barrier_kit::views::form::{Form, Row, Section};
use barrier_kit::views::table::{SortOrder, Table};
use barrier_kit::views::tabs::TabView;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{Direction, Size};
use vow_design::{ACCENT_SWATCHES, COLLECTION, Rgba, SPACE_2, SPLIT_VIEW, TABLE};

const APP_ID: &str = "org.malaysia.tengen-app-show";

fn tint(index: usize) -> Rgba {
    ACCENT_SWATCHES[index % ACCENT_SWATCHES.len()].1
}

fn row_of(children: impl IntoIterator<Item = Element>) -> Rect {
    rect()
        .direction(Direction::Horizontal)
        .spacing(SPACE_2)
        .children(children)
}

fn buttons() -> Form {
    Form::new().section(
        Section::titled("Botones")
            .row(
                Row::new("Variantes").control(row_of([
                    Button::new("Cancelar").into_element(),
                    Button::new("Guardar")
                        .style(ButtonStyle::Primary)
                        .into_element(),
                    Button::new("Inactivo").disabled(true).into_element(),
                ])),
            )
            .row(Row::new("Botón de icono").control(row_of([
                IconButton::new(Symbol::Plus, "Añadir").into_element(),
                IconButton::new(Symbol::Minus, "Quitar").into_element(),
            ])))
            .row(Row::new("Enlace").control(Link::new("Más información…"))),
    )
}

fn toggles() -> Form {
    Form::new().section(
        Section::titled("Interruptores y casillas")
            .row(
                Row::new("Interruptor").control(row_of([
                    Switch::new("Apagado").into_element(),
                    Switch::new("Encendido").on(true).into_element(),
                    Switch::new("Inactivo")
                        .on(true)
                        .disabled(true)
                        .into_element(),
                ])),
            )
            .row(Row::new("Casilla").control(Checkbox::new("Recordarme").checked(true)))
            .row(
                Row::new("Radio").control(
                    RadioGroup::new("Alcance")
                        .option("Solo las ventanas")
                        .option("Todo el escritorio"),
                ),
            ),
    )
}

fn choices() -> Form {
    Form::new().section(
        Section::titled("Elección")
            .row(
                Row::new("Segmentado").control(
                    SegmentedControl::new("Periodo")
                        .segment("Día")
                        .segment("Semana")
                        .segment("Mes"),
                ),
            )
            .row(
                Row::new("Desplegable").control(
                    PopUpButton::new("Semáforos")
                        .item("Automático")
                        .item("Clásico")
                        .item("Acento"),
                ),
            )
            .row(
                Row::new("Color de acento")
                    .control(ColorWell::new("Color de acento", &ACCENT_SWATCHES).selected(Some(0))),
            )
            .row(
                Row::new("Grupo conmutable").control(
                    ToggleGroup::new("Formato")
                        .item("B", "Negrita", true)
                        .item("I", "Cursiva", false)
                        .item("U", "Subrayado", false),
                ),
            )
            .row(Row::new("Stepper").control(Stepper::new("Núcleos", 1, 16).value(4))),
    )
}

fn sliders() -> Form {
    Form::new().section(
        Section::titled("Deslizadores")
            .row(
                Row::new("Por pasos").control(
                    Slider::stepped(4)
                        .value(1)
                        .labels(&["Nada", "Poco", "Medio", "Mucho"]),
                ),
            )
            .row(
                Row::new("Continuo").control(
                    rect()
                        .width(Size::px(SPLIT_VIEW.inspector))
                        .child(LevelSlider::new("Brillo").value(0.4)),
                ),
            )
            .row(
                Row::new("Centro de control").control(
                    rect().width(Size::px(SPLIT_VIEW.inspector)).child(
                        LevelSlider::new("Volumen")
                            .value(0.6)
                            .style(LevelStyle::Fat(Symbol::SpeakerWave)),
                    ),
                ),
            ),
    )
}

fn fields(name: State<String>, query: State<String>) -> Form {
    Form::new().section(
        Section::titled("Campos")
            .row(
                Row::new("Texto").control(
                    rect()
                        .width(Size::px(SPLIT_VIEW.inspector))
                        .child(TextField::new(name, "Nombre de usuario")),
                ),
            )
            .row(
                Row::new("Búsqueda").control(
                    rect()
                        .width(Size::px(SPLIT_VIEW.inspector))
                        .child(SearchField::new(query, "Buscar")),
                ),
            ),
    )
}

fn progress() -> Form {
    Form::new().section(
        Section::titled("Progreso")
            .row(
                Row::new("Barra").control(
                    rect()
                        .width(Size::px(SPLIT_VIEW.inspector))
                        .child(ProgressBar::determinate("Copiando", 0.62)),
                ),
            )
            .row(
                Row::new("Indeterminado").control(
                    rect()
                        .width(Size::px(SPLIT_VIEW.inspector))
                        .child(ProgressBar::indeterminate("Buscando")),
                ),
            )
            .row(Row::new("Anillo").control(ProgressRing::new("Descargando", 0.4))),
    )
}

fn adornments() -> Form {
    Form::new().section(
        Section::titled("Insignias y adornos")
            .row(Row::new("Insignia").control(badge("2")))
            .row(Row::new("Avatar").control(avatar("HB")))
            .row(
                Row::new("Separador").control(
                    rect()
                        .width(Size::px(SPLIT_VIEW.inspector))
                        .child(divider()),
                ),
            ),
    )
}

fn forms() -> Form {
    Form::new()
        .section(
            Section::titled("Filas")
                .row(
                    Row::new("Con control")
                        .detail("Detalle debajo")
                        .control(Switch::new("Demo")),
                )
                .row(Row::value("De valor", "Normal · 87 %"))
                .row(Row::navigation("De navegación", |_| {}))
                .row(Row::destructive("Destructiva…", |_| {})),
        )
        .section(
            Section::new()
                .row(Row::value("Sección sin título", ""))
                .footer("Pie de sección."),
        )
}

fn data(sort: State<Option<SortOrder>>, selected: State<Vec<usize>>) -> Form {
    let mut sort_write = sort;
    let mut selected_write = selected;
    let mut files = [
        ("adr-0047.pdf", "248 KB"),
        ("notas.txt", "4 KB"),
        ("portada.png", "2,3 MB"),
        ("roadmap.pdf", "132 KB"),
    ];
    if let Some(order) = *sort.read() {
        files.sort_by(|a, b| {
            let ordering = if order.column == 0 {
                a.0.cmp(b.0)
            } else {
                a.1.cmp(b.1)
            };
            if order.ascending {
                ordering
            } else {
                ordering.reverse()
            }
        });
    }
    let rows = files
        .iter()
        .map(|(name, size)| vec![name.to_string(), size.to_string()])
        .collect();
    let folder = |index: usize| {
        let [r, g, b, a] = tint(index).to_rgba8();
        rect()
            .width(Size::px(COLLECTION.icon))
            .height(Size::px(COLLECTION.icon))
            .corner_radius(COLLECTION.radius)
            .background(Color::from_argb(a, r, g, b))
    };
    Form::new()
        .section(
            Section::titled("Tabla").row(
                Row::new("Archivos").control(
                    rect()
                        .width(Size::px(SPLIT_VIEW.sidebar_max))
                        .height(Size::px(TABLE.header_height + 4.0 * TABLE.row_height))
                        .child(
                            Table::new("Archivos", vec!["Nombre", "Tamaño"])
                                .rows(rows)
                                .sort(*sort.read())
                                .on_sort(move |order| sort_write.set(Some(order))),
                        ),
                ),
            ),
        )
        .section(
            Section::titled("Rejilla").row(
                Row::new("Carpetas").control(
                    CollectionView::new("Carpetas")
                        .item(folder(0), "Malaysia")
                        .item(folder(1), "Facturas")
                        .item(folder(6), "Fotos")
                        .selected(selected.read().clone())
                        .on_select(move |items| selected_write.set(items)),
                ),
            ),
        )
}

fn tabs() -> Form {
    Form::new().section(
        Section::titled("Pestañas").row(
            Row::new("Barra de herramientas").control(
                rect()
                    .width(Size::px(SPLIT_VIEW.sidebar_max))
                    .height(Size::px(SPLIT_VIEW.inspector / 2.0))
                    .child(
                        TabView::new("Secciones")
                            .tab(
                                Symbol::MagnifyingGlass,
                                "Buscar",
                                label().text("Página Buscar"),
                            )
                            .tab(Symbol::SpeakerWave, "Sonido", label().text("Página Sonido")),
                    ),
            ),
        ),
    )
}

fn feedback(mut alert: State<bool>) -> Form {
    Form::new().section(
        Section::titled("Estados y alertas")
            .row(Row::new("Vacío").control(empty_state(Symbol::MagnifyingGlass, "Sin resultados")))
            .row(
                Row::new("Alerta")
                    .control(Button::new("Mostrar alerta").on_press(move |_| alert.set(true))),
            ),
    )
}

fn about() -> Form {
    Form::new().section(
        Section::titled("Acerca de").row(
            Row::new("Ventana").control(
                rect()
                    .width(Size::px(SPLIT_VIEW.sidebar_max))
                    .height(Size::px(SPLIT_VIEW.inspector))
                    .child(about_window(
                        avatar("MT"),
                        "Master Tengen App Show",
                        "Versión 0.1",
                        "™ y © 2026 MalaysiaDreams",
                    )),
            ),
        ),
    )
}

struct Show;

impl App for Show {
    fn render(&self) -> impl IntoElement {
        let name = use_state(|| "hblanqueto".to_string());
        let query = use_state(String::new);
        let sort = use_state(|| None::<SortOrder>);
        let selected = use_state(Vec::new);
        let mut alert = use_state(|| false);

        let window = PreferencesWindow::new()
            .restoration(APP_ID)
            .group(
                PaneGroup::new(Some("Capa 1 · Primitivas"))
                    .pane(Pane::new(
                        "botones",
                        "Botones",
                        Symbol::Plus,
                        tint(0),
                        buttons(),
                    ))
                    .pane(Pane::new(
                        "interruptores",
                        "Interruptores",
                        Symbol::ChevronRight,
                        tint(6),
                        toggles(),
                    ))
                    .pane(Pane::new(
                        "eleccion",
                        "Elección",
                        Symbol::ChevronUpDown,
                        tint(1),
                        choices(),
                    ))
                    .pane(Pane::new(
                        "deslizadores",
                        "Deslizadores",
                        Symbol::SpeakerWave,
                        tint(4),
                        sliders(),
                    ))
                    .pane(Pane::new(
                        "campos",
                        "Campos",
                        Symbol::MagnifyingGlass,
                        tint(7),
                        fields(name, query),
                    ))
                    .pane(Pane::new(
                        "progreso",
                        "Progreso",
                        Symbol::ChevronDown,
                        tint(5),
                        progress(),
                    ))
                    .pane(Pane::new(
                        "adornos",
                        "Adornos",
                        Symbol::XMark,
                        tint(2),
                        adornments(),
                    )),
            )
            .group(
                PaneGroup::new(Some("Capa 2 · Vistas"))
                    .pane(Pane::new(
                        "formularios",
                        "Formularios",
                        Symbol::ChevronRight,
                        tint(0),
                        forms(),
                    ))
                    .pane(Pane::new(
                        "datos",
                        "Tabla y rejilla",
                        Symbol::ChevronDown,
                        tint(1),
                        data(sort, selected),
                    ))
                    .pane(Pane::new(
                        "pestanas",
                        "Pestañas",
                        Symbol::ChevronUp,
                        tint(3),
                        tabs(),
                    ))
                    .pane(Pane::new(
                        "estados",
                        "Estados y alertas",
                        Symbol::XMark,
                        tint(3),
                        feedback(alert),
                    )),
            )
            .group(PaneGroup::new(Some("Capa 3 · Patrones")).pane(Pane::new(
                "acerca",
                "Acerca de",
                Symbol::ChevronLeft,
                tint(7),
                about(),
            )));

        rect()
            .width(Size::fill())
            .height(Size::fill())
            .child(window)
            .maybe(*alert.read(), |el| {
                el.child(
                    Alert::new(
                        "¿Olvidar esta red?",
                        "Tendrás que volver a escribir la contraseña.",
                    )
                    .confirm("Olvidar", move |_| alert.set(false))
                    .cancel("Cancelar", move |_| alert.set(false)),
                )
            })
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    let _instance = match launch(APP_ID) {
        Ok(Launch::Forwarded) => return,
        Ok(Launch::Primary(connection)) => Some(connection),
        Err(error) => {
            tracing::warn!(%error, "tengen-app-show: sin instancia única");
            None
        }
    };
    run_window(
        AppComponent::new(Show),
        WindowOptions::new("Master Tengen App Show", "tengen-app-show")
            .app_id(APP_ID)
            .decorations(Decorations::Client)
            .size((980, 640)),
    )
    .expect("tengen-app-show");
}

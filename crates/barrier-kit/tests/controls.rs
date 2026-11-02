//! Golden-image tests for every control in light and dark.

use std::{
    fs,
    path::{Path, PathBuf},
};

use barrier_kit::adornments::{Link, avatar, badge, divider};
use barrier_kit::choices::{ColorWell, RadioGroup};
use barrier_kit::compact::{Face, IconButton, Stepper, ToggleGroup};
use barrier_kit::controls::{
    Button, ButtonStyle, Checkbox, Popover, ScrollView, SearchField, SecureField, SegmentedControl,
    Slider, Switch, TextField,
};
use barrier_kit::level::{LevelSlider, LevelStyle};
use barrier_kit::menu::MenuIcon;
use barrier_kit::patterns::about::about_window;
use barrier_kit::patterns::preferences::{Pane, PaneGroup, PreferencesWindow};
use barrier_kit::popup_button::PopUpButton;
use barrier_kit::progress::{ProgressBar, ProgressRing};
use barrier_kit::sidebar::SidebarAccount;
use barrier_kit::symbol::Symbol;
use barrier_kit::theme;
use barrier_kit::views::collection::CollectionView;
use barrier_kit::views::feedback::{Alert, empty_state};
use barrier_kit::views::form::{Form, Row, Section};
use barrier_kit::views::table::{SortOrder, Table};
use barrier_kit::views::tabs::TabView;
use freya_components::menu::{Menu, MenuItem};
use freya_components::theming::hooks::use_init_theme;
use freya_core::integration::{AppComponent, NavigationMode, Platform};
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use keyboard_types::{Key, NamedKey};
use torin::prelude::{Alignment, Direction, Gaps, Size, Size2D};
use vow_design::ACCENT_SWATCHES;

const SIZE: (f32, f32) = (360.0, 240.0);
const WINDOW_SIZE: (f32, f32) = (900.0, 560.0);
const TOLERANCE: u8 = 3;

struct Case {
    dark: bool,
    build: fn() -> Element,
}

impl App for Case {
    fn render(&self) -> impl IntoElement {
        let dark = self.dark;
        use_init_theme(|| theme::for_scheme(dark));
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .direction(Direction::Vertical)
            .main_align(Alignment::Center)
            .cross_align(Alignment::Center)
            .padding(Gaps::new(24.0, 24.0, 24.0, 24.0))
            .child((self.build)())
    }
}

type Control = (&'static str, fn() -> Element);

fn cases() -> Vec<Control> {
    vec![
        ("button", button),
        ("switch", switch),
        ("slider", slider),
        ("segmented-control", segmented_control),
        ("text-field", text_field),
        ("checkbox", checkbox),
        ("radio-group", radio_group),
        ("color-well", color_well),
        ("progress", progress),
        ("level-slider", level_slider),
        ("compact", compact),
        ("adornments", adornments),
        ("popup-button", popup_button),
        ("popup-button-icon", popup_button_icon),
        ("form", form),
        ("tab-view", tab_view),
        ("table", table),
        ("collection", collection),
        ("empty-state", empty),
        ("alert", alert),
        ("sidebar-account", sidebar_account),
        ("secure-field", secure_field),
        ("scroll-view", scroll_view),
        ("menu", menu),
        ("popover", popover),
    ]
}

fn button() -> Element {
    rect()
        .direction(Direction::Horizontal)
        .spacing(12.0)
        .child(Button::new("Cancelar"))
        .child(Button::new("Guardar").style(ButtonStyle::Primary))
        .child(Button::new("Borrar").disabled(true))
        .into_element()
}

fn switch() -> Element {
    rect()
        .direction(Direction::Horizontal)
        .spacing(16.0)
        .child(Switch::new("Apagado"))
        .child(Switch::new("Encendido").on(true))
        .child(Switch::new("Inactivo").on(true).disabled(true))
        .into_element()
}

fn slider() -> Element {
    Slider::stepped(5)
        .value(2)
        .labels(&["A", "B", "C", "D", "E"])
        .into_element()
}

fn segmented_control() -> Element {
    SegmentedControl::new("Periodo")
        .segment("Día")
        .segment("Semana")
        .segment("Mes")
        .selected(1)
        .into_element()
}

fn radio_group() -> Element {
    rect()
        .direction(Direction::Vertical)
        .spacing(12.0)
        .child(
            RadioGroup::new("Aplicar el tinte a")
                .option("Solo las ventanas")
                .option("Todo el escritorio"),
        )
        .child(
            RadioGroup::new("Inactivo")
                .option("Uno")
                .option("Dos")
                .selected(1)
                .disabled(true),
        )
        .into_element()
}

fn color_well() -> Element {
    ColorWell::new("Color de acento", &ACCENT_SWATCHES)
        .selected(Some(0))
        .into_element()
}

fn progress() -> Element {
    rect()
        .width(Size::px(240.0))
        .direction(Direction::Vertical)
        .spacing(16.0)
        .child(ProgressBar::determinate("Copiando sistema", 0.62))
        .child(ProgressBar::indeterminate("Buscando redes"))
        .child(ProgressRing::new("Descargando", 0.4))
        .into_element()
}

fn level_slider() -> Element {
    rect()
        .width(Size::px(240.0))
        .direction(Direction::Vertical)
        .spacing(16.0)
        .child(LevelSlider::new("Brillo").value(0.3))
        .child(
            LevelSlider::new("Volumen")
                .value(0.6)
                .style(LevelStyle::Fat(Symbol::SpeakerWave)),
        )
        .into_element()
}

fn compact() -> Element {
    rect()
        .direction(Direction::Vertical)
        .spacing(12.0)
        .child(IconButton::new(Symbol::Plus, "Añadir"))
        .child(Stepper::new("Núcleos", 1, 16).value(4))
        .child(
            ToggleGroup::new("Formato")
                .item("B", Face::Bold, "Negrita", true)
                .item("I", Face::Italic, "Cursiva", false)
                .item("U", Face::Underline, "Subrayado", false),
        )
        .into_element()
}

fn adornments() -> Element {
    rect()
        .width(Size::px(200.0))
        .direction(Direction::Vertical)
        .spacing(12.0)
        .child(badge("2"))
        .child(divider())
        .child(avatar("HB"))
        .child(Link::new("Más información…"))
        .into_element()
}

fn popup_button() -> Element {
    PopUpButton::new("Semáforos")
        .item("Automático")
        .item("Clásico")
        .item("Acento")
        .selected(1)
        .into_element()
}

fn form() -> Element {
    Form::new()
        .section(
            Section::titled("Ventanas")
                .row(Row::new("Ampliación").control(Switch::new("Ampliación").on(true)))
                .row(Row::value("Estado", "Normal · 87 %"))
                .row(Row::navigation("Avanzado", |_| {})),
        )
        .highlighted(Some("Estado"))
        .into_element()
}

fn tab_view() -> Element {
    TabView::new("Preferencias")
        .tab(Symbol::MagnifyingGlass, "Buscar", label().text("Buscar"))
        .tab(Symbol::SpeakerWave, "Sonido", label().text("Sonido"))
        .into_element()
}

fn table() -> Element {
    let row = |name: &str, size: &str| vec![name.to_string(), size.to_string()];
    Table::new("Archivos", vec!["Nombre", "Tamaño"])
        .rows(vec![
            row("adr-0047.pdf", "248 KB"),
            row("notas.txt", "4 KB"),
            row("portada.png", "2,3 MB"),
        ])
        .sort(Some(SortOrder {
            column: 0,
            ascending: true,
        }))
        .selected(Some(1))
        .into_element()
}

fn collection() -> Element {
    let icon = || {
        rect()
            .width(Size::px(40.0))
            .height(Size::px(32.0))
            .background((90, 163, 240))
    };
    CollectionView::new("Documentos")
        .item(icon(), "Malaysia")
        .item(icon(), "Facturas")
        .item(icon(), "Fotos")
        .selected(vec![1])
        .into_element()
}

fn empty() -> Element {
    empty_state(Symbol::MagnifyingGlass, "Sin resultados").into_element()
}

fn alert() -> Element {
    Alert::new(
        "¿Olvidar esta red?",
        "Tendrás que volver a escribir la contraseña.",
    )
    .confirm("Olvidar", |_| {})
    .into_element()
}

fn sidebar_account() -> Element {
    rect()
        .width(Size::px(220.0))
        .child(SidebarAccount::new("HB", "Héctor Blanqueto", "Cuenta local").selected(true))
        .into_element()
}

fn window_cases() -> Vec<Control> {
    vec![("preferences", preferences), ("about", about)]
}

fn preferences() -> Element {
    let [blue, purple, ..] = ACCENT_SWATCHES.map(|swatch| swatch.1);
    PreferencesWindow::new()
        .account("HB", "Héctor Blanqueto", "Cuenta local")
        .group(
            PaneGroup::new(None)
                .pane(Pane::new(
                    "apariencia",
                    "Apariencia",
                    Symbol::MagnifyingGlass,
                    blue,
                    Form::new().section(
                        Section::new()
                            .row(
                                Row::new("Color de acento").control(
                                    ColorWell::new("Color de acento", &ACCENT_SWATCHES)
                                        .selected(Some(0)),
                                ),
                            )
                            .row(
                                Row::new("Semáforos").control(
                                    PopUpButton::new("Semáforos")
                                        .item("Automático")
                                        .item("Clásico"),
                                ),
                            ),
                    ),
                ))
                .pane(Pane::new(
                    "sonido",
                    "Sonido",
                    Symbol::SpeakerWave,
                    purple,
                    Form::new(),
                )),
        )
        .into_element()
}

fn about() -> Element {
    about_window(
        avatar("C"),
        "Contador",
        "Versión 1.0",
        "™ y © 2026 MalaysiaDreams",
    )
    .into_element()
}

fn popup_button_icon() -> Element {
    let dots = r##"<svg viewBox="0 0 16 16"><circle cx="4" cy="8" r="3" fill="#ff5f57"/><circle cx="12" cy="8" r="3" fill="#28c840"/></svg>"##;
    PopUpButton::new("Semáforos")
        .icon_item("Clásico", MenuIcon::svg(dots))
        .item("Acento")
        .into_element()
}

fn checkbox() -> Element {
    Checkbox::new("Recordarme").checked(true).into_element()
}

fn text_field() -> Element {
    let name = use_state(|| "hblanqueto".to_string());
    let empty = use_state(String::new);
    let query = use_state(|| "wifi".to_string());
    rect()
        .width(Size::px(240.0))
        .direction(Direction::Vertical)
        .spacing(10.0)
        .child(TextField::new(name, "Nombre de usuario"))
        .child(TextField::new(empty, "Nombre de usuario"))
        .child(SearchField::new(query, "Buscar"))
        .into_element()
}

fn secure_field() -> Element {
    SecureField::new().into_element()
}

fn scroll_view() -> Element {
    ScrollView::new()
        .width(Size::px(200.0))
        .height(Size::px(120.0))
        .children([
            label().text("Uno").into_element(),
            label().text("Dos").into_element(),
            label().text("Tres").into_element(),
        ])
        .into_element()
}

fn menu() -> Element {
    Menu::new()
        .children([
            MenuItem::new().child("Abrir").into_element(),
            MenuItem::new().child("Guardar").into_element(),
        ])
        .into_element()
}

fn popover() -> Element {
    Popover::new()
        .children([
            label().text("Popover").into_element(),
            Button::new("Aceptar").into_element(),
        ])
        .into_element()
}

fn golden_path(name: &str, dark: bool) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(format!(
            "{name}-{}.png",
            if dark { "dark" } else { "light" }
        ))
}

fn render(size: (f32, f32), dark: bool, build: fn() -> Element) -> Vec<u8> {
    let app = AppComponent::new(Case { dark, build });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(size.0, size.1), |_| {}, 1.0);
    test.sync_and_update();
    test.sync_and_update();
    let data = test.render();
    data.as_ref().to_vec()
}

fn decode(path: &Path) -> (u32, u32, Vec<u8>) {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().unwrap();
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).unwrap();
    let rgba = match info.color_type {
        png::ColorType::Rgba => buffer[..info.buffer_size()].to_vec(),
        png::ColorType::Rgb => buffer[..info.buffer_size()]
            .chunks(3)
            .flat_map(|px| [px[0], px[1], px[2], 255])
            .collect(),
        other => panic!("unexpected PNG colour type {other:?}"),
    };
    (info.width, info.height, rgba)
}

fn compare(golden: &Path, actual_png: &[u8], name: &str, dark: bool) {
    let actual_path = std::env::temp_dir().join(format!(
        "malaysia-control-{name}-{}.png",
        if dark { "dark" } else { "light" }
    ));
    fs::write(&actual_path, actual_png).unwrap();
    let (gw, gh, golden_px) = decode(golden);
    let (aw, ah, actual_px) = decode(&actual_path);
    assert_eq!((gw, gh), (aw, ah), "{name}: image size changed");
    let bad = golden_px
        .iter()
        .zip(&actual_px)
        .filter(|(a, b)| a.abs_diff(**b) > TOLERANCE)
        .count();
    assert_eq!(
        bad,
        0,
        "{name} ({}): {bad} channels differ by more than {TOLERANCE}; actual at {actual_path:?}",
        if dark { "dark" } else { "light" }
    );
}

#[test]
fn controls_match_their_golden_images() {
    let update = std::env::var_os("UPDATE_GOLDEN").is_some();
    let controls = cases().into_iter().map(|(name, build)| (name, build, SIZE));
    let windows = window_cases()
        .into_iter()
        .map(|(name, build)| (name, build, WINDOW_SIZE));
    for (name, build, size) in controls.chain(windows) {
        for dark in [false, true] {
            let path = golden_path(name, dark);
            let actual = render(size, dark, build);
            if update || !path.exists() {
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(&path, &actual).unwrap();
            } else {
                compare(&path, &actual, name, dark);
            }
        }
    }
}

#[test]
fn slider_click_snaps_to_the_nearest_step() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let values: Rc<RefCell<Vec<usize>>> = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    struct SliderCase {
        values: Rc<RefCell<Vec<usize>>>,
    }
    impl App for SliderCase {
        fn render(&self) -> impl IntoElement {
            let values = self.values.clone();
            Slider::stepped(4)
                .value(1)
                .labels(&["Sutil", "Normal", "Fuerte", "Intenso"])
                .on_change(move |v| values.borrow_mut().push(v))
        }
    }

    let app = AppComponent::new(SliderCase {
        values: values_clone,
    });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(240.0, 80.0), |_| {}, 1.0);
    test.sync_and_update();

    test.click_cursor((190.0, 20.0));
    test.sync_and_update();
    assert_eq!(values.borrow().as_slice(), &[3]);

    test.click_cursor((130.0, 20.0));
    test.sync_and_update();
    assert_eq!(values.borrow().as_slice(), &[3, 2]);
}

#[test]
fn disabled_slider_does_not_emit() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let values: Rc<RefCell<Vec<usize>>> = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    struct SliderCase {
        values: Rc<RefCell<Vec<usize>>>,
    }
    impl App for SliderCase {
        fn render(&self) -> impl IntoElement {
            let values = self.values.clone();
            Slider::stepped(4)
                .value(1)
                .disabled(true)
                .on_change(move |v| values.borrow_mut().push(v))
        }
    }

    let app = AppComponent::new(SliderCase {
        values: values_clone,
    });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(240.0, 80.0), |_| {}, 1.0);
    test.sync_and_update();

    test.click_cursor((190.0, 20.0));
    test.sync_and_update();
    assert!(values.borrow().is_empty());
}

#[test]
fn checkbox_click_toggles_and_disabled_does_not_emit() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let values: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    struct CheckboxCase {
        values: Rc<RefCell<Vec<bool>>>,
    }
    impl App for CheckboxCase {
        fn render(&self) -> impl IntoElement {
            let values = self.values.clone();
            Checkbox::new("Recordarme")
                .checked(false)
                .on_toggle(move |v| values.borrow_mut().push(v))
        }
    }

    let app = AppComponent::new(CheckboxCase {
        values: values_clone,
    });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(160.0, 60.0), |_| {}, 1.0);
    test.sync_and_update();

    test.click_cursor((8.0, 8.0));
    test.sync_and_update();
    assert_eq!(values.borrow().as_slice(), &[true]);

    test.click_cursor((8.0, 8.0));
    test.sync_and_update();
    assert_eq!(values.borrow().as_slice(), &[true, false]);
}

#[test]
fn switch_click_toggles_and_disabled_does_not_emit() {
    use std::cell::RefCell;
    use std::rc::Rc;

    struct SwitchCase {
        disabled: bool,
        values: Rc<RefCell<Vec<bool>>>,
    }
    impl App for SwitchCase {
        fn render(&self) -> impl IntoElement {
            let values = self.values.clone();
            Switch::new("Wi-Fi")
                .disabled(self.disabled)
                .on_toggle(move |v| values.borrow_mut().push(v))
        }
    }

    for (disabled, expected) in [(false, vec![true, false]), (true, vec![])] {
        let values = Rc::new(RefCell::new(Vec::new()));
        let app = AppComponent::new(SwitchCase {
            disabled,
            values: values.clone(),
        });
        let (mut test, _) = TestingRunner::new(app, Size2D::new(60.0, 30.0), |_| {}, 1.0);
        test.sync_and_update();
        for _ in 0..2 {
            test.click_cursor((10.0, 9.0));
            test.sync_and_update();
        }
        assert_eq!(*values.borrow(), expected);
    }
}

#[test]
fn segmented_control_click_selects_once_per_new_segment() {
    use std::cell::RefCell;
    use std::rc::Rc;

    struct SegmentedCase {
        values: Rc<RefCell<Vec<usize>>>,
    }
    impl App for SegmentedCase {
        fn render(&self) -> impl IntoElement {
            let values = self.values.clone();
            SegmentedControl::new("Vista")
                .segment("Cuadrícula")
                .segment("Lista")
                .on_change(move |i| values.borrow_mut().push(i))
        }
    }

    let values = Rc::new(RefCell::new(Vec::new()));
    let app = AppComponent::new(SegmentedCase {
        values: values.clone(),
    });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(200.0, 30.0), |_| {}, 1.0);
    test.sync_and_update();
    for x in [150.0, 150.0, 50.0] {
        test.click_cursor((x, 15.0));
        test.sync_and_update();
    }
    assert_eq!(*values.borrow(), [1, 0]);
}

struct FocusCase {
    keyboard: bool,
    switch: State<bool>,
    checkbox: State<bool>,
    segment: State<usize>,
}

impl App for FocusCase {
    fn render(&self) -> impl IntoElement {
        let (mut switch, mut checkbox, mut segment) = (self.switch, self.checkbox, self.segment);
        if self.keyboard {
            Platform::get()
                .navigation_mode
                .set_if_modified(NavigationMode::Keyboard);
        }
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .direction(Direction::Vertical)
            .main_align(Alignment::Center)
            .cross_align(Alignment::Center)
            .padding(Gaps::new(24.0, 24.0, 24.0, 24.0))
            .spacing(16.0)
            .child(Switch::new("Wi-Fi").on_toggle(move |v| switch.set(v)))
            .child(Checkbox::new("Recordarme").on_toggle(move |v| checkbox.set(v)))
            .child(
                SegmentedControl::new("Vista")
                    .segment("Cuadrícula")
                    .segment("Lista")
                    .on_change(move |i| segment.set(i)),
            )
    }
}

fn focus_case(keyboard: bool) -> (TestingRunner, State<bool>, State<bool>, State<usize>) {
    let switch = State::create_global(false);
    let checkbox = State::create_global(false);
    let segment = State::create_global(0);
    let app = AppComponent::new(FocusCase {
        keyboard,
        switch,
        checkbox,
        segment,
    });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(SIZE.0, SIZE.1), |_| {}, 1.0);
    test.sync_and_update();
    (test, switch, checkbox, segment)
}

fn press(test: &mut TestingRunner, key: Key) {
    test.press_key(key);
    test.sync_and_update();
}

#[test]
fn tab_reaches_each_control_and_space_activates_it() {
    let (mut test, switch, checkbox, segment) = focus_case(false);
    let space = || Key::Character(" ".into());

    press(&mut test, Key::Named(NamedKey::Tab));
    press(&mut test, space());
    press(&mut test, Key::Named(NamedKey::Tab));
    press(&mut test, space());
    press(&mut test, Key::Named(NamedKey::Tab));
    press(&mut test, Key::Named(NamedKey::Tab));
    press(&mut test, Key::Named(NamedKey::Enter));

    assert!(*switch.peek());
    assert!(*checkbox.peek());
    assert_eq!(*segment.peek(), 1);
}

#[test]
fn keyboard_focus_ring_matches_its_golden_image() {
    let (mut test, ..) = focus_case(true);
    press(&mut test, Key::Named(NamedKey::Tab));
    test.poll_n(std::time::Duration::from_millis(16), 3);
    let actual = test.render().as_ref().to_vec();
    let path = golden_path("focus-ring", false);
    if std::env::var_os("UPDATE_GOLDEN").is_some() || !path.exists() {
        fs::write(&path, &actual).unwrap();
    } else {
        compare(&path, &actual, "focus-ring", false);
    }
}

#[test]
fn button_press_emits_only_when_enabled() {
    use std::cell::RefCell;
    use std::rc::Rc;

    struct ButtonCase {
        disabled: bool,
        presses: Rc<RefCell<u32>>,
    }
    impl App for ButtonCase {
        fn render(&self) -> impl IntoElement {
            let presses = self.presses.clone();
            Button::new("Guardar")
                .disabled(self.disabled)
                .on_press(move |_| *presses.borrow_mut() += 1)
        }
    }

    for (disabled, expected) in [(false, 1), (true, 0)] {
        let presses = Rc::new(RefCell::new(0));
        let app = AppComponent::new(ButtonCase {
            disabled,
            presses: presses.clone(),
        });
        let (mut test, _) = TestingRunner::new(app, Size2D::new(120.0, 40.0), |_| {}, 1.0);
        test.sync_and_update();
        test.click_cursor((20.0, 15.0));
        test.sync_and_update();
        assert_eq!(*presses.borrow(), expected);
    }
}

#[test]
fn search_field_clear_button_empties_the_query() {
    struct SearchCase {
        query: State<String>,
    }
    impl App for SearchCase {
        fn render(&self) -> impl IntoElement {
            SearchField::new(self.query, "Buscar")
        }
    }

    let query = State::create_global("wifi".to_string());
    let app = AppComponent::new(SearchCase { query });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(200.0, 30.0), |_| {}, 1.0);
    test.sync_and_update();
    test.click_cursor((180.0, 15.0));
    test.sync_and_update();
    assert!(query.peek().is_empty());
}

#[test]
fn radio_group_click_selects_and_disabled_does_not_emit() {
    use std::cell::RefCell;
    use std::rc::Rc;

    struct RadioCase {
        disabled: bool,
        values: Rc<RefCell<Vec<usize>>>,
    }
    impl App for RadioCase {
        fn render(&self) -> impl IntoElement {
            let values = self.values.clone();
            RadioGroup::new("Alcance")
                .option("Ventanas")
                .option("Escritorio")
                .disabled(self.disabled)
                .on_change(move |i| values.borrow_mut().push(i))
        }
    }

    for (disabled, expected) in [(false, vec![1]), (true, vec![])] {
        let values = Rc::new(RefCell::new(Vec::new()));
        let app = AppComponent::new(RadioCase {
            disabled,
            values: values.clone(),
        });
        let (mut test, _) = TestingRunner::new(app, Size2D::new(300.0, 30.0), |_| {}, 1.0);
        test.sync_and_update();
        test.click_cursor((110.0, 9.0));
        test.sync_and_update();
        assert_eq!(*values.borrow(), expected);
    }
}

#[test]
fn level_slider_follows_clicks_and_arrow_keys() {
    let level = State::create_global(0.0f32);

    struct LevelCase {
        level: State<f32>,
    }
    impl App for LevelCase {
        fn render(&self) -> impl IntoElement {
            let mut level = self.level;
            LevelSlider::new("Volumen").on_change(move |v| level.set(v))
        }
    }

    let app = AppComponent::new(LevelCase { level });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(220.0, 20.0), |_| {}, 1.0);
    test.sync_and_update();
    test.sync_and_update();
    test.click_cursor((110.0, 10.0));
    test.sync_and_update();
    assert!((*level.peek() - 0.5).abs() < 0.01);

    press(&mut test, Key::Named(NamedKey::ArrowRight));
    assert!((*level.peek() - 0.55).abs() < 0.01);
}

#[test]
fn stepper_stops_at_its_bounds() {
    let value = State::create_global(0i64);

    struct StepperCase {
        value: State<i64>,
    }
    impl App for StepperCase {
        fn render(&self) -> impl IntoElement {
            let mut value = self.value;
            Stepper::new("Núcleos", 1, 2).on_change(move |v| value.set(v))
        }
    }

    let app = AppComponent::new(StepperCase { value });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(120.0, 30.0), |_| {}, 1.0);
    test.sync_and_update();
    for _ in 0..3 {
        test.click_cursor((75.0, 15.0));
        test.sync_and_update();
    }
    assert_eq!(*value.peek(), 2);
}

#[test]
fn toggle_group_flips_each_item_on_its_own() {
    use std::cell::RefCell;
    use std::rc::Rc;

    struct ToggleCase {
        events: Rc<RefCell<Vec<(usize, bool)>>>,
    }
    impl App for ToggleCase {
        fn render(&self) -> impl IntoElement {
            let events = self.events.clone();
            ToggleGroup::new("Formato")
                .item("B", Face::Bold, "Negrita", true)
                .item("I", Face::Italic, "Cursiva", false)
                .on_toggle(move |e| events.borrow_mut().push(e))
        }
    }

    let events = Rc::new(RefCell::new(Vec::new()));
    let app = AppComponent::new(ToggleCase {
        events: events.clone(),
    });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(120.0, 30.0), |_| {}, 1.0);
    test.sync_and_update();
    for x in [15.0, 47.0, 15.0] {
        test.click_cursor((x, 15.0));
        test.sync_and_update();
    }
    assert_eq!(*events.borrow(), [(0, false), (1, true), (0, true)]);
}

#[test]
fn disabled_checkbox_does_not_emit() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let values: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    struct CheckboxCase {
        values: Rc<RefCell<Vec<bool>>>,
    }
    impl App for CheckboxCase {
        fn render(&self) -> impl IntoElement {
            let values = self.values.clone();
            Checkbox::new("Recordarme")
                .checked(false)
                .disabled(true)
                .on_toggle(move |v| values.borrow_mut().push(v))
        }
    }

    let app = AppComponent::new(CheckboxCase {
        values: values_clone,
    });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(160.0, 60.0), |_| {}, 1.0);
    test.sync_and_update();

    test.click_cursor((8.0, 8.0));
    test.sync_and_update();
    assert!(values.borrow().is_empty());
}

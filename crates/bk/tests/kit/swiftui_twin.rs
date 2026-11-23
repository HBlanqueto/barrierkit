use std::{fs, path::PathBuf};

use bk::compact::IconButton;
use bk::controls::{TextField, ToolGroup};
use bk::elements::Label;
use bk::sidebar::{Sidebar, SidebarItem};
use bk::symbol::Symbol;
use bk::theme::{self, SurfaceKind};
use bk::titlebar::Titlebar;
use bk::views::feedback::ContentUnavailableView;
use bk::views::form::{Form, Row, Section};
use bk::views::list::List;
use bk::views::split::SplitView;
use ctappearance::{AppearanceConfig, ColorScheme, TrafficLightStyle, resolve};
use ctdesign::SANS_FAMILIES;
use freya_components::theming::hooks::use_init_theme;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Direction, Size, Size2D};

const SIZE: (f32, f32) = (760.0, 480.0);
const TOLERANCE: u8 = 8;

struct Element {
    name: &'static str,
    rect: (usize, usize, usize, usize),
    text: bool,
    ceiling: f32,
}

const fn element(
    name: &'static str,
    rect: (usize, usize, usize, usize),
    text: bool,
    ceiling: f32,
) -> Element {
    Element {
        name,
        rect,
        text,
        ceiling,
    }
}

const NOTES: [Element; 10] = [
    element("lights", (10, 10, 100, 42), false, 14.0),
    element("sidebar toggle", (170, 2, 218, 50), false, 24.0),
    element("sidebar selected row", (10, 52, 210, 84), true, 14.0),
    element("sidebar rows", (10, 84, 210, 152), true, 10.0),
    element("sidebar background", (0, 160, 220, 480), false, 1.0),
    element("title", (230, 10, 330, 42), true, 12.0),
    element("toolbar group", (670, 2, 758, 50), false, 21.0),
    element("title bar edge", (222, 48, 758, 56), false, 1.0),
    element("empty state", (300, 200, 700, 360), true, 6.0),
    element("detail background", (222, 56, 758, 200), false, 1.0),
];

const TASKS_ELEMENTS: [Element; 6] = [
    element("lights", (10, 10, 100, 42), false, 14.0),
    element("title", (100, 10, 220, 42), true, 12.0),
    element("toolbar group", (670, 2, 758, 50), false, 16.0),
    element("list rows", (0, 56, 760, 160), true, 8.0),
    element("title bar edge", (0, 48, 760, 56), false, 1.0),
    element("window background", (0, 170, 760, 480), false, 1.0),
];

const FORM_ELEMENTS: [Element; 7] = [
    element("heading", (28, 50, 140, 72), true, 18.0),
    element("card background", (300, 80, 600, 112), false, 1.0),
    element("row separators", (28, 113, 732, 116), false, 1.0),
    element("switch", (680, 82, 730, 110), false, 26.0),
    element("row labels", (28, 80, 200, 186), true, 6.0),
    element("row values", (560, 116, 732, 150), true, 8.0),
    element("text field row", (28, 152, 732, 186), true, 4.0),
];

struct FormTwin {
    dark: bool,
}

impl App for FormTwin {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(self.dark));
        let name = use_state(|| "Ada".to_string());
        rect()
            .expanded()
            .background(theme::color(theme::surface_background(
                &theme::resolved(),
                SurfaceKind::Toplevel,
            )))
            .direction(Direction::Vertical)
            .child(rect().height(Size::px(32.0)))
            .child(
                Form::new().section(
                    Section::titled("Windows")
                        .row(Row::toggle("Zoom", true, |_| {}))
                        .row(Row::value("Status", "Normal"))
                        .row(Row::new("Name").control(TextField::new(name, "Name"))),
                ),
            )
    }
}

struct NotesTwin {
    dark: bool,
}

const TASKS: [(&str, Symbol); 4] = [
    ("Buy milk", Symbol::House),
    ("Call Ada", Symbol::Phone),
    ("Write report", Symbol::DocText),
    ("Book flights", Symbol::PaperPlane),
];

struct TasksTwin {
    dark: bool,
}

impl App for TasksTwin {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(self.dark));
        rect()
            .expanded()
            .background(theme::color(theme::surface_background(
                &theme::resolved(),
                SurfaceKind::Toplevel,
            )))
            .direction(Direction::Vertical)
            .child(
                Titlebar::new().title("Tasks").right(
                    ToolGroup::new()
                        .child(IconButton::new(Symbol::Plus, "Add"))
                        .child(IconButton::new(Symbol::Ellipsis, "More")),
                ),
            )
            .child(
                List::new("Tasks", TASKS.map(|(title, _)| title.to_string()).to_vec())
                    .row(|index| Label::new(TASKS[index].0, TASKS[index].1)),
            )
    }
}

impl App for NotesTwin {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(self.dark));
        let item = |title: &'static str, glyph: Symbol| {
            SidebarItem::new(title)
                .symbol(glyph)
                .selected(title == "All Notes")
        };
        let detail = rect()
            .expanded()
            .direction(Direction::Vertical)
            .child(
                Titlebar::new().title("Notes").with_sidebar(true).right(
                    ToolGroup::new()
                        .child(IconButton::new(Symbol::SquareAndPencil, "New note"))
                        .child(IconButton::new(Symbol::Ellipsis, "More")),
                ),
            )
            .child(ContentUnavailableView::new(
                "No Note Selected",
                Symbol::DocText,
            ));
        rect()
            .expanded()
            .background(theme::color(theme::surface_background(
                &theme::resolved(),
                SurfaceKind::Toplevel,
            )))
            .child(
                SplitView::new(
                    Sidebar::new()
                        .traffic_lights(true)
                        .child(item("All Notes", Symbol::DocText))
                        .child(item("Recent", Symbol::Clock))
                        .child(item("Starred", Symbol::Star)),
                    detail,
                )
                .sidebar_width(220.0),
            )
    }
}

fn render(app: AppComponent, dark: bool) -> Vec<u8> {
    render_at(app, dark, 1.0)
}

fn render_at(app: AppComponent, dark: bool, scale: f64) -> Vec<u8> {
    let resolved = resolve(
        &AppearanceConfig {
            color_scheme: if dark {
                ColorScheme::Dark
            } else {
                ColorScheme::Light
            },
            reduced_motion: true,
            traffic_lights: TrafficLightStyle::Standard,
            ..Default::default()
        },
        None,
    );
    let (mut test, _) = TestingRunner::new(
        app,
        Size2D::new(SIZE.0 * scale as f32, SIZE.1 * scale as f32),
        move |runner| {
            runner.provide_root_context(move || State::create(resolved));
        },
        scale,
    );
    test.set_default_fonts(&SANS_FAMILIES.map(std::borrow::Cow::Borrowed));
    test.sync_and_update();
    test.sync_and_update();
    test.render().as_bytes().to_vec()
}

fn reference(name: &str, dark: bool) -> (usize, usize, Vec<u8>) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/reference")
        .join(format!(
            "swiftui-{name}-{}.png",
            if dark { "dark" } else { "light" }
        ));
    crate::image::decode(&fs::read(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}")))
}

fn percent_different(
    reference: &[u8],
    actual: &[u8],
    width: usize,
    (x0, y0, x1, y1): (usize, usize, usize, usize),
) -> f32 {
    let (mut counted, mut different) = (0u32, 0u32);
    for y in y0..y1 {
        for x in x0..x1 {
            let index = (y * width + x) * 4;
            if reference[index + 3] != 255 {
                continue;
            }
            counted += 1;
            let worst = (0..3)
                .map(|channel| reference[index + channel].abs_diff(actual[index + channel]))
                .max()
                .unwrap();
            different += u32::from(worst > TOLERANCE);
        }
    }
    100.0 * different as f32 / counted.max(1) as f32
}

fn compare(name: &str, scene: fn(bool) -> AppComponent, elements: &[Element]) {
    let mut worse = Vec::new();
    for dark in [false, true] {
        let actual_png = render(scene(dark), dark);
        if let Some(out) = std::env::var_os("APPKIT_OUT") {
            let out = PathBuf::from(out);
            fs::create_dir_all(&out).unwrap();
            fs::write(
                out.join(format!(
                    "twin-{name}-{}.png",
                    if dark { "dark" } else { "light" }
                )),
                &actual_png,
            )
            .unwrap();
        }
        let (width, height, actual) = crate::image::decode(&actual_png);
        let (rw, rh, reference) = reference(name, dark);
        assert_eq!(
            (width, height),
            (rw, rh),
            "the twin and the kit differ in size"
        );
        for element in elements {
            let percent = percent_different(&reference, &actual, width, element.rect);
            println!(
                "{name:6} {:5} {:22} {percent:5.1}% {}",
                if dark { "dark" } else { "light" },
                element.name,
                if element.text { "(text)" } else { "" }
            );
            if percent > element.ceiling {
                worse.push(format!(
                    "{name} {} {}: {percent:.1}% over {}%",
                    if dark { "dark" } else { "light" },
                    element.name,
                    element.ceiling
                ));
            }
        }
    }
    assert!(worse.is_empty(), "{worse:?}");
}

#[test]
fn the_notes_window_matches_the_swiftui_twin_per_element() {
    compare(
        "notes",
        |dark| AppComponent::new(NotesTwin { dark }),
        &NOTES,
    );
}

#[test]
fn the_tasks_window_matches_the_swiftui_twin_per_element() {
    compare(
        "tasks",
        |dark| AppComponent::new(TasksTwin { dark }),
        &TASKS_ELEMENTS,
    );
}

#[test]
fn the_form_window_matches_the_swiftui_twin_per_element() {
    compare(
        "form",
        |dark| AppComponent::new(FormTwin { dark }),
        &FORM_ELEMENTS,
    );
}

fn halved(image: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(width / 2 * (height / 2) * 4);
    for y in 0..height / 2 {
        for x in 0..width / 2 {
            for channel in 0..4 {
                let sum: u32 = [(0, 0), (1, 0), (0, 1), (1, 1)]
                    .iter()
                    .map(|(dx, dy)| {
                        u32::from(image[((2 * y + dy) * width + 2 * x + dx) * 4 + channel])
                    })
                    .sum();
                out.push((sum / 4) as u8);
            }
        }
    }
    out
}

#[test]
fn the_notes_window_at_scale_two_is_the_window_at_scale_one_drawn_twice_as_fine() {
    for dark in [false, true] {
        let (width, height, once) =
            crate::image::decode(&render_at(AppComponent::new(NotesTwin { dark }), dark, 1.0));
        let twice_png = render_at(AppComponent::new(NotesTwin { dark }), dark, 2.0);
        if let Some(out) = std::env::var_os("APPKIT_OUT") {
            let out = PathBuf::from(out);
            fs::create_dir_all(&out).unwrap();
            fs::write(
                out.join(format!(
                    "twin-notes-2x-{}.png",
                    if dark { "dark" } else { "light" }
                )),
                &twice_png,
            )
            .unwrap();
        }
        let (wide, high, twice) = crate::image::decode(&twice_png);
        assert_eq!((wide, high), (2 * width, 2 * height), "twice the pixels");
        let twice = halved(&twice, wide, high);
        for element in NOTES.iter().filter(|element| !element.text) {
            let percent = percent_different(&once, &twice, width, element.rect);
            println!(
                "scale 2 {:5} {:22} {percent:5.1}%",
                if dark { "dark" } else { "light" },
                element.name
            );
            assert!(
                percent <= 10.0,
                "{} {}: {percent:.1}% differs from the scale 1 drawing",
                if dark { "dark" } else { "light" },
                element.name
            );
        }
    }
}

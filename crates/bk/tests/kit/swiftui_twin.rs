use std::{fs, path::PathBuf};

use bk::choices::RadioGroup;
use bk::compact::IconButton;
use bk::compact::Stepper;
use bk::controls::{Button, Checkbox, SearchField, SegmentedControl, Slider, TextField, ToolGroup};
use bk::date::{CivilDate, CivilDateTime};
use bk::date_picker::DatePicker;
use bk::elements::{Label, VStack};
use bk::indicator::CapacityIndicator;
use bk::popup_button::PopUpButton;
use bk::progress::ProgressView;
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
use torin::prelude::{Alignment, Direction, Gaps, Size, Size2D};

const SIZE: (f32, f32) = (760.0, 480.0);
const TOLERANCE: u8 = 8;

struct Element {
    name: &'static str,
    rect: (usize, usize, usize, usize),
    text: bool,
    ceiling: f32,
    // With the system's Reduce Transparency on.
    reduced_ceiling: f32,
}

impl Element {
    const fn reduced(mut self, ceiling: f32) -> Self {
        self.reduced_ceiling = ceiling;
        self
    }
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
        reduced_ceiling: ceiling,
    }
}

const NOTES: [Element; 10] = [
    element("lights", (10, 10, 100, 42), false, 14.0),
    element("sidebar toggle", (170, 2, 218, 50), false, 24.0),
    element("sidebar selected row", (10, 52, 210, 84), true, 14.0).reduced(15.0),
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
    // The SwiftUI list draws a hairline under the toolbar when Reduce Transparency replaces its scroll-edge effect; the kit draws none.
    element("title bar edge", (0, 48, 760, 56), false, 1.0).reduced(13.0),
    element("window background", (0, 170, 760, 480), false, 1.0),
];

const FORM_ELEMENTS: [Element; 7] = [
    element("heading", (28, 50, 140, 72), true, 17.0),
    element("card background", (300, 80, 600, 112), false, 1.0),
    element("row separators", (28, 113, 732, 116), false, 1.0),
    element("switch", (680, 82, 730, 110), false, 14.0),
    element("row labels", (28, 80, 200, 186), true, 6.0),
    element("row values", (560, 116, 732, 150), true, 8.0),
    element("text field row", (28, 152, 732, 186), true, 4.0),
];

const CONTROLS_ELEMENTS: [Element; 9] = [
    element("pop-up button", (560, 82, 725, 112), true, 11.0),
    element("segmented control", (485, 121, 725, 150), true, 14.0),
    element("stepper", (695, 163, 725, 191), false, 11.0),
    element("slider", (340, 205, 725, 230), false, 11.0),
    element("date field", (640, 240, 725, 270), true, 26.0),
    element("progress", (28, 278, 732, 332), true, 3.0),
    element("capacity gauge", (28, 334, 732, 394), true, 6.0),
    element("checkbox", (28, 396, 200, 430), true, 10.0),
    element("button row", (28, 436, 200, 474), true, 5.0),
];

struct ControlsTwin {
    dark: bool,
}

impl App for ControlsTwin {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(self.dark));
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
                    Section::titled("Controls")
                        .row(
                            Row::new("Size").control(
                                PopUpButton::new("Size")
                                    .item("Small")
                                    .item("Medium")
                                    .item("Large")
                                    .selected(1),
                            ),
                        )
                        .row(
                            Row::new("View").control(
                                SegmentedControl::new("View")
                                    .segment("List")
                                    .segment("Icons")
                                    .segment("Columns")
                                    .selected(0),
                            ),
                        )
                        .row(Row::new("Count").control(Stepper::new("Count", 0, 10).value(3)))
                        .row(Row::new("Volume").control(Slider::new("Volume").value(0.4)))
                        .row(Row::new("Date").control(DatePicker::new(
                            "Date",
                            CivilDateTime::new(
                                CivilDate {
                                    year: 2023,
                                    month: 11,
                                    day: 14,
                                },
                                0,
                                0,
                            ),
                        )))
                        .row(
                            Row::new("Progress")
                                .stacked()
                                .control(ProgressView::determinate("Progress", 0.6)),
                        )
                        .row(
                            Row::new("Level")
                                .stacked()
                                .control(CapacityIndicator::new("Level", 0.5)),
                        )
                        .row(Row::new("").control(Checkbox::new("Check").checked(true)))
                        .row(Row::button("Reset", |()| {})),
                ),
            )
    }
}

const STANDALONE_ELEMENTS: [Element; 5] = [
    element("pop-up button", (18, 50, 120, 78), true, 19.0),
    element("checkbox", (18, 92, 84, 114), true, 36.0),
    element("radio group", (18, 128, 70, 174), true, 27.0),
    element("button", (18, 188, 82, 218), true, 24.0),
    element("search field", (17, 232, 224, 262), true, 20.0),
];

struct StandaloneTwin {
    dark: bool,
}

impl App for StandaloneTwin {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(self.dark));
        let search = use_state(String::new);
        rect()
            .expanded()
            .background(theme::color(theme::surface_background(
                &theme::resolved(),
                SurfaceKind::Toplevel,
            )))
            .direction(Direction::Vertical)
            .child(rect().height(Size::px(32.0)))
            .child(
                VStack::new()
                    .spacing(20.0)
                    .padding(Gaps::new_all(20.0))
                    .cross_align(Alignment::Start)
                    .child(
                        PopUpButton::new("Size")
                            .item("Small")
                            .item("Medium")
                            .item("Large")
                            .selected(1),
                    )
                    .child(Checkbox::new("Check").checked(true))
                    .child(
                        RadioGroup::new("Mode")
                            .option("One")
                            .option("Two")
                            .selected(0),
                    )
                    .child(Button::new("Reset"))
                    .child(SearchField::new(search, "Search").width(Size::px(200.0))),
            )
    }
}

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

fn render(app: AppComponent, dark: bool, reduced_transparency: bool) -> Vec<u8> {
    render_at(app, dark, reduced_transparency, 1.0)
}

fn render_at(app: AppComponent, dark: bool, reduced_transparency: bool, scale: f64) -> Vec<u8> {
    let resolved = resolve(
        &AppearanceConfig {
            color_scheme: if dark {
                ColorScheme::Dark
            } else {
                ColorScheme::Light
            },
            reduced_motion: true,
            reduce_transparency: reduced_transparency,
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

// `light`, `dark`, and with the system's Reduce Transparency on, `light-rt` and `dark-rt`.
fn cell(dark: bool, reduced_transparency: bool) -> String {
    format!(
        "{}{}",
        if dark { "dark" } else { "light" },
        if reduced_transparency { "-rt" } else { "" }
    )
}

fn reference(name: &str, cell: &str) -> Option<(usize, usize, Vec<u8>)> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/reference")
        .join(format!("swiftui-{name}-{cell}.png"));
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(_) if cell.ends_with("-rt") => return None,
        Err(error) => panic!("read {path:?}: {error}"),
    };
    Some(crate::image::decode(&bytes))
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
    for (dark, reduced) in [(false, false), (true, false), (false, true), (true, true)] {
        let cell = cell(dark, reduced);
        let Some((rw, rh, reference)) = reference(name, &cell) else {
            println!("{name:6} {cell:9} no reference yet");
            continue;
        };
        let actual_png = render(scene(dark), dark, reduced);
        if let Some(out) = std::env::var_os("APPKIT_OUT") {
            let out = PathBuf::from(out);
            fs::create_dir_all(&out).unwrap();
            fs::write(out.join(format!("twin-{name}-{cell}.png")), &actual_png).unwrap();
        }
        let (width, height, actual) = crate::image::decode(&actual_png);
        assert_eq!(
            (width, height),
            (rw, rh),
            "the twin and the kit differ in size"
        );
        for element in elements {
            let percent = percent_different(&reference, &actual, width, element.rect);
            println!(
                "{name:6} {cell:9} {:22} {percent:5.1}% {}",
                element.name,
                if element.text { "(text)" } else { "" }
            );
            let ceiling = if reduced {
                element.reduced_ceiling
            } else {
                element.ceiling
            };
            if percent > ceiling {
                worse.push(format!(
                    "{name} {cell} {}: {percent:.1}% over {ceiling}%",
                    element.name
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

#[test]
fn the_controls_window_matches_the_swiftui_twin_per_element() {
    compare(
        "controls",
        |dark| AppComponent::new(ControlsTwin { dark }),
        &CONTROLS_ELEMENTS,
    );
}

#[test]
fn the_standalone_controls_match_the_swiftui_twin_per_element() {
    compare(
        "standalone",
        |dark| AppComponent::new(StandaloneTwin { dark }),
        &STANDALONE_ELEMENTS,
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
        let (width, height, once) = crate::image::decode(&render_at(
            AppComponent::new(NotesTwin { dark }),
            dark,
            false,
            1.0,
        ));
        let twice_png = render_at(AppComponent::new(NotesTwin { dark }), dark, false, 2.0);
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

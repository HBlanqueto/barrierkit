use std::cell::RefCell;
use std::rc::Rc;

use bk::controls::Switch;
use bk::elements::Text;
use bk::styles::{ShapeStyle, TableStyle};
use bk::symbol::Symbol;
use bk::theme;
use bk::views::list::List;
use bk::views::table::{ListRowModifiers, Table, Visibility};
use ctappearance::{AppearanceConfig, ColorScheme, resolve};
use ctdesign::SANS_FAMILIES;
use freya_components::theming::hooks::use_init_theme;
use freya_core::integration::AppComponent;
use freya_core::prelude::Label as TextRun;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Size, Size2D};

const NAMES: [&str; 4] = ["adr-0047.pdf", "notas.txt", "portada.png", "datos.csv"];
const KINDS: [&str; 4] = ["PDF", "Text", "Image", "Table"];
const SIZE: (f32, f32) = (300.0, 170.0);
const FIRST_ROW: (f64, f64) = (150.0, 17.0);

struct Case {
    dark: bool,
    build: Rc<dyn Fn() -> Element>,
}

impl App for Case {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(self.dark));
        rect().expanded().child((self.build)())
    }
}

fn runner(dark: bool, build: Rc<dyn Fn() -> Element>) -> TestingRunner {
    let resolved = resolve(
        &AppearanceConfig {
            color_scheme: if dark {
                ColorScheme::Dark
            } else {
                ColorScheme::Light
            },
            reduce_transparency: true,
            reduced_motion: true,
            ..Default::default()
        },
        None,
    );
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Case { dark, build }),
        Size2D::new(SIZE.0, SIZE.1),
        move |runner| runner.provide_root_context(move || State::create(resolved)),
        1.0,
    );
    test.set_default_fonts(&SANS_FAMILIES.map(std::borrow::Cow::Borrowed));
    test.sync_and_update();
    test.sync_and_update();
    test
}

fn settle(test: &mut TestingRunner) {
    test.sync_and_update();
    test.sync_and_update();
}

fn pixels(dark: bool, focused: bool, build: impl Fn() -> Element + 'static) -> Vec<u8> {
    let mut test = runner(dark, Rc::new(build));
    if focused {
        test.click_cursor(FIRST_ROW);
        settle(&mut test);
    }
    crate::image::decode(test.render().as_bytes()).2
}

fn differing(a: &[u8], b: &[u8]) -> usize {
    a.iter()
        .zip(b)
        .filter(|(a, b)| a.abs_diff(**b) > crate::controls::TOLERANCE)
        .count()
}

fn names() -> Vec<String> {
    NAMES.map(String::from).to_vec()
}

fn rows() -> Vec<Vec<String>> {
    NAMES
        .iter()
        .zip(KINDS)
        .map(|(name, kind)| vec![name.to_string(), kind.to_string()])
        .collect()
}

fn same_pixels(
    drawn: impl Fn() -> Element + Clone + 'static,
    view: impl Fn() -> Element + Clone + 'static,
) {
    for (dark, focused) in [(false, false), (false, true), (true, false), (true, true)] {
        let differences = differing(
            &pixels(dark, focused, drawn.clone()),
            &pixels(dark, focused, view.clone()),
        );
        assert_eq!(
            differences, 0,
            "dark {dark}, focused {focused}: {differences} channels differ"
        );
    }
}

#[test]
fn a_list_row_that_is_a_text_view_draws_like_the_text_row() {
    let drawn = || List::new("Files", names()).selected(vec![1]).into_element();
    let view = || {
        List::new("Files", names())
            .row(|index| Text::new(NAMES[index]))
            .selected(vec![1])
            .into_element()
    };
    same_pixels(drawn, view);
}

#[test]
fn the_rows_of_a_list_with_sections_get_the_index_across_the_sections() {
    let drawn = || {
        List::new("Files", Vec::new())
            .section("Favorites", names()[..2].to_vec())
            .section("Locations", names()[2..].to_vec())
            .selected(vec![2])
            .into_element()
    };
    let view = || {
        List::new("Files", Vec::new())
            .section("Favorites", names()[..2].to_vec())
            .section("Locations", names()[2..].to_vec())
            .row(|index| Text::new(NAMES[index]))
            .selected(vec![2])
            .into_element()
    };
    same_pixels(drawn, view);
}

#[test]
fn a_table_cell_that_is_a_text_view_draws_like_the_text_cell_beside_an_icon() {
    let table = || {
        Table::new("Files", ["Name", "Kind"])
            .rows(rows())
            .icons(vec![Symbol::Document; 4])
            .selected(vec![1])
    };
    let view = move || {
        table()
            .cell(0, |row| Text::new(NAMES[row]))
            .cell(1, |row| Text::new(KINDS[row]))
            .into_element()
    };
    same_pixels(move || table().into_element(), view);
}

#[test]
fn a_table_that_takes_owned_columns_draws_like_one_with_literals() {
    let literals = || {
        Table::new("Files", ["Name", "Kind"])
            .rows(rows())
            .into_element()
    };
    let owned = || {
        Table::new("Files", vec![String::from("Name"), String::from("Kind")])
            .rows(rows())
            .into_element()
    };
    same_pixels(literals, owned);
}

type Selections = Rc<RefCell<Vec<Vec<usize>>>>;
type Toggles = Rc<RefCell<Vec<bool>>>;

fn pressable(selected: Selections, toggled: Toggles) -> Element {
    List::new("Files", names())
        .row(move |index| {
            let toggled = toggled.clone();
            rect()
                .on_press(|event: Event<PressEventData>| event.stop_propagation())
                .child(Switch::new(NAMES[index]).on_toggle(move |on: bool| {
                    toggled.borrow_mut().push(on);
                }))
        })
        .on_select(move |rows: Vec<usize>| selected.borrow_mut().push(rows))
        .style(|style| TableStyle {
            row_height: 40.0,
            ..style
        })
        .into_element()
}

#[test]
fn a_press_on_a_control_that_stops_it_does_not_select_the_row_and_a_press_beside_it_does() {
    let (selected, toggled) = (Selections::default(), Toggles::default());
    let build = {
        let (selected, toggled) = (selected.clone(), toggled.clone());
        Rc::new(move || pressable(selected.clone(), toggled.clone()))
    };
    let mut test = runner(false, build);
    test.click_cursor((36.0, 25.0));
    settle(&mut test);
    assert_eq!(*toggled.borrow(), [true], "the switch toggled");
    assert!(selected.borrow().is_empty(), "the row was not selected");
    test.click_cursor((200.0, 25.0));
    settle(&mut test);
    assert_eq!(*selected.borrow(), [vec![0]]);
}

#[test]
fn a_view_row_shows_the_value_its_parent_holds() {
    struct Counter;
    impl App for Counter {
        fn render(&self) -> impl IntoElement {
            let mut presses = use_state(|| 0);
            rect()
                .child(
                    rect()
                        .width(Size::px(100.0))
                        .height(Size::px(20.0))
                        .on_press(move |_| *presses.write() += 1)
                        .child(label().text(format!("parent {}", presses.read()))),
                )
                .child(
                    rect().width(Size::px(200.0)).height(Size::px(100.0)).child(
                        List::new("Files", names())
                            .row(move |_| label().text(format!("row {}", presses.read()))),
                    ),
                )
        }
    }
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Counter),
        Size2D::new(300.0, 200.0),
        |_| {},
        1.0,
    );
    settle(&mut test);
    let shows = |test: &TestingRunner, text: &str| {
        test.find(|_, element| TextRun::try_downcast(element).filter(|run| run.text == text))
            .is_some()
    };
    assert!(shows(&test, "parent 0") && shows(&test, "row 0"));
    test.click_cursor((20.0, 10.0));
    settle(&mut test);
    assert!(shows(&test, "parent 1") && shows(&test, "row 1"));
}

#[test]
fn a_hidden_row_separator_draws_like_a_style_without_one() {
    let drawn = || {
        List::new("Files", names())
            .style(|style| TableStyle {
                row_separator: None,
                ..style
            })
            .into_element()
    };
    let view = || {
        List::new("Files", names())
            .list_row_separator(|_| Visibility::Hidden)
            .into_element()
    };
    same_pixels(drawn, view);
}

#[test]
fn a_visible_row_separator_draws_a_line_a_table_does_not() {
    let plain = || Table::new("Files", ["Name"]).rows(rows()).into_element();
    let visible = || {
        Table::new("Files", ["Name"])
            .rows(rows())
            .list_row_separator(|_| Visibility::Visible)
            .into_element()
    };
    assert_ne!(
        differing(&pixels(false, false, plain), &pixels(false, false, visible)),
        0
    );
}

#[test]
fn the_background_of_a_row_in_a_sectioned_list_is_asked_with_the_index_across_the_sections() {
    let asked = Rc::new(RefCell::new(Vec::new()));
    let seen = asked.clone();
    let list = move || {
        let seen = seen.clone();
        List::new("Files", Vec::new())
            .section("Favorites", names()[..2].to_vec())
            .section("Locations", names()[2..].to_vec())
            .list_row_background(move |row| {
                seen.borrow_mut().push(row);
                ShapeStyle::Clear
            })
            .into_element()
    };
    pixels(false, false, list);
    let mut asked = asked.borrow().clone();
    asked.sort();
    asked.dedup();
    assert_eq!(asked, vec![0, 1, 2, 3]);
}

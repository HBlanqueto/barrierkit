use bk::controls::Button;
use bk::prelude::*;
use bk::symbol::Symbol;
use bk::views::feedback::ContentUnavailableView;
use freya_core::integration::AppComponent;
use freya_testing::TestingRunner;
use torin::prelude::{Area, Size2D};

struct Case(Box<dyn Fn() -> Element>);

impl App for Case {
    fn render(&self) -> impl IntoElement {
        rect().expanded().child((self.0)())
    }
}

fn runner(build: impl Fn() -> Element + 'static) -> TestingRunner {
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Case(Box::new(build))),
        Size2D::new(400.0, 300.0),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    test.sync_and_update();
    test
}

fn area(test: &TestingRunner, id: &str) -> Area {
    test.find(|node, element| {
        (element.accessibility().builder.author_id() == Some(id)).then(|| node.layout().area)
    })
    .unwrap_or_else(|| panic!("no node probed as {id}"))
}

// Frames read from a SwiftUI ContentUnavailableView in a 400 x 300 view on macOS 27.2 (~/src/probes/contentunavailable.swift).
const SWIFTUI_TITLE_ONLY_TOP: f32 = 107.0;
const SWIFTUI_WITH_DESCRIPTION_TOP: f32 = 90.0;
const SWIFTUI_WITH_ACTIONS_TOP: f32 = 80.0;
const SWIFTUI_VERTICAL_TOLERANCE: f32 = 8.0;

fn title_only() -> Element {
    ContentUnavailableView::new("No Note Selected", Symbol::DocText).into_element()
}

fn with_description() -> Element {
    ContentUnavailableView::new("No Results", Symbol::MagnifyingGlass)
        .description("Check the spelling or try a new search.")
        .into_element()
}

fn with_actions() -> Element {
    ContentUnavailableView::new("No Mail", Symbol::Tray)
        .description("New mail you receive will appear here.")
        .actions(Button::new("Refresh"))
        .into_element()
}

// A probed box includes its top margin, which is the gap above its line.
#[test]
fn the_symbol_sits_in_a_44_high_line_and_the_title_20_below_it_as_in_swiftui() {
    let test = runner(title_only);
    let symbol = area(&test, "content-unavailable-symbol");
    let title = area(&test, "content-unavailable-title");
    assert_eq!(symbol.height(), 44.0);
    assert_eq!(title.min_y(), symbol.max_y());
    assert_eq!(title.height(), 20.0 + 19.0);
}

#[test]
fn the_description_is_6_below_the_title_and_19_high() {
    let test = runner(with_description);
    let title = area(&test, "content-unavailable-title");
    let description = area(&test, "content-unavailable-description");
    assert_eq!(description.min_y(), title.max_y());
    assert_eq!(description.height(), 6.0 + 19.0);
}

#[test]
fn the_actions_are_19_below_the_description() {
    let test = runner(with_actions);
    let description = area(&test, "content-unavailable-description");
    let actions = area(&test, "content-unavailable-actions");
    assert_eq!(actions.min_y(), description.max_y());
    assert_eq!(actions.height(), 19.0 + 24.0);
}

#[test]
fn the_view_centers_its_content_in_the_space_it_fills() {
    for build in [title_only, with_description, with_actions] {
        let test = runner(build);
        let symbol = area(&test, "content-unavailable-symbol");
        assert_eq!(symbol.min_x() + symbol.width() / 2.0, 200.0);
    }
}

#[test]
fn the_content_starts_within_8_of_where_swiftui_puts_it() {
    for (build, top) in [
        (title_only as fn() -> Element, SWIFTUI_TITLE_ONLY_TOP),
        (with_description, SWIFTUI_WITH_DESCRIPTION_TOP),
        (with_actions, SWIFTUI_WITH_ACTIONS_TOP),
    ] {
        let test = runner(build);
        let start = area(&test, "content-unavailable-symbol").min_y();
        assert!(
            (start - top).abs() <= SWIFTUI_VERTICAL_TOLERANCE,
            "{start} vs {top}"
        );
    }
}

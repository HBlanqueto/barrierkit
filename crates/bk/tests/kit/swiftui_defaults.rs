use bk::ProbeExt;
use bk::controls::{Button, ControlSize};
use bk::prelude::*;
use bk::symbol::Symbol;
use bk::views::feedback::ContentUnavailableView;
use bk::views::form::{Form, Row, Section};
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

fn text_height(font: ctdesign::Font) -> f32 {
    let test = runner(move || {
        rect()
            .probe("line")
            .child(bk::style::text("Hxg", font, ctdesign::Rgba::rgb8(0, 0, 0)))
            .into_element()
    });
    area(&test, "line").height()
}

// Heights of a SwiftUI Text on macOS 27.2 (~/src/probes/textlines.swift).
#[test]
fn the_text_styles_have_the_line_heights_of_swiftui() {
    use ctdesign::*;
    for (font, height) in [
        (LARGE_TITLE, 31.0),
        (TITLE, 26.0),
        (TITLE_2, 19.0),
        (TITLE_3, 19.0),
        (HEADLINE, 16.0),
        (BODY, 16.0),
        (CALLOUT, 15.0),
        (SUBHEADLINE, 14.0),
        (FOOTNOTE, 13.0),
        (CAPTION_2, 13.0),
    ] {
        assert_eq!(text_height(font), height, "{}pt", font.size);
    }
}

fn size_of(build: impl Fn() -> Element + 'static) -> (f32, f32) {
    let test = runner(move || rect().probe("stack").child(build()).into_element());
    let found = area(&test, "stack");
    (found.width(), found.height())
}

// SwiftUI puts 8 between buttons in either stack (fittingSize 74 x 24 and 33 x 56 for "A" and "B").
#[test]
fn a_stack_of_buttons_is_8_apart_as_in_swiftui() {
    let (button_width, button_height) = size_of(|| Button::new("A").into_element());
    let (row_width, row_height) = size_of(|| {
        HStack::new()
            .child(Button::new("A"))
            .child(Button::new("B"))
            .into_element()
    });
    let (column_width, column_height) = size_of(|| {
        VStack::new()
            .child(Button::new("A"))
            .child(Button::new("B"))
            .into_element()
    });
    assert_eq!(row_height, button_height);
    assert!(
        (row_width - (2.0 * button_width + 8.0)).abs() < 1.0,
        "{row_width}"
    );
    assert_eq!(column_height, 2.0 * button_height + 8.0);
    assert!(column_width < row_width);
}

fn form_height(width: f32, rows: fn(Section) -> Section) -> f32 {
    let test = runner(move || {
        rect()
            .width(Size::px(width))
            .probe("form")
            .child(Form::new().section(rows(Section::titled("Ventanas"))))
            .into_element()
    });
    area(&test, "form").height()
}

// A SwiftUI grouped Form on macOS 27.2 (~/src/probes/formgrouped.swift): the heading at 20 and 16 high, the card 10 under
// it, rows of 16 plus 10 above and below with 1 between them, a control as tall as 24 makes its row 44.
fn three_rows(section: Section) -> Section {
    section
        .row(Row::toggle("Ampliación", true, |_| {}))
        .row(Row::value("Estado", "Normal"))
        .row(Row::navigation("Avanzado", |_| {}))
}

#[test]
fn a_form_is_a_heading_and_a_card_of_rows_36_high_as_in_swiftui() {
    let height = form_height(360.0, three_rows);
    assert_eq!(height, 20.0 + 16.0 + 10.0 + (3.0 * 36.0 + 2.0) + 20.0);
}

#[test]
fn a_form_keeps_the_control_on_the_row_at_any_width() {
    assert_eq!(
        form_height(260.0, three_rows),
        form_height(360.0, three_rows)
    );
}

#[test]
fn a_row_with_a_button_is_44_high() {
    let height = form_height(360.0, |section| {
        section.row(Row::new("Ventana").control(Button::new("Avanzado")))
    });
    assert_eq!(height, 20.0 + 16.0 + 10.0 + 44.0 + 20.0);
}

// NSSlider and SwiftUI's Slider are 12, 14, 16 and 20 high at mini, small, regular and large; a label line is only
// reserved when the slider has labels (~/src/probes/intrinsic.swift).
#[test]
fn a_slider_is_as_tall_as_its_knob_until_it_has_labels() {
    use bk::controls::{ControlSize, Slider};
    for (size, height) in [
        (ControlSize::Mini, 12.0),
        (ControlSize::Small, 14.0),
        (ControlSize::Regular, 16.0),
        (ControlSize::Large, 20.0),
    ] {
        let (_, found) = size_of(move || {
            rect()
                .width(Size::px(100.0))
                .child(Slider::new("S").size(size))
                .into_element()
        });
        assert_eq!(found, height, "{size:?}");
    }
    let (_, labeled) = size_of(|| {
        rect()
            .width(Size::px(100.0))
            .child(Slider::new("S").labels(&["Low", "High"]))
            .into_element()
    });
    assert!(labeled > 16.0);
}

fn label_width(title: &'static str) -> f32 {
    size_of(move || bk::elements::Label::new(title, Symbol::DocText).into_element()).0
}

// SwiftUI's Label("Doc", systemImage:) is 48 wide for a 26 wide title: a 16 wide icon and 6 before the title.
#[test]
fn a_label_is_a_16_wide_icon_and_6_before_its_title() {
    let title = size_of(|| bk::elements::Text::new("Doc").into_element()).0;
    assert_eq!(label_width("Doc"), 16.0 + 6.0 + title);
}

// Frames read from AppKit and SwiftUI on macOS 27.2 in a window (~/src/probes/intrinsic.swift, controlsizes.swift), at
// mini, small, regular, large and extra large.
const SIZES: [ControlSize; 5] = [
    ControlSize::Mini,
    ControlSize::Small,
    ControlSize::Regular,
    ControlSize::Large,
    ControlSize::ExtraLarge,
];

#[test]
fn a_segmented_control_is_as_wide_as_its_segments_at_every_size() {
    use bk::controls::SegmentedControl;
    for (size, width) in SIZES.into_iter().zip([82.0, 90.0, 98.0, 106.0, 122.0]) {
        let (found, _) = size_of(move || {
            SegmentedControl::new("S")
                .segment("One")
                .segment("Two")
                .size(size)
                .into_element()
        });
        assert!(
            (found - width).abs() < 3.0,
            "{size:?}: {found} against {width}"
        );
    }
}

#[test]
fn a_stepper_arrows_are_as_tall_as_appkits_at_every_size() {
    use bk::compact::Stepper;
    for (size, height) in SIZES.into_iter().zip([16.0, 20.0, 24.0, 28.0, 36.0]) {
        let (_, found) = size_of(move || Stepper::new("S", 0, 10).size(size).into_element());
        assert_eq!(found, height, "{size:?}");
    }
}

#[test]
fn a_progress_bar_is_12_high_when_small_and_20_when_regular_or_larger() {
    use bk::progress::ProgressView;
    for (size, height) in SIZES.into_iter().zip([12.0, 12.0, 20.0, 20.0, 20.0]) {
        let (_, found) = size_of(move || {
            rect()
                .width(Size::px(100.0))
                .child(ProgressView::determinate("P", 0.5).control_size(size))
                .into_element()
        });
        assert_eq!(found, height, "{size:?}");
    }
}

#[test]
fn a_spinner_is_10_16_and_32_square() {
    use bk::progress::Spinner;
    for (size, side) in SIZES.into_iter().zip([10.0, 16.0, 32.0, 32.0, 32.0]) {
        let (width, height) = size_of(move || Spinner::new("S").control_size(size).into_element());
        assert_eq!((width, height), (side, side), "{size:?}");
    }
}

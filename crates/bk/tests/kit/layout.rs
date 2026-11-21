use bk::ProbeExt;
use bk::prelude::*;
use freya_core::integration::AppComponent;
use freya_testing::TestingRunner;
use torin::prelude::{Alignment, Area, Size, Size2D};

struct Case(Box<dyn Fn() -> Element>);

impl App for Case {
    fn render(&self) -> impl IntoElement {
        rect()
            .expanded()
            .main_align(Alignment::Start)
            .cross_align(Alignment::Start)
            .child((self.0)())
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

fn at(test: &TestingRunner, id: &str) -> (f32, f32) {
    let found = area(test, id);
    (found.min_x(), found.min_y())
}

fn size(test: &TestingRunner, id: &str) -> (f32, f32) {
    let found = area(test, id);
    (found.width(), found.height())
}

fn block(id: &'static str, width: f32, height: f32) -> Rect {
    rect()
        .width(Size::px(width))
        .height(Size::px(height))
        .probe(id)
}

fn grid(id: &'static str) -> Grid {
    Grid::new().a11y_builder(move |node| node.set_author_id(id))
}

#[test]
fn a_spacer_pushes_the_last_child_to_the_end() {
    let test = runner(|| {
        HStack::new()
            .spacing(0.0)
            .width(Size::px(200.0))
            .child(block("a", 10.0, 10.0))
            .child(Spacer::new())
            .child(block("b", 10.0, 10.0))
            .into_element()
    });
    assert_eq!(at(&test, "a").0, 0.0);
    assert_eq!(at(&test, "b").0, 190.0);
}

#[test]
fn two_spacers_share_the_free_space_equally() {
    let test = runner(|| {
        HStack::new()
            .spacing(0.0)
            .width(Size::px(200.0))
            .child(block("a", 10.0, 10.0))
            .child(Spacer::new())
            .child(block("b", 10.0, 10.0))
            .child(Spacer::new())
            .child(block("c", 10.0, 10.0))
            .into_element()
    });
    assert_eq!(at(&test, "b").0, 95.0);
    assert_eq!(at(&test, "c").0, 190.0);
}

#[test]
fn a_spacer_never_shrinks_below_its_minimum_length() {
    let test = runner(|| {
        HStack::new()
            .spacing(0.0)
            .width(Size::px(100.0))
            .child(block("a", 10.0, 10.0))
            .child(Spacer::new().min_length(150.0))
            .child(block("b", 10.0, 10.0))
            .into_element()
    });
    assert_eq!(at(&test, "b").0, 160.0);
}

#[test]
fn a_spacer_in_a_vertical_stack_pushes_along_the_column() {
    let test = runner(|| {
        VStack::new()
            .spacing(0.0)
            .height(Size::px(100.0))
            .child(block("a", 10.0, 10.0))
            .child(Spacer::new())
            .child(block("b", 10.0, 10.0))
            .into_element()
    });
    assert_eq!(at(&test, "a").1, 0.0);
    assert_eq!(at(&test, "b").1, 90.0);
}

#[test]
fn a_spacer_does_not_stretch_a_row_across_its_height() {
    let test = runner(|| {
        HStack::new()
            .cross_align(Alignment::Start)
            .a11y_builder(|node| node.set_author_id("row"))
            .child(block("a", 10.0, 10.0))
            .child(Spacer::new())
            .into_element()
    });
    assert_eq!(size(&test, "row").1, 10.0);
}

#[test]
fn a_stack_holding_a_spacer_fills_a_centered_column() {
    let test = runner(|| {
        VStack::new()
            .width(Size::px(400.0))
            .child(
                HStack::new()
                    .spacing(0.0)
                    .child(block("a", 10.0, 10.0))
                    .child(Spacer::new())
                    .child(block("b", 10.0, 10.0)),
            )
            .into_element()
    });
    assert_eq!(at(&test, "a").0, 0.0);
    assert_eq!(at(&test, "b").0, 390.0);
}

#[test]
fn a_frame_centers_its_content_by_default() {
    let test = runner(|| {
        block("c", 20.0, 10.0)
            .frame()
            .width(Size::px(100.0))
            .height(Size::px(30.0))
            .into_element()
    });
    assert_eq!(at(&test, "c"), (40.0, 10.0));
}

#[test]
fn a_frame_places_its_content_by_alignment() {
    let cases = [
        (ContentAlignment::TOP_LEADING, (0.0, 0.0)),
        (ContentAlignment::TOP, (40.0, 0.0)),
        (ContentAlignment::LEADING, (0.0, 10.0)),
        (ContentAlignment::TRAILING, (80.0, 10.0)),
        (ContentAlignment::BOTTOM_LEADING, (0.0, 20.0)),
        (ContentAlignment::BOTTOM_TRAILING, (80.0, 20.0)),
    ];
    for (alignment, expected) in cases {
        let test = runner(move || {
            block("c", 20.0, 10.0)
                .frame()
                .width(Size::px(100.0))
                .height(Size::px(30.0))
                .alignment(alignment)
                .into_element()
        });
        assert_eq!(at(&test, "c"), expected, "{alignment:?}");
    }
}

#[test]
fn a_frame_that_fills_the_width_aligns_its_content() {
    let test = runner(|| {
        block("c", 20.0, 10.0)
            .frame()
            .width(Size::fill())
            .alignment(ContentAlignment::TRAILING)
            .a11y_builder(|node| node.set_author_id("frame"))
            .into_element()
    });
    assert_eq!(size(&test, "frame"), (400.0, 10.0));
    assert_eq!(at(&test, "c").0, 380.0);
}

#[test]
fn a_frame_that_fills_the_width_of_a_centered_column_starts_at_its_edge() {
    let test = runner(|| {
        VStack::new()
            .width(Size::px(400.0))
            .child(
                block("c", 20.0, 10.0)
                    .frame()
                    .width(Size::fill())
                    .alignment(ContentAlignment::LEADING)
                    .a11y_builder(|node| node.set_author_id("frame")),
            )
            .into_element()
    });
    assert_eq!(at(&test, "frame").0, 0.0);
    assert_eq!(at(&test, "c").0, 0.0);
}

fn labeled_rows() -> Grid {
    grid("grid")
        .horizontal_spacing(8.0)
        .vertical_spacing(4.0)
        .row(
            GridRow::new()
                .child(block("a1", 40.0, 10.0))
                .child(block("b1", 100.0, 16.0)),
        )
        .row(
            GridRow::new()
                .child(block("a2", 60.0, 10.0))
                .child(block("b2", 50.0, 10.0)),
        )
}

#[test]
fn grid_columns_share_the_widest_cell_and_rows_the_tallest() {
    let test = runner(|| labeled_rows().into_element());
    assert_eq!(size(&test, "grid"), (168.0, 30.0));
    assert_eq!(at(&test, "b1").0, at(&test, "b2").0 - 25.0);
    assert_eq!(at(&test, "a2").1, 20.0);
}

#[test]
fn grid_cells_are_centered_in_their_slot_by_default() {
    let test = runner(|| labeled_rows().into_element());
    assert_eq!(at(&test, "a1"), (10.0, 3.0));
    assert_eq!(at(&test, "b1"), (68.0, 0.0));
    assert_eq!(at(&test, "a2"), (0.0, 20.0));
    assert_eq!(at(&test, "b2"), (93.0, 20.0));
}

#[test]
fn a_grid_alignment_moves_every_cell_in_its_slot() {
    let test = runner(|| {
        labeled_rows()
            .alignment(ContentAlignment::TOP_LEADING)
            .into_element()
    });
    assert_eq!(at(&test, "a1"), (0.0, 0.0));
    assert_eq!(at(&test, "b2"), (68.0, 20.0));
}

#[test]
fn a_grid_row_alignment_moves_only_its_cells_vertically() {
    let test = runner(|| {
        grid("grid")
            .horizontal_spacing(8.0)
            .vertical_spacing(4.0)
            .alignment(ContentAlignment::TOP_LEADING)
            .row(
                GridRow::new()
                    .alignment(VerticalAlignment::Bottom)
                    .child(block("a1", 40.0, 10.0))
                    .child(block("b1", 100.0, 16.0)),
            )
            .into_element()
    });
    assert_eq!(at(&test, "a1"), (0.0, 6.0));
    assert_eq!(at(&test, "b1"), (48.0, 0.0));
}

#[test]
fn a_column_alignment_applies_to_every_cell_of_the_column() {
    let test = runner(|| {
        grid("grid")
            .horizontal_spacing(8.0)
            .alignment(ContentAlignment::TOP_LEADING)
            .row(
                GridRow::new()
                    .child(
                        block("a1", 40.0, 10.0)
                            .grid_column_alignment(HorizontalAlignment::Trailing),
                    )
                    .child(block("b1", 30.0, 10.0)),
            )
            .row(
                GridRow::new()
                    .child(block("a2", 60.0, 10.0))
                    .child(block("b2", 30.0, 10.0)),
            )
            .into_element()
    });
    assert_eq!(at(&test, "a1").0, 20.0);
    assert_eq!(at(&test, "a2").0, 0.0);
}

#[test]
fn a_cell_spanning_two_columns_splits_its_excess_width_equally() {
    let test = runner(|| {
        grid("grid")
            .horizontal_spacing(4.0)
            .vertical_spacing(4.0)
            .row(
                GridRow::new()
                    .child(block("a1", 10.0, 10.0))
                    .child(block("b1", 50.0, 10.0)),
            )
            .row(GridRow::new().child(block("span", 200.0, 10.0).grid_cell_columns(2)))
            .into_element()
    });
    assert_eq!(size(&test, "grid"), (200.0, 24.0));
    assert_eq!(at(&test, "a1").0, 34.0);
    assert_eq!(at(&test, "b1").0, 116.0);
    assert_eq!(at(&test, "span"), (0.0, 14.0));
}

#[test]
fn a_cell_outside_a_row_spans_all_the_columns_and_is_centered() {
    let test = runner(|| {
        grid("grid")
            .horizontal_spacing(4.0)
            .vertical_spacing(4.0)
            .row(
                GridRow::new()
                    .child(block("a1", 10.0, 10.0))
                    .child(block("b1", 50.0, 10.0)),
            )
            .child(block("bare", 20.0, 10.0))
            .into_element()
    });
    assert_eq!(size(&test, "grid"), (64.0, 24.0));
    assert_eq!(at(&test, "bare"), (22.0, 14.0));
}

#[test]
fn a_short_row_takes_the_first_columns() {
    let test = runner(|| {
        grid("grid")
            .horizontal_spacing(4.0)
            .vertical_spacing(0.0)
            .alignment(ContentAlignment::TOP_LEADING)
            .row(
                GridRow::new()
                    .child(block("a1", 10.0, 10.0))
                    .child(block("b1", 10.0, 10.0))
                    .child(block("c1", 10.0, 10.0)),
            )
            .row(GridRow::new().child(block("a2", 10.0, 10.0)))
            .into_element()
    });
    assert_eq!(size(&test, "grid"), (38.0, 20.0));
    assert_eq!(at(&test, "a2"), (0.0, 10.0));
}

#[test]
fn a_grid_inside_a_centered_stack_is_centered_by_its_final_width() {
    let test = runner(|| {
        VStack::new()
            .width(Size::px(400.0))
            .child(labeled_rows())
            .into_element()
    });
    assert_eq!(size(&test, "grid"), (168.0, 30.0));
    assert_eq!(at(&test, "grid").0, 116.0);
}

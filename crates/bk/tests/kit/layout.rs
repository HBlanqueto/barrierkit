use bk::ProbeExt;
use bk::prelude::*;
use bk::views::form::{Form, Row, Section};
use freya_core::integration::AppComponent;
use freya_testing::TestingRunner;
use torin::prelude::{Alignment, Size, Size2D};

use crate::support::area;

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

fn zstack(id: &'static str) -> ZStack {
    ZStack::new().a11y_builder(move |node| node.set_author_id(id))
}

fn layers() -> ZStack {
    zstack("z")
        .child(block("big", 60.0, 20.0))
        .child(block("small", 10.0, 4.0))
}

#[test]
fn a_zstack_is_as_large_as_its_largest_child_and_centers_the_others() {
    let test = runner(|| layers().into_element());
    assert_eq!(size(&test, "z"), (60.0, 20.0));
    assert_eq!(at(&test, "big"), (0.0, 0.0));
    assert_eq!(at(&test, "small"), (25.0, 8.0));
}

#[test]
fn a_zstack_alignment_places_every_child() {
    let corner = |alignment| runner(move || layers().alignment(alignment).into_element());
    assert_eq!(
        at(&corner(ContentAlignment::TOP_LEADING), "small"),
        (0.0, 0.0)
    );
    assert_eq!(
        at(&corner(ContentAlignment::BOTTOM_TRAILING), "small"),
        (50.0, 16.0)
    );
}

#[test]
fn a_zstack_whose_largest_child_is_not_first_still_ends_up_that_large() {
    let test = runner(|| {
        zstack("z")
            .child(block("small", 10.0, 4.0))
            .child(block("big", 60.0, 20.0))
            .into_element()
    });
    assert_eq!(size(&test, "z"), (60.0, 20.0));
    assert_eq!(at(&test, "small"), (25.0, 8.0));
}

#[test]
fn an_overlay_is_placed_in_the_frame_of_the_view_and_does_not_size_it() {
    let test = runner(|| {
        block("base", 40.0, 20.0)
            .overlay(
                ContentAlignment::BOTTOM_TRAILING,
                block("badge", 10.0, 10.0),
            )
            .a11y_builder(|node| node.set_author_id("o"))
            .into_element()
    });
    assert_eq!(size(&test, "o"), (40.0, 20.0));
    assert_eq!(at(&test, "base"), (0.0, 0.0));
    assert_eq!(at(&test, "badge"), (30.0, 10.0));
}

#[test]
fn an_overlay_larger_than_the_view_leaves_its_size_alone() {
    let test = runner(|| {
        block("base", 40.0, 20.0)
            .overlay(ContentAlignment::CENTER, block("wide", 80.0, 20.0))
            .a11y_builder(|node| node.set_author_id("o"))
            .into_element()
    });
    assert_eq!(size(&test, "o"), (40.0, 20.0));
    assert_eq!(at(&test, "wide"), (-20.0, 0.0));
}

#[test]
fn a_background_view_sits_behind_the_view_and_is_sized_by_it() {
    let test = runner(|| {
        block("base", 40.0, 20.0)
            .background_view(ContentAlignment::CENTER, block("back", 20.0, 10.0))
            .a11y_builder(|node| node.set_author_id("b"))
            .into_element()
    });
    assert_eq!(size(&test, "b"), (40.0, 20.0));
    assert_eq!(at(&test, "base"), (0.0, 0.0));
    assert_eq!(at(&test, "back"), (10.0, 5.0));
}

#[derive(PartialEq)]
struct Stairs;

impl Layout for Stairs {
    fn place_subviews(&self, _proposal: Size2D, subviews: &[Size2D]) -> Placement {
        Placement {
            size: Size2D::new(subviews.iter().map(|size| size.width).sum(), 30.0),
            origins: (0..subviews.len())
                .map(|index| (index as f32 * 20.0, index as f32 * 5.0))
                .collect(),
        }
    }
}

fn stairs() -> LayoutContainer<Stairs> {
    LayoutContainer::with_layout(Stairs)
        .a11y_builder(|node| node.set_author_id("s"))
        .child(block("a", 20.0, 10.0))
        .child(block("b", 20.0, 10.0))
        .child(block("c", 20.0, 10.0))
}

#[test]
fn a_custom_layout_places_each_subview_and_sets_the_container_size() {
    let test = runner(|| stairs().into_element());
    assert_eq!(size(&test, "s"), (60.0, 30.0));
    assert_eq!(at(&test, "a"), (0.0, 0.0));
    assert_eq!(at(&test, "b"), (20.0, 5.0));
    assert_eq!(at(&test, "c"), (40.0, 10.0));
}

fn fits(width: f32) -> Element {
    rect()
        .width(Size::px(width))
        .child(
            ViewThatFits::new(Axis::Horizontal)
                .candidate(300.0, block("wide", 10.0, 10.0))
                .candidate(150.0, block("medium", 10.0, 10.0))
                .candidate(0.0, block("narrow", 10.0, 10.0)),
        )
        .into_element()
}

fn shown(test: &TestingRunner) -> Vec<&'static str> {
    ["wide", "medium", "narrow"]
        .into_iter()
        .filter(|id| {
            test.find(|_, element| {
                (element.accessibility().builder.author_id() == Some(id)).then_some(())
            })
            .is_some()
        })
        .collect()
}

#[test]
fn view_that_fits_shows_the_first_candidate_that_fits_the_offered_width() {
    assert_eq!(shown(&runner(|| fits(350.0))), ["wide"]);
    assert_eq!(shown(&runner(|| fits(200.0))), ["medium"]);
}

#[test]
fn view_that_fits_falls_back_to_the_last_candidate() {
    assert_eq!(shown(&runner(|| fits(100.0))), ["narrow"]);
}

fn right_to_left(view: impl IntoElement) -> Element {
    view.layout_direction(LayoutDirection::RightToLeft)
        .into_element()
}

#[test]
fn a_row_runs_from_the_right_in_right_to_left() {
    let row = || {
        HStack::new()
            .spacing(0.0)
            .width(Size::px(200.0))
            .main_align(Alignment::Start)
            .child(block("a", 40.0, 10.0))
            .child(block("b", 20.0, 10.0))
            .child(block("c", 10.0, 10.0))
    };
    let test = runner(move || right_to_left(row()));
    assert_eq!(at(&test, "a").0, 160.0);
    assert_eq!(at(&test, "b").0, 140.0);
    assert_eq!(at(&test, "c").0, 130.0);
}

#[test]
fn a_column_leading_alignment_is_the_right_edge_in_right_to_left() {
    let test = runner(|| {
        right_to_left(
            VStack::new()
                .width(Size::px(200.0))
                .cross_align(Alignment::Start)
                .child(block("a", 40.0, 10.0))
                .child(block("b", 20.0, 10.0)),
        )
    });
    assert_eq!(at(&test, "a").0, 160.0);
    assert_eq!(at(&test, "b").0, 180.0);
}

#[test]
fn a_frame_alignment_leading_is_the_right_edge_in_right_to_left() {
    let test = runner(|| {
        right_to_left(
            block("a", 30.0, 10.0)
                .frame()
                .width(Size::px(200.0))
                .alignment(ContentAlignment::LEADING),
        )
    });
    assert_eq!(at(&test, "a").0, 170.0);
}

#[test]
fn a_grid_runs_its_columns_from_the_right_in_right_to_left() {
    let test = runner(|| {
        right_to_left(
            Grid::new()
                .horizontal_spacing(0.0)
                .alignment(ContentAlignment::LEADING)
                .row(
                    GridRow::new()
                        .child(block("a", 40.0, 10.0))
                        .child(block("b", 20.0, 10.0)),
                )
                .row(
                    GridRow::new()
                        .child(block("c", 20.0, 10.0))
                        .child(block("d", 10.0, 10.0)),
                ),
        )
    });
    assert_eq!(at(&test, "a").0, 20.0);
    assert_eq!(at(&test, "b").0, 0.0);
    assert_eq!(at(&test, "c").0, 40.0);
    assert_eq!(at(&test, "d").0, 10.0);
}

#[test]
fn a_zstack_and_a_custom_layout_mirror_in_right_to_left() {
    let test = runner(|| right_to_left(layers().alignment(ContentAlignment::TOP_LEADING)));
    assert_eq!(at(&test, "small").0, 50.0);
    let test = runner(|| right_to_left(stairs()));
    assert_eq!(at(&test, "a").0, 40.0);
    assert_eq!(at(&test, "c").0, 0.0);
    assert_eq!(at(&test, "b").1, 5.0);
}

#[derive(PartialEq)]
struct BuiltBelow;

impl Component for BuiltBelow {
    fn render(&self) -> impl IntoElement {
        HStack::new()
            .spacing(0.0)
            .width(Size::px(200.0))
            .main_align(Alignment::Start)
            .child(block("a", 40.0, 10.0))
            .child(block("b", 20.0, 10.0))
    }
}

#[test]
fn a_row_built_inside_a_component_below_the_scope_mirrors_once() {
    let test = runner(|| right_to_left(BuiltBelow));
    assert_eq!(at(&test, "a").0, 160.0);
    assert_eq!(at(&test, "b").0, 140.0);
}

#[test]
fn a_scope_that_repeats_the_direction_does_not_mirror_again() {
    let test = runner(|| {
        right_to_left(right_to_left(
            HStack::new()
                .spacing(0.0)
                .width(Size::px(200.0))
                .main_align(Alignment::Start)
                .child(block("a", 40.0, 10.0))
                .child(block("b", 20.0, 10.0)),
        ))
    });
    assert_eq!(at(&test, "a").0, 160.0);
}

fn settings() -> Form {
    Form::new().section(Section::new().row(Row::new("Name").control(block("c", 20.0, 10.0))))
}

#[test]
fn a_form_row_puts_its_control_on_the_left_in_right_to_left() {
    let left_to_right = runner(|| settings().into_element());
    let right_to_left = runner(|| right_to_left(settings()));
    assert!(at(&left_to_right, "c").0 > 200.0);
    assert!(at(&right_to_left, "c").0 < 200.0);
}

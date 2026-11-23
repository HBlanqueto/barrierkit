use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use bk::ProbeExt;
use bk::controls::{ScrollController, ScrollGeometry, ScrollPhase, ScrollPosition, ScrollView};
use bk::prelude::*;
use bk::views::table::Table;
use freya_core::integration::{AppComponent, PlatformEvent, WheelEventName, WheelSource};
use freya_testing::TestingRunner;
use torin::prelude::{Area, Size2D};

const VIEWPORT: f32 = 300.0;
const LIMIT_ANY: f32 = CONTENT - VIEWPORT;
const CONTENT: f32 = 1600.0;
const CONTENT_OF_ROWS: f32 = 1600.0;
#[cfg(not(target_os = "macos"))]
const LIMIT: f32 = CONTENT - VIEWPORT;

struct Case;

impl App for Case {
    fn render(&self) -> impl IntoElement {
        ScrollView::new()
            .child(rect().probe("first").height(Size::px(40.0)))
            .child(rect().height(Size::px(CONTENT - 40.0)))
    }
}

struct RefreshCase {
    refreshing: Rc<Cell<Option<State<bool>>>>,
    calls: Rc<Cell<u32>>,
}

impl App for RefreshCase {
    fn render(&self) -> impl IntoElement {
        let mut refreshing = use_state(|| false);
        self.refreshing.set(Some(refreshing));
        let calls = self.calls.clone();
        ScrollView::new()
            .refreshable(*refreshing.read(), move |()| {
                calls.set(calls.get() + 1);
                refreshing.set(true);
            })
            .child(rect().probe("first").height(Size::px(40.0)))
            .child(rect().height(Size::px(CONTENT - 40.0)))
    }
}

#[derive(Clone, Default)]
struct Handles {
    controller: Rc<Cell<Option<ScrollController>>>,
    position: Rc<Cell<Option<State<ScrollPosition>>>>,
    geometry: Rc<Cell<Option<ScrollGeometry>>>,
    phases: Rc<RefCell<Vec<(ScrollPhase, ScrollPhase)>>>,
}

struct PositionCase(Handles);

impl App for PositionCase {
    fn render(&self) -> impl IntoElement {
        let controller = use_scroll_controller();
        let position = use_state(ScrollPosition::default);
        let handles = self.0.clone();
        handles.controller.set(Some(controller));
        handles.position.set(Some(position));
        ScrollView::new()
            .controller(controller)
            .scroll_position(position)
            .on_scroll_geometry_change(move |geometry| handles.geometry.set(Some(geometry)))
            .on_scroll_phase_change({
                let phases = self.0.phases.clone();
                move |change| phases.borrow_mut().push(change)
            })
            .children((0..40).map(|row| {
                let view = rect().height(Size::px(40.0));
                if row == 20 {
                    view.probe("target")
                } else {
                    view
                }
                .id(format!("row-{row}"))
                .into_element()
            }))
    }
}

fn positioned() -> (TestingRunner, Handles) {
    let handles = Handles::default();
    let test = runner_of(AppComponent::new(PositionCase(handles.clone())));
    (test, handles)
}

fn target_top(test: &TestingRunner) -> f32 {
    test.find(|node, element| {
        (element.accessibility().builder.author_id() == Some("target"))
            .then(|| -> Area { node.layout().area })
    })
    .expect("target")
    .min_y()
}

struct TableCase;

impl App for TableCase {
    fn render(&self) -> impl IntoElement {
        Table::new("Files", vec!["Name"])
            .rows((0..100).map(|row| vec![format!("File {row}")]).collect())
    }
}

fn runner() -> TestingRunner {
    runner_of(AppComponent::new(Case))
}

fn runner_of(app: AppComponent) -> TestingRunner {
    let (mut test, _) = TestingRunner::new(app, Size2D::new(400.0, VIEWPORT), |_| {}, 1.0);
    test.sync_and_update();
    test.sync_and_update();
    test
}

fn top(test: &TestingRunner) -> f32 {
    test.find(|node, element| {
        (element.accessibility().builder.author_id() == Some("first"))
            .then(|| -> Area { node.layout().area })
    })
    .expect("first")
    .min_y()
}

fn first_row(test: &TestingRunner) -> f32 {
    test.find(|node, element| {
        (element.accessibility().builder.role() == AccessibilityRole::Row)
            .then(|| -> Area { node.layout().area })
    })
    .expect("a row")
    .min_y()
}

fn wheel(test: &mut TestingRunner, source: WheelSource, phase: TouchPhase, dy: f64) {
    test.send_event(PlatformEvent::Wheel {
        name: WheelEventName::Wheel,
        scroll: (0.0, dy).into(),
        cursor: (200.0, 150.0).into(),
        source,
        phase,
    });
    test.sync_and_update();
}

fn drag(test: &mut TestingRunner, steps: u32, dy: f64) {
    for step in 0..steps {
        let phase = if step == 0 {
            TouchPhase::Started
        } else {
            TouchPhase::Moved
        };
        wheel(test, WheelSource::Pixel, phase, dy);
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn settle(test: &mut TestingRunner, milliseconds: u64) -> Vec<f32> {
    (0..milliseconds / 8)
        .map(|_| {
            test.poll_n(Duration::from_millis(8), 1);
            top(test)
        })
        .collect()
}

#[test]
fn a_touchpad_pull_past_the_start_stretches_the_content_and_it_returns() {
    let mut test = runner();
    drag(&mut test, 1, 100.0);
    let stretch = top(&test);
    assert!((5.0..9.0).contains(&stretch), "stretched {stretch}");
    wheel(&mut test, WheelSource::Pixel, TouchPhase::Ended, 0.0);
    let path = settle(&mut test, 900);
    assert!(path.iter().all(|y| *y >= 0.0), "never above the start");
    assert!(path.windows(2).all(|pair| pair[1] <= pair[0] + 0.01));
    assert_eq!(*path.last().unwrap(), 0.0);
}

#[test]
#[cfg(not(target_os = "macos"))]
fn a_flick_glides_the_distance_of_a_catalyst_scroll_view_and_stops_inside() {
    let mut test = runner();
    drag(&mut test, 30, -20.0);
    wheel(&mut test, WheelSource::Pixel, TouchPhase::Ended, 0.0);
    let released = top(&test);
    let path = settle(&mut test, 1200);
    let glide = released - *path.last().unwrap();
    assert!((230.0..600.0).contains(&glide), "glided {glide}");
    assert!(path.windows(2).all(|pair| pair[1] <= pair[0] + 0.01));
    assert!(*path.last().unwrap() > -LIMIT);
}

#[test]
#[cfg(not(target_os = "macos"))]
fn a_flick_into_the_end_bounces_and_rests_on_the_edge() {
    let mut test = runner();
    drag(&mut test, 20, -60.0);
    wheel(&mut test, WheelSource::Pixel, TouchPhase::Ended, 0.0);
    let path = settle(&mut test, 2500);
    assert!(
        path.iter().any(|y| *y < -LIMIT - 5.0),
        "went past the end by {}",
        path.iter().fold(0.0f32, |low, y| low.min(*y)) + LIMIT
    );
    assert!((*path.last().unwrap() + LIMIT).abs() < 0.5);
}

#[test]
fn a_mouse_wheel_does_not_stretch_the_content() {
    let mut test = runner();
    wheel(&mut test, WheelSource::Line, TouchPhase::Moved, 100.0);
    assert_eq!(top(&test), 0.0);
}

#[test]
fn a_table_stretches_past_its_start_under_a_touchpad_pull() {
    let mut test = runner_of(AppComponent::new(TableCase));
    let rest = first_row(&test);
    drag(&mut test, 1, 100.0);
    assert!(first_row(&test) > rest + 3.0);
    wheel(&mut test, WheelSource::Pixel, TouchPhase::Ended, 0.0);
    test.poll(Duration::from_millis(8), Duration::from_millis(900));
    assert!((first_row(&test) - rest).abs() < 0.01);
}

#[test]
fn a_pull_past_the_threshold_refreshes_holds_and_returns_when_it_is_done() {
    let (refreshing, calls) = (Rc::new(Cell::new(None)), Rc::new(Cell::new(0)));
    let mut test = runner_of(AppComponent::new(RefreshCase {
        refreshing: refreshing.clone(),
        calls: calls.clone(),
    }));
    drag(&mut test, 1, 40.0);
    assert_eq!(calls.get(), 0, "a short pull does not refresh");
    drag(&mut test, 6, 60.0);
    assert_eq!(calls.get(), 1, "one refresh for one pull");
    wheel(&mut test, WheelSource::Pixel, TouchPhase::Ended, 0.0);
    let held = *settle(&mut test, 1000).last().unwrap();
    assert!((held - 60.0).abs() < 1.0, "held at {held}");
    test.run_in(|| refreshing.get().unwrap().set(false));
    let path = settle(&mut test, 1000);
    assert!(path.last().unwrap().abs() < 0.5, "returned to {path:?}");
    assert_eq!(calls.get(), 1);
}

fn pan(test: &mut TestingRunner, from: f64, to: f64, steps: u32) {
    test.press_touch((200.0, from));
    for step in 1..=steps {
        let y = from + (to - from) * f64::from(step) / f64::from(steps);
        test.move_touch((200.0, y));
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn a_finger_pans_the_content_and_it_glides_after_the_release() {
    let mut test = runner();
    pan(&mut test, 250.0, 50.0, 20);
    let held = top(&test);
    assert!((-210.0..-150.0).contains(&held), "panned to {held}");
    test.release_touch((200.0, 50.0));
    let path = settle(&mut test, 1200);
    assert!(
        *path.last().unwrap() < held - 20.0,
        "glided on from {held} to {path:?}"
    );
    assert!(*path.last().unwrap() > -LIMIT_ANY);
}

#[test]
fn a_mouse_drag_does_not_pan_a_scroll_view() {
    let mut test = runner();
    test.press_cursor((200.0, 250.0));
    test.move_cursor((200.0, 50.0));
    test.release_cursor((200.0, 50.0));
    assert_eq!(top(&test), 0.0);
}

#[test]
fn a_finger_pulled_past_the_start_stretches_the_content_and_it_returns() {
    let mut test = runner();
    pan(&mut test, 50.0, 200.0, 15);
    assert!(top(&test) > 10.0, "stretched {}", top(&test));
    test.release_touch((200.0, 200.0));
    let path = settle(&mut test, 1000);
    assert!(path.last().unwrap().abs() < 0.5);
}

#[test]
fn scrolling_to_an_id_with_a_start_anchor_puts_its_view_on_the_top_edge() {
    let (mut test, handles) = positioned();
    test.run_in(|| {
        handles.controller.get().unwrap().scroll_to(
            ScrollPosition::id("row-20", Some(Alignment::Start)),
            Animation::None,
        )
    });
    test.sync_and_update();
    assert_eq!(target_top(&test), 0.0);
    assert_eq!(
        handles.controller.get().unwrap().content_offset(),
        (0.0, 800.0)
    );
}

#[test]
fn a_position_the_app_sets_in_its_state_scrolls_the_view_and_stays_unchanged() {
    let (mut test, handles) = positioned();
    let request = ScrollPosition::id("row-20", Some(Alignment::Start));
    test.run_in(|| handles.position.get().unwrap().set(request.clone()));
    test.poll(Duration::from_millis(8), Duration::from_millis(100));
    assert_eq!(target_top(&test), 0.0);
    assert_eq!(*handles.position.get().unwrap().peek(), request);
    test.run_in(|| {
        handles
            .position
            .get()
            .unwrap()
            .set(ScrollPosition::edge(Edge::Bottom))
    });
    test.poll(Duration::from_millis(8), Duration::from_millis(100));
    assert_eq!(
        handles.controller.get().unwrap().content_offset().1,
        LIMIT_ANY
    );
}

#[test]
fn a_scroll_by_the_user_is_written_back_to_the_state_as_a_point() {
    let (mut test, handles) = positioned();
    wheel(&mut test, WheelSource::Line, TouchPhase::Moved, -100.0);
    test.poll(Duration::from_millis(8), Duration::from_millis(100));
    let position = handles.position.get().unwrap().peek().clone();
    assert!(position.is_positioned_by_user());
    assert_eq!(
        position.point(),
        Some(handles.controller.get().unwrap().content_offset())
    );
    assert!(position.point().unwrap().1 > 0.0);
}

#[test]
fn the_geometry_callback_reports_the_offset_and_the_sizes() {
    let (mut test, handles) = positioned();
    test.run_in(|| {
        handles
            .controller
            .get()
            .unwrap()
            .scroll_to(ScrollPosition::at(0.0, 120.0), Animation::None)
    });
    test.poll(Duration::from_millis(8), Duration::from_millis(100));
    let geometry = handles.geometry.get().unwrap();
    assert_eq!(geometry.content_offset, (0.0, 120.0));
    assert_eq!(geometry.content_size.1, CONTENT_OF_ROWS);
    assert_eq!(geometry.container_size.1, VIEWPORT);
}

#[test]
fn an_animated_scroll_is_animating_until_it_lands_and_the_phase_callback_says_so() {
    let (mut test, handles) = positioned();
    let animation = Animation::Ease { duration_ms: 200 };
    test.run_in(|| {
        handles
            .controller
            .get()
            .unwrap()
            .scroll_to(ScrollPosition::at(0.0, 400.0), animation)
    });
    test.poll(Duration::from_millis(8), Duration::from_millis(60));
    assert_eq!(
        handles.controller.get().unwrap().phase(),
        ScrollPhase::Animating
    );
    test.poll(Duration::from_millis(8), Duration::from_millis(400));
    assert_eq!(handles.controller.get().unwrap().content_offset().1, 400.0);
    assert_eq!(
        *handles.phases.borrow(),
        [
            (ScrollPhase::Idle, ScrollPhase::Animating),
            (ScrollPhase::Animating, ScrollPhase::Idle)
        ]
    );
}

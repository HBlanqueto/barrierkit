use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use bk::ProbeExt;
use bk::controls::ScrollView;
use bk::prelude::*;
use bk::views::table::Table;
use freya_core::integration::{AppComponent, PlatformEvent, WheelEventName, WheelSource};
use freya_testing::TestingRunner;
use torin::prelude::{Area, Size2D};

const VIEWPORT: f32 = 300.0;
const CONTENT: f32 = 1600.0;
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

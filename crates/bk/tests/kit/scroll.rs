use std::time::Duration;

use bk::ProbeExt;
use bk::controls::ScrollView;
use bk::prelude::*;
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

fn runner() -> TestingRunner {
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Case),
        Size2D::new(400.0, VIEWPORT),
        |_| {},
        1.0,
    );
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

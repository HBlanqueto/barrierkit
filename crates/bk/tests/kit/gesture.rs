use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use bk::gesture::{
    Attachable, DragGesture, DragValue, LongPressGesture, MagnifyGesture, MagnifyValue, TapGesture,
    use_gesture_state,
};
use bk::modifiers::ViewModifiers;
use bk::prelude::*;
use freya_core::integration::{AppComponent, PlatformEvent, TouchEventName};
use freya_testing::TestingRunner;
use torin::prelude::Size2D;

#[derive(Default)]
struct Log {
    child_taps: Cell<u32>,
    parent_taps: Cell<u32>,
    drags: RefCell<Vec<DragValue>>,
    drag_ended: Cell<u32>,
    pressed: Cell<u32>,
    magnified: RefCell<Vec<f32>>,
    held: Cell<i32>,
}

struct Case {
    log: Rc<Log>,
    build: fn(&Rc<Log>) -> Element,
}

impl App for Case {
    fn render(&self) -> impl IntoElement {
        (self.build)(&self.log)
    }
}

fn runner(log: &Rc<Log>, build: fn(&Rc<Log>) -> Element) -> TestingRunner {
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Case {
            log: log.clone(),
            build,
        }),
        Size2D::new(200.0, 200.0),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    test.sync_and_update();
    test
}

fn bump(counter: &Cell<u32>) {
    counter.set(counter.get() + 1);
}

fn square() -> Rect {
    rect().width(Size::px(200.0)).height(Size::px(200.0))
}

fn tap_on_a_view(log: &Rc<Log>) -> Element {
    let taps = log.clone();
    square()
        .gesture(TapGesture::new().on_ended(move |_| bump(&taps.child_taps)))
        .into_element()
}

#[test]
fn a_click_is_a_tap_and_a_drag_is_not() {
    let log = Rc::new(Log::default());
    let mut test = runner(&log, tap_on_a_view);
    test.click_cursor((100.0, 100.0));
    assert_eq!(log.child_taps.get(), 1);
    test.press_cursor((100.0, 100.0));
    test.move_cursor((150.0, 100.0));
    test.release_cursor((150.0, 100.0));
    assert_eq!(log.child_taps.get(), 1, "a press that moved is no tap");
}

fn double_tap(log: &Rc<Log>) -> Element {
    let (single, double) = (log.clone(), log.clone());
    square()
        .gesture(
            TapGesture::new()
                .count(2)
                .on_ended(move |_| bump(&double.parent_taps))
                .exclusively(TapGesture::new().on_ended(move |_| bump(&single.child_taps))),
        )
        .into_element()
}

#[test]
fn a_double_tap_takes_two_clicks() {
    let log = Rc::new(Log::default());
    let mut test = runner(&log, double_tap);
    test.click_cursor((100.0, 100.0));
    assert_eq!(
        log.child_taps.get(),
        0,
        "the single tap waits for the second"
    );
    assert_eq!(log.parent_taps.get(), 0);
    test.click_cursor((100.0, 100.0));
    assert_eq!(log.parent_taps.get(), 1);
}

#[test]
fn a_single_tap_that_waited_for_a_second_fires_when_the_interval_passes() {
    let log = Rc::new(Log::default());
    let mut test = runner(&log, double_tap);
    test.click_cursor((100.0, 100.0));
    assert_eq!(log.child_taps.get(), 0);
    test.poll(Duration::from_millis(16), Duration::from_millis(700));
    assert_eq!((log.child_taps.get(), log.parent_taps.get()), (1, 0));
}

fn nested_taps(log: &Rc<Log>) -> Element {
    let (child, parent) = (log.clone(), log.clone());
    square()
        .gesture(TapGesture::new().on_ended(move |_| bump(&parent.parent_taps)))
        .child(
            rect()
                .width(Size::px(100.0))
                .height(Size::px(100.0))
                .gesture(TapGesture::new().on_ended(move |_| bump(&child.child_taps))),
        )
        .into_element()
}

#[test]
fn the_deepest_tap_wins_over_the_tap_of_its_ancestor() {
    let log = Rc::new(Log::default());
    let mut test = runner(&log, nested_taps);
    test.click_cursor((50.0, 50.0));
    assert_eq!((log.child_taps.get(), log.parent_taps.get()), (1, 0));
    test.click_cursor((150.0, 150.0));
    assert_eq!((log.child_taps.get(), log.parent_taps.get()), (1, 1));
}

fn simultaneous_taps(log: &Rc<Log>) -> Element {
    let (child, parent) = (log.clone(), log.clone());
    square()
        .simultaneous_gesture(TapGesture::new().on_ended(move |_| bump(&parent.parent_taps)))
        .child(
            rect()
                .width(Size::px(100.0))
                .height(Size::px(100.0))
                .gesture(TapGesture::new().on_ended(move |_| bump(&child.child_taps))),
        )
        .into_element()
}

#[test]
fn a_simultaneous_gesture_fires_with_its_descendant() {
    let log = Rc::new(Log::default());
    let mut test = runner(&log, simultaneous_taps);
    test.click_cursor((50.0, 50.0));
    assert_eq!((log.child_taps.get(), log.parent_taps.get()), (1, 1));
}

fn high_priority_tap(log: &Rc<Log>) -> Element {
    let (child, parent) = (log.clone(), log.clone());
    square()
        .high_priority_gesture(TapGesture::new().on_ended(move |_| bump(&parent.parent_taps)))
        .child(
            rect()
                .width(Size::px(100.0))
                .height(Size::px(100.0))
                .gesture(TapGesture::new().on_ended(move |_| bump(&child.child_taps))),
        )
        .into_element()
}

#[test]
fn a_high_priority_tap_beats_the_tap_of_its_descendant() {
    let log = Rc::new(Log::default());
    let mut test = runner(&log, high_priority_tap);
    test.click_cursor((50.0, 50.0));
    assert_eq!((log.child_taps.get(), log.parent_taps.get()), (0, 1));
}

fn drag_with_state(log: &Rc<Log>) -> Element {
    let (changed, ended, held) = (log.clone(), log.clone(), log.clone());
    let offset = use_gesture_state(0.0f32);
    held.held.set(offset.get() as i32);
    square()
        .gesture(
            DragGesture::new()
                .updating(offset, |value, state| *state = value.translation.0)
                .on_changed(move |value| changed.drags.borrow_mut().push(value))
                .on_ended(move |_| bump(&ended.drag_ended)),
        )
        .into_element()
}

#[test]
fn a_drag_reports_its_translation_and_its_gesture_state_returns_when_it_ends() {
    let log = Rc::new(Log::default());
    let mut test = runner(&log, drag_with_state);
    test.press_cursor((20.0, 20.0));
    test.move_cursor((25.0, 20.0));
    assert!(log.drags.borrow().is_empty(), "under the minimum distance");
    test.move_cursor((60.0, 20.0));
    test.sync_and_update();
    assert_eq!(log.held.get(), 40, "the state follows the drag");
    assert_eq!(log.drags.borrow().last().unwrap().translation, (40.0, 0.0));
    test.release_cursor((60.0, 20.0));
    test.sync_and_update();
    assert_eq!(log.drag_ended.get(), 1);
    assert_eq!(log.held.get(), 0, "the state is back");
}

fn long_press(log: &Rc<Log>) -> Element {
    let pressed = log.clone();
    let taps = log.clone();
    square()
        .gesture(
            LongPressGesture::new()
                .on_ended(move |_| bump(&pressed.pressed))
                .exclusively(TapGesture::new().on_ended(move |_| bump(&taps.child_taps))),
        )
        .into_element()
}

#[test]
fn a_long_press_ends_after_its_duration_and_a_short_press_is_a_tap() {
    let log = Rc::new(Log::default());
    let mut test = runner(&log, long_press);
    test.click_cursor((100.0, 100.0));
    assert_eq!((log.pressed.get(), log.child_taps.get()), (0, 1));
    test.press_cursor((100.0, 100.0));
    test.poll(Duration::from_millis(16), Duration::from_millis(700));
    assert_eq!(log.pressed.get(), 1);
}

fn magnify(log: &Rc<Log>) -> Element {
    let magnified = log.clone();
    square()
        .gesture(
            MagnifyGesture::new().on_changed(move |value: MagnifyValue| {
                magnified.magnified.borrow_mut().push(value.magnification)
            }),
        )
        .into_element()
}

fn finger(test: &mut TestingRunner, name: TouchEventName, id: u64, x: f64, phase: TouchPhase) {
    test.send_event(PlatformEvent::Touch {
        name,
        location: (x, 100.0).into(),
        finger_id: id,
        phase,
        force: None,
    });
    test.sync_and_update();
}

#[test]
fn two_fingers_moving_apart_magnify_the_view() {
    let log = Rc::new(Log::default());
    let mut test = runner(&log, magnify);
    finger(
        &mut test,
        TouchEventName::TouchStart,
        1,
        80.0,
        TouchPhase::Started,
    );
    finger(
        &mut test,
        TouchEventName::TouchStart,
        2,
        120.0,
        TouchPhase::Started,
    );
    finger(
        &mut test,
        TouchEventName::TouchMove,
        2,
        160.0,
        TouchPhase::Moved,
    );
    let seen = log.magnified.borrow().last().copied().unwrap();
    assert!((seen - 2.0).abs() < 0.01, "magnified by {seen}");
}

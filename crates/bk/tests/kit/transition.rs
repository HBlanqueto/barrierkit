use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use bk::ProbeExt;
use bk::animation::with_animation;
use bk::prelude::*;
use freya_core::integration::{App, AppComponent};
use freya_testing::TestingRunner;
use torin::prelude::{Area, Size2D};

const EASE: Animation = Animation::Ease { duration_ms: 200 };

struct Case {
    shown: Rc<Cell<Option<State<bool>>>>,
    transition: Transition,
    initially: bool,
}

impl App for Case {
    fn render(&self) -> impl IntoElement {
        let shown = use_state(|| self.initially);
        self.shown.set(Some(shown));
        VStack::new()
            .spacing(10.0)
            .child(rect().width(Size::px(60.0)).height(Size::px(20.0)))
            .child(
                Presence::new((*shown.read()).then(|| {
                    rect()
                        .probe("box")
                        .width(Size::px(60.0))
                        .height(Size::px(20.0))
                }))
                .transition(self.transition),
            )
            .child(
                rect()
                    .probe("after")
                    .width(Size::px(60.0))
                    .height(Size::px(20.0)),
            )
    }
}

fn runner(
    transition: Transition,
    initially: bool,
) -> (TestingRunner, Rc<Cell<Option<State<bool>>>>) {
    let shown = Rc::new(Cell::new(None));
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Case {
            shown: shown.clone(),
            transition,
            initially,
        }),
        Size2D::new(200.0, 200.0),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    test.sync_and_update();
    (test, shown)
}

fn set(
    test: &mut TestingRunner,
    shown: &Rc<Cell<Option<State<bool>>>>,
    value: bool,
    animated: bool,
) {
    let mut state = shown.get().expect("the case rendered");
    test.run_in(|| {
        if animated {
            with_animation(EASE, || state.set(value));
        } else {
            state.set(value);
        }
    });
}

fn area(test: &TestingRunner, id: &str) -> Option<Area> {
    test.find(|node, element| {
        (element.accessibility().builder.author_id() == Some(id))
            .then(|| -> Area { node.layout().area })
    })
}

fn opacity(test: &TestingRunner) -> Option<f32> {
    test.find(|_, element| element.effect().and_then(|effect| effect.opacity))
}

fn advance(test: &mut TestingRunner, milliseconds: u64) {
    test.poll(
        Duration::from_millis(8),
        Duration::from_millis(milliseconds),
    );
}

#[test]
fn a_view_inserted_inside_an_animation_fades_in_and_ends_at_rest() {
    let (mut test, shown) = runner(Transition::OPACITY, false);
    assert!(area(&test, "box").is_none());
    set(&mut test, &shown, true, true);
    advance(&mut test, 80);
    let middle = opacity(&test).expect("the presence wraps the view");
    assert!((0.05..0.95).contains(&middle), "{middle}");
    advance(&mut test, 400);
    assert_eq!(opacity(&test), Some(1.0));
}

#[test]
fn a_view_removed_inside_an_animation_stays_mounted_until_it_ends() {
    let (mut test, shown) = runner(Transition::OPACITY, true);
    set(&mut test, &shown, false, true);
    advance(&mut test, 80);
    assert!(area(&test, "box").is_some());
    let middle = opacity(&test).expect("still wrapped");
    assert!((0.05..0.95).contains(&middle), "{middle}");
    advance(&mut test, 400);
    assert!(area(&test, "box").is_none());
}

#[test]
fn without_an_animation_a_view_comes_and_goes_at_once() {
    let (mut test, shown) = runner(Transition::OPACITY, true);
    set(&mut test, &shown, false, false);
    advance(&mut test, 16);
    assert!(area(&test, "box").is_none());
    set(&mut test, &shown, true, false);
    advance(&mut test, 16);
    assert_eq!(opacity(&test), Some(1.0));
}

#[test]
fn a_view_that_is_absent_takes_no_room_in_a_stack() {
    let (test, _) = runner(Transition::OPACITY, false);
    assert_eq!(area(&test, "after").expect("after").min_y(), 30.0);
}

#[test]
fn a_view_moving_in_from_an_edge_starts_one_width_outside_and_lands_in_place() {
    let (mut test, shown) = runner(Transition::move_edge(Edge::Leading), false);
    set(&mut test, &shown, true, true);
    advance(&mut test, 40);
    let early = area(&test, "box").expect("box").min_x();
    assert!(early < -10.0, "{early}");
    advance(&mut test, 400);
    assert_eq!(area(&test, "box").expect("box").min_x(), 0.0);
}

struct Swap {
    on: Rc<Cell<Option<State<bool>>>>,
}

impl App for Swap {
    fn render(&self) -> impl IntoElement {
        let on = use_state(|| false);
        self.on.set(Some(on));
        let (id, width) = if *on.read() {
            ("new", 40.0)
        } else {
            ("old", 30.0)
        };
        rect()
            .probe(id)
            .width(Size::px(width))
            .height(Size::px(20.0))
            .content_transition(ContentTransition::Opacity)
    }
}

#[test]
fn a_changed_view_crossfades_with_the_old_one_until_the_animation_ends() {
    let on = Rc::new(Cell::new(None));
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Swap { on: on.clone() }),
        Size2D::new(200.0, 200.0),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    test.sync_and_update();
    assert!(area(&test, "old").is_some());
    let mut state = on.get().expect("rendered");
    test.run_in(|| with_animation(EASE, || state.set(true)));
    advance(&mut test, 80);
    assert!(area(&test, "old").is_some() && area(&test, "new").is_some());
    advance(&mut test, 400);
    assert!(area(&test, "old").is_none() && area(&test, "new").is_some());
}

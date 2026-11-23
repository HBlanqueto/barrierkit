use std::cell::Cell;
use std::rc::Rc;

use bk::modifiers::ViewModifiers;
use freya_core::integration::{App, AppComponent};
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Size, Size2D};

struct Case {
    hovering: Rc<Cell<bool>>,
    taps: Rc<Cell<u32>>,
}

impl App for Case {
    fn render(&self) -> impl IntoElement {
        let (hovering, taps) = (self.hovering.clone(), self.taps.clone());
        rect()
            .child(
                rect()
                    .width(Size::px(100.0))
                    .height(Size::px(100.0))
                    .on_hover(move |inside| hovering.set(inside)),
            )
            .child(
                rect()
                    .width(Size::px(100.0))
                    .height(Size::px(100.0))
                    .on_tap_gesture(2, move |_| taps.set(taps.get() + 1)),
            )
    }
}

fn runner(hovering: &Rc<Cell<bool>>, taps: &Rc<Cell<u32>>) -> TestingRunner {
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Case {
            hovering: hovering.clone(),
            taps: taps.clone(),
        }),
        Size2D::new(200.0, 200.0),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    test
}

#[test]
fn on_hover_reports_the_pointer_entering_and_leaving() {
    let (hovering, taps) = (Rc::default(), Rc::default());
    let mut test = runner(&hovering, &taps);
    test.move_cursor((50.0, 50.0));
    assert!(hovering.get());
    test.move_cursor((150.0, 50.0));
    assert!(!hovering.get());
}

#[test]
fn on_tap_gesture_fires_after_its_count_of_taps() {
    let (hovering, taps) = (Rc::default(), Rc::default());
    let mut test = runner(&hovering, &taps);
    test.click_cursor((50.0, 150.0));
    assert_eq!(taps.get(), 0);
    test.click_cursor((50.0, 150.0));
    assert_eq!(taps.get(), 1);
}

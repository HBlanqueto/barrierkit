use std::cell::Cell;
use std::rc::Rc;

use bk::modifiers::ViewModifiers;
use freya_core::integration::{App, AppComponent};
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Size, Size2D};

struct Case {
    hovering: Rc<Cell<bool>>,
}

impl App for Case {
    fn render(&self) -> impl IntoElement {
        let hovering = self.hovering.clone();
        rect().child(
            rect()
                .width(Size::px(100.0))
                .height(Size::px(100.0))
                .on_hover(move |inside| hovering.set(inside)),
        )
    }
}

#[test]
fn on_hover_reports_the_pointer_entering_and_leaving() {
    let hovering = Rc::<Cell<bool>>::default();
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Case {
            hovering: hovering.clone(),
        }),
        Size2D::new(200.0, 200.0),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    test.move_cursor((50.0, 50.0));
    assert!(hovering.get());
    test.move_cursor((150.0, 50.0));
    assert!(!hovering.get());
}

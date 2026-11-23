use bk::ProbeExt;
use bk::prelude::*;
use freya_core::integration::{App, AppComponent};
use freya_testing::TestingRunner;
use torin::prelude::Size2D;

struct Case;

impl App for Case {
    fn render(&self) -> impl IntoElement {
        VStack::new()
            .spacing(0.0)
            .child(rect().height(Size::px(10.0)))
            .child(
                rect()
                    .probe("inside")
                    .width(Size::px(60.0))
                    .height(Size::px(20.0))
                    .glass_effect(Glass::CLEAR, Shape::Capsule),
            )
    }
}

#[test]
fn a_glass_effect_wraps_its_view_without_moving_or_resizing_it() {
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Case),
        Size2D::new(200.0, 100.0),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    test.sync_and_update();
    let area = crate::support::area(&test, "inside");
    assert_eq!(
        (area.min_x(), area.min_y(), area.width(), area.height()),
        (0.0, 10.0, 60.0, 20.0)
    );
}

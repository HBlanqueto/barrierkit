use barrier_kit::controls::Switch;
use barrier_kit::views::form::{Form, Row, Section};
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Area, Size2D};

struct Case;

impl App for Case {
    fn render(&self) -> impl IntoElement {
        Form::new().section(
            Section::titled("Ventana")
                .row(Row::new("Mantener encima").control(Switch::new("Interruptor"))),
        )
    }
}

fn switch_area(width: f32) -> Area {
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Case),
        Size2D::new(width, 400.0),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    test.sync_and_update();
    test.find(|node, el| {
        (el.accessibility().builder.label() == Some("Interruptor")).then(|| node.layout().area)
    })
    .expect("switch")
}

#[test]
fn a_narrow_form_puts_the_control_below_its_label_on_the_left() {
    assert!(switch_area(300.0).min_x() < 100.0);
}

#[test]
fn a_wide_form_keeps_the_control_on_the_right_of_its_label() {
    assert!(switch_area(800.0).max_x() > 700.0);
}

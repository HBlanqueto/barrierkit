//! Tests for the vertical titlebars and the titlebar light position.

use barrier_kit::chrome::{ChromeStyle, TitlebarLayout, WindowChrome};
use barrier_kit::titlebar::Titlebar;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Area, Size, Size2D};
use vow_design::TITLEBAR;

struct Case(ChromeStyle);

impl App for Case {
    fn render(&self) -> impl IntoElement {
        WindowChrome::new(
            self.0,
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .child(Titlebar::new().title("Rótulo")),
        )
    }
}

fn light_groups(style: ChromeStyle) -> Vec<Area> {
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Case(style)),
        Size2D::new(900.0, 400.0),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    let diameter = TITLEBAR.buttons.diameter as f32;
    let length = 3.0 * diameter + 2.0 * TITLEBAR.buttons.spacing as f32;
    test.find_many(|node, _| {
        let area = node.layout().area;
        let size = (area.width(), area.height());
        (size == (length, diameter) || size == (diameter, length)).then_some(area)
    })
}

#[test]
fn the_titlebar_lights_start_where_the_compositor_draws_them() {
    let groups = light_groups(ChromeStyle::default());
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].min_x(), TITLEBAR.buttons.margin_start as f32);
}

#[test]
fn unified_without_a_sidebar_falls_back_to_a_rail_with_one_set_of_vertical_lights() {
    let groups = light_groups(ChromeStyle {
        layout: TitlebarLayout::Unified,
        ..Default::default()
    });
    assert_eq!(groups.len(), 1, "got {groups:?}");
    assert!(
        groups[0].height() > groups[0].width(),
        "the lights are stacked"
    );
}

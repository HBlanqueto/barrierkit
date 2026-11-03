//! Tests for the vertical titlebars and the titlebar light position.

use barrier_kit::chrome::{ChromeStyle, RailStyle, TitlebarLayout, WindowChrome};
use barrier_kit::compact::IconButton;
use barrier_kit::symbol::Symbol;
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
    let mut groups = test.find_many(|node, _| {
        let area = node.layout().area;
        let size = (area.width(), area.height());
        (size == (length, diameter) || size == (diameter, length)).then_some(area)
    });
    groups.dedup_by(|a, b| a.origin == b.origin);
    groups
}

#[test]
fn the_titlebar_lights_start_where_the_compositor_draws_them() {
    let groups = light_groups(ChromeStyle::default());
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].min_x(), TITLEBAR.buttons.margin_start as f32);
}

#[test]
fn a_unified_rail_without_a_sidebar_falls_back_to_a_strip_with_one_set_of_vertical_lights() {
    let groups = light_groups(ChromeStyle {
        layout: TitlebarLayout::Rail,
        rail_style: RailStyle::Unified,
        ..Default::default()
    });
    assert_eq!(groups.len(), 1, "got {groups:?}");
    assert!(
        groups[0].height() > groups[0].width(),
        "the lights are stacked"
    );
}

struct Uneven;

impl App for Uneven {
    fn render(&self) -> impl IntoElement {
        rect().width(Size::fill()).height(Size::fill()).child(
            Titlebar::new()
                .title("Escuchando")
                .right(IconButton::new(Symbol::ListBullet, "Cola")),
        )
    }
}

#[test]
fn the_title_is_centred_in_the_window_whatever_each_side_holds() {
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Uneven),
        Size2D::new(900.0, 400.0),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    test.sync_and_update();
    let title = test
        .find(|node, el| {
            (el.accessibility().builder.label() == Some("titlebar-title"))
                .then(|| node.layout().area)
        })
        .expect("title");
    assert!(
        (title.center().x - 450.0).abs() < 0.5,
        "title centre {}",
        title.center().x
    );
}

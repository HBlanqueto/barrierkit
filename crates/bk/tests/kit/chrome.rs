use bk::chrome::{RailStyle, TitlebarLayout, WindowChrome};
use bk::compact::IconButton;
use bk::styles::WindowChromeStyle;
use bk::symbol::Symbol;
use bk::titlebar::{Titlebar, WindowToolbarStyle};
use bk::views::navigation::{History, NavigationButtons};
use ctdesign::TITLEBAR;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Area, Size2D};

struct Case(TitlebarLayout, RailStyle);

impl App for Case {
    fn render(&self) -> impl IntoElement {
        let Self(layout, rail_style) = *self;
        WindowChrome::new(rect().expanded().child(Titlebar::new().title("Rótulo"))).style(
            move |s| WindowChromeStyle {
                layout,
                rail_style,
                ..s
            },
        )
    }
}

fn light_groups(layout: TitlebarLayout, rail_style: RailStyle) -> Vec<Area> {
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Case(layout, rail_style)),
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
fn the_compact_titlebar_lights_sit_12_from_the_left_and_centred_in_the_bar() {
    let groups = light_groups(TitlebarLayout::Top, RailStyle::Normal);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].min_x(), 12.0);
    assert_eq!(groups[0].min_y(), 13.0);
}

#[test]
fn a_unified_rail_without_a_sidebar_falls_back_to_a_strip_with_one_set_of_vertical_lights() {
    let groups = light_groups(TitlebarLayout::Rail, RailStyle::Unified);
    assert_eq!(groups.len(), 1, "got {groups:?}");
    assert!(
        groups[0].height() > groups[0].width(),
        "the lights are stacked"
    );
}

struct Uneven;

impl App for Uneven {
    fn render(&self) -> impl IntoElement {
        rect().expanded().child(
            Titlebar::new()
                .toolbar_style(WindowToolbarStyle::UnifiedCompact)
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
            (el.accessibility().builder.author_id() == Some("titlebar-title"))
                .then(|| node.layout().area)
        })
        .expect("title");
    assert!(
        (title.center().x - 450.0).abs() < 0.5,
        "title centre {}",
        title.center().x
    );
}

struct Items(bool);

impl App for Items {
    fn render(&self) -> impl IntoElement {
        let history = use_state(|| {
            let mut history = History::new("a");
            history.push("b");
            history
        });
        rect().expanded().child(
            Titlebar::new()
                .with_sidebar(self.0)
                .toolbar_style(if self.0 {
                    WindowToolbarStyle::Unified
                } else {
                    WindowToolbarStyle::UnifiedCompact
                })
                .left(NavigationButtons::new(history))
                .right(IconButton::new(Symbol::Plus, "Añadir")),
        )
    }
}

type Sizes = Vec<(f32, f32)>;

fn item_sizes(sidebar: bool) -> (Sizes, Sizes) {
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Items(sidebar)),
        Size2D::new(900.0, 400.0),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    test.sync_and_update();
    let sizes = |wanted: [(f32, f32); 2]| {
        test.find_many(|node, _| {
            let area = node.layout().area;
            let size = (area.width(), area.height());
            wanted.contains(&size).then_some(size)
        })
    };
    (
        sizes([(63.0, 24.0), (76.0, 38.0)]),
        sizes([(26.0, 24.0), (38.0, 38.0)]),
    )
}

#[test]
fn the_compact_bar_holds_24_high_platters_and_the_unified_bar_38_high_ones() {
    for (sidebar, pair, button) in [
        (false, (63.0, 24.0), (26.0, 24.0)),
        (true, (76.0, 38.0), (38.0, 38.0)),
    ] {
        let (pairs, buttons) = item_sizes(sidebar);
        assert!(!pairs.is_empty(), "no arrows of {pair:?}");
        assert!(pairs.iter().all(|size| *size == pair), "{pairs:?}");
        assert!(buttons.len() >= 3, "two arrows and the plus: {buttons:?}");
        assert!(buttons.iter().all(|size| *size == button), "{buttons:?}");
    }
}

use bk::ProbeExt;
use bk::platform::WindowStatus;
use bk::prelude::{ToolbarItem, ToolbarItemPlacement, ToolbarSpacer};
use bk::sidebar::Sidebar;
use bk::theme;
use bk::titlebar::Titlebar;
use bk::views::split::SplitView;
use ctdesign::SIDEBAR;
use freya_components::theming::hooks::use_init_theme;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Size, Size2D};

fn marker(id: &'static str) -> Rect {
    rect()
        .width(Size::px(20.0))
        .height(Size::px(10.0))
        .probe(id)
}

struct ToolbarCase;

impl App for ToolbarCase {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(false));
        Titlebar::preview(WindowStatus::default(), "Ventana", false)
            .toolbar(ToolbarItem::new(
                ToolbarItemPlacement::PrimaryAction,
                marker("primary"),
            ))
            .toolbar(ToolbarItem::new(
                ToolbarItemPlacement::Principal,
                marker("principal"),
            ))
            .toolbar(ToolbarSpacer::flexible())
            .toolbar(ToolbarItem::new(
                ToolbarItemPlacement::Navigation,
                marker("navigation"),
            ))
    }
}

fn x_of(test: &TestingRunner, id: &str) -> Option<f32> {
    test.find(|node, el| {
        (el.accessibility().builder.author_id() == Some(id)).then(|| node.layout().area.origin.x)
    })
}

#[test]
fn toolbar_items_sit_by_placement_and_a_principal_item_replaces_the_title() {
    let app = AppComponent::new(ToolbarCase);
    let (mut test, _) = TestingRunner::new(app, Size2D::new(600.0, 120.0), |_| {}, 1.0);
    test.sync_and_update();
    test.sync_and_update();
    let navigation = x_of(&test, "navigation").expect("navigation item");
    let principal = x_of(&test, "principal").expect("principal item");
    let primary = x_of(&test, "primary").expect("primary item");
    assert!(
        navigation < principal && principal < primary,
        "{navigation} {principal} {primary}"
    );
    let title = test.find(|_, el| Label::try_downcast(el).filter(|label| label.text == "Ventana"));
    assert!(
        title.is_none(),
        "the principal item takes the title's place"
    );
}

struct ThreeColumns;

impl App for ThreeColumns {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(false));
        SplitView::new(Sidebar::new(), rect().expanded().probe("detail"))
            .content(rect().expanded().probe("content"))
    }
}

fn area_of(test: &TestingRunner, id: &str) -> (f32, f32) {
    test.find(|node, el| {
        (el.accessibility().builder.author_id() == Some(id)).then(|| {
            let area = node.layout().area;
            (area.origin.x, area.size.width)
        })
    })
    .unwrap_or_else(|| panic!("{id} exists"))
}

#[test]
fn a_content_column_sits_between_the_sidebar_and_the_detail_and_resizes() {
    let app = AppComponent::new(ThreeColumns);
    let (mut test, _) = TestingRunner::new(app, Size2D::new(900.0, 600.0), |_| {}, 1.0);
    test.sync_and_update();
    test.poll(
        std::time::Duration::from_millis(16),
        std::time::Duration::from_millis(400),
    );
    let sidebar = SIDEBAR.width as f32;
    let (content_x, content_width) = area_of(&test, "content");
    assert_eq!((content_x, content_width), (sidebar, sidebar));
    assert!(area_of(&test, "detail").0 >= content_x + content_width);

    let edge = f64::from(content_x + content_width) - 1.0;
    test.move_cursor((edge, 300.0));
    test.press_cursor((edge, 300.0));
    test.sync_and_update();
    test.move_cursor((edge + 30.0, 300.0));
    test.sync_and_update();
    test.sync_and_update();
    let (_, width) = area_of(&test, "content");
    let grip_centre = content_x + width - ctdesign::SPLIT_VIEW.grip / 2.0;
    assert!((grip_centre - (edge + 30.0) as f32).abs() < 1.0);
    test.release_cursor((edge + 30.0, 300.0));
}

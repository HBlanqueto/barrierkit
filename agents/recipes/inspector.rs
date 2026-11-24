use bk::compact::IconButton;
use bk::prelude::*;
use bk::sidebar::{Sidebar, SidebarItem};
use bk::views::split::SplitView;

fn band(color: (u8, u8, u8)) -> Rect {
    rect()
        .width(Size::fill())
        .height(Size::percent(50.0))
        .background(color)
}

fn bands() -> Element {
    VStack::new()
        .spacing(0.0)
        .expanded()
        .child(band((220, 40, 40)))
        .child(band((40, 90, 220)))
        .into_element()
}

fn main() -> Result<(), RunError> {
    Application::new("com.example.photos")
        .window("Photos", || {
            let mut inspector = use_state(|| true);
            let selection = use_state(|| 0);
            let detail = ZStack::new()
                .expanded()
                .alignment(ContentAlignment::TOP)
                .child(bands())
                .child(
                    Titlebar::new()
                        .title("Photos")
                        .with_sidebar(true)
                        .toolbar(ToolbarItem::new(
                            ToolbarItemPlacement::PrimaryAction,
                            ToolGroup::new().child(
                                IconButton::new(Symbol::SidebarRight, "Inspector")
                                    .on_press(move |_| inspector.toggle()),
                            ),
                        )),
                );
            SplitView::new(
                Sidebar::new().traffic_lights(true).child(
                    SidebarItem::new("Library")
                        .symbol(Symbol::Photo)
                        .tag(selection, 0),
                ),
                detail,
            )
            .background_extension(bands())
            .inspector(
                rect().expanded().center().child(Text::new("Inspector")),
                *inspector.read(),
            )
            .sidebar_width(220.0)
        })
        .options(|window| {
            window
                .size((760, 480))
                .window_style(WindowStyle::HiddenTitleBar)
        })
        .run()
}

use bk::compact::IconButton;
use bk::prelude::*;
use bk::sidebar::{Sidebar, SidebarItem};
use bk::views::feedback::ContentUnavailableView;
use bk::views::split::SplitView;

fn main() -> Result<(), RunError> {
    let style = match std::env::args().nth(1).as_deref() {
        Some("title-bar") => WindowStyle::TitleBar,
        Some("plain") => WindowStyle::Plain,
        _ => WindowStyle::HiddenTitleBar,
    };
    run_window(
        || {
            let mut page = use_state(|| 0);
            let item = |index: usize, title: &'static str, glyph: Symbol| {
                SidebarItem::new(title)
                    .symbol(glyph)
                    .selected(*page.read() == index)
                    .on_select(move |_| page.set(index))
            };
            let detail = rect()
                .expanded()
                .direction(Direction::Vertical)
                .child(
                    Titlebar::new()
                        .title("Notes")
                        .with_sidebar(true)
                        .toolbar(ToolbarItem::new(
                            ToolbarItemPlacement::PrimaryAction,
                            ToolGroup::new()
                                .child(IconButton::new(Symbol::SquareAndPencil, "New note"))
                                .child(IconButton::new(Symbol::Ellipsis, "More")),
                        )),
                )
                .child(ContentUnavailableView::new(
                    "No Note Selected",
                    Symbol::DocText,
                ));
            SplitView::new(
                Sidebar::new()
                    .traffic_lights(true)
                    .child(item(0, "All Notes", Symbol::DocText))
                    .child(item(1, "Recent", Symbol::Clock))
                    .child(item(2, "Starred", Symbol::Star)),
                detail,
            )
            .sidebar_width(220.0)
        },
        WindowOptions::new("Notes", "notes")
            .size((760, 480))
            .window_style(style),
    )
}

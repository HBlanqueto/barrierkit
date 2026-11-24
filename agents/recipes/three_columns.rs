use bk::compact::IconButton;
use bk::prelude::*;
use bk::sidebar::{Sidebar, SidebarItem};
use bk::titlebar::sidebar_bar_height;
use bk::views::split::SplitView;
use torin::prelude::Gaps;

const NOTES: [&str; 3] = ["Groceries", "Trip to Lisbon", "Ideas"];

fn main() -> Result<(), RunError> {
    Application::new("com.example.notes")
        .window("Notes", || {
            let mut note = use_state(|| 0);
            let selection = use_state(|| 0);
            let content = rect()
                .expanded()
                .direction(Direction::Vertical)
                .padding(Gaps::new(sidebar_bar_height(), 12.0, 0.0, 12.0))
                .children(NOTES.iter().enumerate().map(|(index, title)| {
                    rect()
                        .width(Size::fill())
                        .padding(Gaps::new(8.0, 0.0, 8.0, 0.0))
                        .on_press(move |_| note.set(index))
                        .child(Text::new(*title))
                }));
            let detail = VStack::new()
                .expanded()
                .child(
                    Titlebar::new()
                        .title(NOTES[*note.read()])
                        .with_sidebar(true)
                        .toolbar(ToolbarItem::new(
                            ToolbarItemPlacement::PrimaryAction,
                            ToolGroup::new()
                                .child(IconButton::new(Symbol::SquareAndPencil, "New note")),
                        )),
                )
                .child(
                    rect()
                        .expanded()
                        .center()
                        .child(Text::new(NOTES[*note.read()]).font(TITLE)),
                );
            SplitView::new(
                Sidebar::new().traffic_lights(true).child(
                    SidebarItem::new("All Notes")
                        .symbol(Symbol::DocText)
                        .tag(selection, 0),
                ),
                detail,
            )
            .content(content)
            .sidebar_width(200.0)
        })
        .options(|window| {
            window
                .size((860, 480))
                .window_style(WindowStyle::HiddenTitleBar)
        })
        .run()
}

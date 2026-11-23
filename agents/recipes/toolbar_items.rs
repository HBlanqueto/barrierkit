use bk::compact::IconButton;
use bk::prelude::*;
use bk::views::navigation::{History, NavigationButtons};

fn main() -> Result<(), RunError> {
    run_window(
        || {
            let mut history = use_state(|| History::new("Inbox"));
            let page = *history.read().current();
            VStack::new()
                .expanded()
                .child(
                    Titlebar::new()
                        .toolbar(ToolbarItem::new(
                            ToolbarItemPlacement::Navigation,
                            NavigationButtons::new(history),
                        ))
                        .toolbar(ToolbarItem::new(
                            ToolbarItemPlacement::Principal,
                            Text::new(page).font(HEADLINE),
                        ))
                        .toolbar(ToolbarItem::new(
                            ToolbarItemPlacement::Automatic,
                            ToolGroup::new()
                                .child(IconButton::new(Symbol::MagnifyingGlass, "Search")),
                        ))
                        .toolbar(ToolbarSpacer::flexible())
                        .toolbar(ToolbarItem::new(
                            ToolbarItemPlacement::PrimaryAction,
                            ToolGroup::new().child(
                                IconButton::new(Symbol::Plus, "New")
                                    .on_press(move |_| history.write().push("Drafts")),
                            ),
                        )),
                )
                .child(
                    rect()
                        .expanded()
                        .center()
                        .child(Text::new(page).font(TITLE)),
                )
        },
        WindowOptions::new("Mail", "mail")
            .size((760, 480))
            .window_style(WindowStyle::Plain),
    )
}

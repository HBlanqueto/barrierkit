use bk::platform::WindowControls;
use bk::prelude::*;

fn main() -> Result<(), RunError> {
    let style = match std::env::args().nth(1).as_deref() {
        Some("title-bar") => WindowStyle::TitleBar,
        Some("plain") => WindowStyle::Plain,
        _ => WindowStyle::HiddenTitleBar,
    };
    let toolbar_style = match std::env::args().nth(2).as_deref() {
        Some("compact") => WindowToolbarStyle::UnifiedCompact,
        Some("expanded") => WindowToolbarStyle::Expanded,
        _ => WindowToolbarStyle::Unified,
    };
    run_window(
        move || {
            let controls = WindowControls::get();
            let status = *controls.status.read();
            // One line per change, for `scripts/macos-fullscreen.sh`.
            use_side_effect(move || {
                eprintln!("full-screen: {}", controls.status.read().fullscreen)
            });
            rect()
                .expanded()
                .direction(Direction::Vertical)
                .child(
                    Titlebar::new()
                        .title("Full screen")
                        .toolbar_style(toolbar_style)
                        .toolbar(ToolbarItem::new(
                            ToolbarItemPlacement::Navigation,
                            ToolGroup::new().child(IconButton::new(Symbol::ChevronLeft, "Back")),
                        ))
                        .toolbar(ToolbarItem::new(
                            ToolbarItemPlacement::PrimaryAction,
                            ToolGroup::new()
                                .child(IconButton::new(Symbol::Plus, "New"))
                                .child(IconButton::new(Symbol::SquareAndArrowUp, "Share")),
                        )),
                )
                .child(
                    rect()
                        .expanded()
                        .center()
                        .direction(Direction::Vertical)
                        .spacing(12.0)
                        .child(Text::new(if status.fullscreen {
                            "In full screen"
                        } else {
                            "In a window"
                        }))
                        .child(
                            Button::new("Toggle Full Screen")
                                .on_press(move |_| controls.toggle_fullscreen()),
                        ),
                )
        },
        WindowOptions::new("Full screen", "com.example.full-screen")
            .size((520, 320))
            .window_style(style),
    )
}

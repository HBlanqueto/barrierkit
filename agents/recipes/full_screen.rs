use bk::platform::WindowControls;
use bk::prelude::*;

fn main() -> Result<(), RunError> {
    let style = match std::env::args().nth(1).as_deref() {
        Some("hidden-title-bar") => WindowStyle::HiddenTitleBar,
        Some("plain") => WindowStyle::Plain,
        _ => WindowStyle::TitleBar,
    };
    run_window(
        || {
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
                        .toolbar_style(WindowToolbarStyle::Unified),
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
        WindowOptions::new("Full screen", "full-screen")
            .size((520, 320))
            .window_style(style),
    )
}

use bk::patterns::settings::{Pane, PaneGroup, SettingsWindow};
use bk::prelude::*;
use ctdesign::{ACCENT_SWATCHES, SETTINGS};

fn main() -> Result<(), RunError> {
    run_window(
        || {
            let open_at_login = use_state(|| false);
            let startup = Form::new().section(
                Section::titled("Startup").row(
                    Row::new("Open at login")
                        .keywords(&["launch", "boot"])
                        .control(Switch::new("Open at login", open_at_login)),
                ),
            );
            let general = Pane::new(
                "general",
                "General",
                Symbol::CircleHalf,
                ACCENT_SWATCHES[0].1,
                startup,
            );
            SettingsWindow::new()
                .restoration("com.example.notes")
                .group(PaneGroup::new(None).pane(general))
        },
        WindowOptions::new("Settings", "com.example.notes")
            .size((SETTINGS.window_width, SETTINGS.window_height)),
    )
}

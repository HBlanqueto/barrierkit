use bk::patterns::settings::{Pane, PaneGroup, SettingsWindow};
use bk::prelude::*;
use bk::views::form::{Form, Row, Section};
use ctdesign::{ACCENT_SWATCHES, SETTINGS};

fn main() -> Result<(), RunError> {
    let startup = Form::new().section(
        Section::titled("Startup").row(
            Row::new("Open at login")
                .keywords(&["launch", "boot"])
                .control(Switch::new("Open at login")),
        ),
    );
    let general = Pane::new(
        "general",
        "General",
        Symbol::CircleHalf,
        ACCENT_SWATCHES[0].1,
        startup,
    );
    let group = PaneGroup::new(None).pane(general);
    run_window(
        move || {
            SettingsWindow::new()
                .restoration("notes")
                .group(group.clone())
        },
        WindowOptions::new("Settings", "notes")
            .size((SETTINGS.window_width, SETTINGS.window_height)),
    )
}

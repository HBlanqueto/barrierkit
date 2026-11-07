use bk::prelude::*;

fn main() -> Result<(), RunError> {
    run_window(
        || {
            rect()
                .expanded()
                .background(color(Palette::current().window_background))
                .child(Titlebar::new().title("Notes").with_sidebar(true))
                .child(Text::new("No notes yet").padding(SPACE_4))
        },
        WindowOptions::new("Notes", "notes"),
    )
}

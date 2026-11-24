use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("notes")
        .window("Notes", || {
            rect()
                .expanded()
                .child(Titlebar::new().title("Notes"))
                .child(Text::new("No notes yet").padding(SPACE_4))
        })
        .options(|window| window.window_style(WindowStyle::HiddenTitleBar))
        .run()
}

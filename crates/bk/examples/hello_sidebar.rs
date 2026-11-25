use bk::prelude::*;

fn main() -> Result<(), RunError> {
    run_window(
        || {
            SplitView::new(
                Sidebar::new().traffic_lights(true),
                Text::new("Hello, world").font(TITLE).padding(40.),
            )
        },
        WindowOptions::new("Hello", "com.example.hello_sidebar")
            .size((520, 320))
            .window_style(WindowStyle::HiddenTitleBar),
    )
}

use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("hello")
        .window("Hello", || {
            Text::new("Hello, world").font(TITLE).padding(40.)
        })
        .run()
}

use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("com.example.hello")
        .window("Hello", || {
            Text::new("Hello, world").font(TITLE).padding(40.)
        })
        .run()
}

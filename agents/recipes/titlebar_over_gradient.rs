use bk::backdrop::{Backdrop, BackdropSource};
use bk::compact::IconButton;
use bk::prelude::*;
use ctdesign::BACKDROP_SCENES;

#[derive(PartialEq)]
struct Hello;

impl Component for Hello {
    fn render(&self) -> impl IntoElement {
        VStack::new()
            .expanded()
            .child(
                Titlebar::new().title("Hello").right(
                    ToolGroup::new()
                        .child(IconButton::new(Symbol::Plus, "Add"))
                        .child(IconButton::new(Symbol::Ellipsis, "More")),
                ),
            )
            .child(
                rect()
                    .expanded()
                    .center()
                    .child(Text::new("Hello, world").font(TITLE)),
            )
    }
}

fn main() -> Result<(), RunError> {
    run_window(
        || Backdrop::new(BackdropSource::Colors(BACKDROP_SCENES.dusk), Hello),
        WindowOptions::new("Hello", "hello"),
    )
}

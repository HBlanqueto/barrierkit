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
                Titlebar::new().title("Hello").toolbar(ToolbarItem::new(
                    ToolbarItemPlacement::PrimaryAction,
                    ToolGroup::new()
                        .child(IconButton::new(Symbol::Plus, "Add"))
                        .child(IconButton::new(Symbol::Ellipsis, "More")),
                )),
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
    Application::new("hello")
        .window("Hello", || {
            Backdrop::new(BackdropSource::Colors(BACKDROP_SCENES.dusk), Hello)
        })
        .options(|window| window.window_style(WindowStyle::HiddenTitleBar))
        .run()
}

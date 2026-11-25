use bk::popover::Popover;
use bk::prelude::*;
use bk::views::feedback::Sheet;
use torin::prelude::{Area, Gaps};

fn main() -> Result<(), RunError> {
    let sheet = std::env::args().nth(1).as_deref() == Some("sheet");
    run_window(
        move || {
            use_hook(move || {
                if !sheet {
                    Popover::new((200.0, 120.0), || {
                        rect()
                            .padding(20.0)
                            .child(Text::new("Popover content"))
                            .into()
                    })
                    .show(Area::new((190.0, 186.0).into(), (100.0, 24.0).into()));
                }
            });
            rect()
                .expanded()
                .child(
                    rect()
                        .padding(Gaps::new(186.0, 0.0, 0.0, 190.0))
                        .child(Button::new("Show")),
                )
                .maybe_child(
                    sheet
                        .then(|| Sheet::new(rect().width(Size::px(390.0)).height(Size::px(240.0)))),
                )
        },
        WindowOptions::new("popups", "com.example.popups").size((480, 400)),
    )
}

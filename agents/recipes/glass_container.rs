use bk::compact::IconButton;
use bk::glass_container::GlassContainer;
use bk::prelude::*;
use torin::prelude::Position;

const ROWS: [(u8, u8, u8); 6] = [
    (214, 82, 82),
    (237, 158, 51),
    (235, 204, 64),
    (102, 179, 89),
    (77, 128, 230),
    (153, 89, 191),
];

fn pair(top: f32, clear: bool) -> impl IntoElement {
    rect()
        .position(Position::new_absolute().top(top).left(40.0))
        .child(
            GlassContainer::new(20.0)
                .gap(6.0)
                .style(move |style| GlassContainerStyle { clear, ..style })
                .item("add", 22.0, IconButton::new(Symbol::Plus, "Add"))
                .item("remove", 22.0, IconButton::new(Symbol::Minus, "Remove")),
        )
}

fn main() -> Result<(), RunError> {
    run_window(
        || {
            rect()
                .expanded()
                .child(
                    rect()
                        .width(Size::fill())
                        .direction(Direction::Vertical)
                        .children((0..12).map(|index| {
                            rect()
                                .width(Size::fill())
                                .height(Size::px(44.0))
                                .background(ROWS[index % ROWS.len()])
                        })),
                )
                .child(pair(40.0, false))
                .child(pair(140.0, true))
        },
        WindowOptions::new("Glass container", "com.example.glass-container")
            .size((460, 300))
            .window_style(WindowStyle::Plain),
    )
}

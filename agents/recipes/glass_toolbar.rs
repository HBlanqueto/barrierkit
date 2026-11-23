use bk::compact::IconButton;
use bk::media::TopGlass;
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

fn main() -> Result<(), RunError> {
    run_window(
        || {
            rect()
                .expanded()
                .background(color(Palette::current().window_background))
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
                .child(
                    rect()
                        .position(Position::new_absolute().top(0.0).left(0.0))
                        .width(Size::fill())
                        .child(TopGlass::new())
                        .child(
                            Titlebar::new()
                                .title("Glass")
                                .style(|style| TitlebarStyle {
                                    background: StateValue::new(ShapeStyle::Clear),
                                    ..style
                                })
                                .right(
                                    ToolGroup::new()
                                        .child(IconButton::new(Symbol::Plus, "Add"))
                                        .child(IconButton::new(Symbol::Ellipsis, "More")),
                                ),
                        ),
                )
        },
        WindowOptions::new("Glass", "glass")
            .size((760, 480))
            .window_style(WindowStyle::Plain),
    )
}

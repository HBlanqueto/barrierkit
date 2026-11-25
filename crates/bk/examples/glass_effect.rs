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

fn badge(top: f32, left: f32, glass: Glass, shape: Shape, label: &'static str) -> impl IntoElement {
    rect()
        .position(Position::new_absolute().top(top).left(left))
        .child(Text::new(label).padding(14.0).glass_effect(glass, shape))
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
                .child(badge(30.0, 30.0, Glass::REGULAR, Shape::Capsule, "Capsule"))
                .child(badge(
                    100.0,
                    30.0,
                    Glass::CLEAR,
                    Shape::RoundedRectangle {
                        radius: 18.0,
                        style: CornerStyle::Continuous,
                    },
                    "Clear, rounded",
                ))
                .child(badge(170.0, 30.0, Glass::REGULAR, Shape::Circle, "OK"))
                .child(badge(30.0, 240.0, Glass::REGULAR, Shape::Capsule, "Fourth"))
        },
        WindowOptions::new("Glass effect", "com.example.glass-effect")
            .size((460, 300))
            .window_style(WindowStyle::Plain),
    )
}

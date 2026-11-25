use bk::media::{Material, TopGlass};
use bk::prelude::*;
use ctdesign::ACCENT_SWATCHES;
use torin::prelude::Position;

const ROWS: usize = 80;

fn main() -> Result<(), RunError> {
    run_window(
        || {
            rect()
                .expanded()
                .background(color(Palette::current().window_background))
                .child(
                    List::new("Rows", (1..=ROWS).map(|n| format!("Row {n}")).collect())
                        .row(|index| {
                            rect()
                                .width(Size::fill())
                                .height(Size::fill())
                                .background(color(ACCENT_SWATCHES[index % ACCENT_SWATCHES.len()].1))
                                .center()
                                .child(Text::new(format!("Row {}", index + 1)))
                        })
                        .style(|style| TableStyle {
                            row_height: 44.0,
                            ..style
                        }),
                )
                .child(
                    rect()
                        .position(Position::new_absolute().top(0.0).left(0.0))
                        .width(Size::fill())
                        .child(TopGlass::new())
                        .child(Titlebar::new().title("Glass window")),
                )
                .child(
                    rect()
                        .position(Position::new_absolute().bottom(16.0).right(16.0))
                        .width(Size::px(160.0))
                        .height(Size::px(44.0))
                        .center()
                        .child(Material::new(12.0))
                        .child(Text::new("Material")),
                )
        },
        WindowOptions::new("Glass window", "com.example.glass-window"),
    )
}

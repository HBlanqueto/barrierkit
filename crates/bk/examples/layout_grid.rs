use bk::prelude::*;

const FIELDS: [(&str, &str); 3] = [
    ("Name", "Ada Lovelace"),
    ("Email", "ada@example.org"),
    ("City", "London"),
];

fn main() -> Result<(), RunError> {
    Application::new("com.example.layout-grid")
        .window("Layout grid", || {
            let grid = FIELDS.iter().fold(
                Grid::new().alignment(ContentAlignment::LEADING),
                |grid, &(label, value)| {
                    grid.row(
                        GridRow::new()
                            .child(
                                Text::new(label)
                                    .grid_column_alignment(HorizontalAlignment::Trailing),
                            )
                            .child(Text::new(value)),
                    )
                },
            );
            VStack::new()
                .width(Size::px(280.0))
                .child(
                    Text::new("Contact")
                        .font(HEADLINE)
                        .frame()
                        .width(Size::fill())
                        .alignment(ContentAlignment::LEADING),
                )
                .child(grid)
                .child(
                    HStack::new()
                        .child(Button::new("Reset"))
                        .child(Spacer::new())
                        .child(Button::new("Save").style(ButtonStyle::bordered_prominent)),
                )
        })
        .run()
}

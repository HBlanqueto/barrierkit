use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("com.example.text-runs")
        .window("Rich text", || {
            VStack::new()
                .padding(SPACE_4)
                .spacing(SPACE_3)
                .child(
                    Text::new("Total: ")
                        + Text::new("42 items")
                            .bold(true)
                            .foreground_style(ShapeStyle::Accent),
                )
                .child(Text::markdown(
                    "**Bold**, *italic*, ~~struck~~, `code` and a [link](https://example.org).",
                ))
                .child(Text::new("Select this line and press Command+C").text_selection(true))
        })
        .options(|window| window.size((420, 220)).min_size(Some(WINDOW_MIN_SIZE)))
        .run()
}

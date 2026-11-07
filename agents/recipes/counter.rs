use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("counter")
        .window("Counter", || {
            let mut count = use_state(|| 0);
            VStack::new()
                .child(Text::new(count.read().to_string()).font(TITLE))
                .child(
                    Button::new("Add one")
                        .style(ButtonStyle::bordered_prominent)
                        .on_press(move |_| *count.write() += 1),
                )
        })
        .options(|window| window.size((420, 300)).min_size(Some(WINDOW_MIN_SIZE)))
        .run()
}

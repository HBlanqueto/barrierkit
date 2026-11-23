use bk::app::{Shortcut, ShortcutKey};
use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("shortcut")
        .window("Shortcut", || {
            let mut count = use_state(|| 0);
            VStack::new()
                .padding(SPACE_4)
                .spacing(SPACE_2)
                .child(Text::new(format!("Count {}", count.read())))
                .child(Text::new("Ctrl+K adds one, Escape resets"))
                .child(
                    Button::new("Add one")
                        .keyboard_shortcut(Shortcut::ctrl(ShortcutKey::Char('k')))
                        .on_press(move |_| *count.write() += 1),
                )
                .child(
                    Button::new("Reset")
                        .role(ButtonRole::Destructive)
                        .on_press(move |_| count.set(0)),
                )
                .keyboard_shortcut(Shortcut::CANCEL_ACTION)
        })
        .run()
}

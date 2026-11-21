use bk::app::{Shortcut, ShortcutKey};
use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("shortcut")
        .window("Shortcut", || {
            let mut count = use_state(|| 0);
            VStack::new()
                .child(Text::new(format!("Count {}", count.read())))
                .child(
                    Button::new("Add one")
                        .keyboard_shortcut(Shortcut::ctrl(ShortcutKey::Char('k')))
                        .on_press(move |_| *count.write() += 1),
                )
        })
        .run()
}

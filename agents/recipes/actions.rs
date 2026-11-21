use bk::app::{Action, Shortcut, ShortcutKey, use_menu_action};
use bk::main_menu::{MainMenu, Menu};
use bk::prelude::*;

fn add_title() -> &'static str {
    "Add one"
}

const ADD: Action = Action::new("add", add_title).shortcut(Shortcut::ctrl(ShortcutKey::Char('k')));

fn main() -> Result<(), RunError> {
    Application::new("actions")
        .window("Actions", || {
            let count = use_state(|| 0);
            use_menu_action(move |id| {
                if id == ADD.id {
                    let mut count = count;
                    *count.write() += 1;
                }
            });
            Text::new(format!("Count {}", count.read()))
        })
        .options(|window| {
            window.menu(MainMenu::new(vec![Menu::new(
                "Counter",
                vec![ADD.item(true)],
            )]))
        })
        .run()
}

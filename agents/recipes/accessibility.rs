use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("com.example.accessibility")
        .window("Accessibility", || {
            let mut tasks = use_state(|| 0);
            let note = use_state(String::new);
            VStack::new()
                .child(
                    Text::new("Tasks")
                        .font(TITLE)
                        .accessibility_add_traits(AccessibilityTraits::IS_HEADER),
                )
                .child(
                    Button::new("+")
                        .on_press(move |_| *tasks.write() += 1)
                        .accessibility_label("Add a task")
                        .accessibility_hint("Adds one to the list")
                        .accessibility_identifier("add-task"),
                )
                .child(
                    HStack::new()
                        .child(Text::new("Inbox"))
                        .child(Text::new(format!("{} open", tasks.read())))
                        .accessibility_element(AccessibilityChildBehavior::Combine),
                )
                .child(Divider::new().accessibility_hidden(true))
                .child(TextField::new(note, "Note").accessibility_label("Task note"))
        })
        .options(|window| window.size((420, 300)).min_size(Some(WINDOW_MIN_SIZE)))
        .run()
}

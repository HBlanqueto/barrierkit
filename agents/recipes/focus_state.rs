use bk::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum Field {
    Name,
    Email,
}

fn main() -> Result<(), RunError> {
    Application::new("com.example.focus")
        .window("Focus", || {
            let focus = use_focus_state::<Field>();
            let name = use_state(String::new);
            let email = use_state(String::new);
            VStack::new()
                .child(TextField::new(name, "Name").focused(focus, Field::Name))
                .child(TextField::new(email, "Email").focused(focus, Field::Email))
                .child(Button::new("Next").on_press(move |_| focus.set(Some(Field::Email))))
                .default_focus(focus, Field::Name)
        })
        .run()
}

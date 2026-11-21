use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("focus")
        .window("Focus", || {
            let query = use_state(String::new);
            let field = use_a11y();
            VStack::new()
                .child(SearchField::new(query, "Search").a11y_id(field))
                .child(Button::new("Focus the field").on_press(move |_| field.request_focus()))
        })
        .run()
}

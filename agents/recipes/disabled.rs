use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("com.example.disabled")
        .window("Disabled", || {
            let name = use_state(String::new);
            let mut saving = use_state(|| false);
            VStack::new()
                .child(
                    VStack::new()
                        .child(TextField::new(name, "Name"))
                        .child(Button::new("Save"))
                        .disabled(*saving.read()),
                )
                .child(
                    Switch::new("Saving")
                        .on(*saving.read())
                        .on_toggle(move |on| saving.set(on)),
                )
        })
        .run()
}

use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("com.example.modifiers")
        .window("Modifiers", || {
            HStack::new()
                .padding(SPACE_4)
                .child(Button::new("Save").margin(SPACE_2).help("Save the file"))
                .child(
                    Switch::new("Wi-Fi")
                        .margin(SPACE_2)
                        .help("Turn Wi-Fi on or off"),
                )
        })
        .run()
}

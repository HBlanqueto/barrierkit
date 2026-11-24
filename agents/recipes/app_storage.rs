use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("com.example.hello")
        .window("Hello", || {
            let mut bold = use_app_storage("bold", false);
            let mut italic = use_app_storage("italic", false);
            VStack::new()
                .padding(SPACE_4)
                .child(
                    Text::new("Hola mundo")
                        .maybe(*bold.read(), Text::bold)
                        .maybe(*italic.read(), Text::italic),
                )
                .child(
                    Checkbox::new("Bold")
                        .checked(*bold.read())
                        .on_toggle(move |on| bold.set(on)),
                )
                .child(
                    Checkbox::new("Italic")
                        .checked(*italic.read())
                        .on_toggle(move |on| italic.set(on)),
                )
        })
        .run()
}

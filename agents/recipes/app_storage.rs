use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("com.example.hello")
        .window("Hello", || {
            let bold = use_app_storage("bold", false);
            let italic = use_app_storage("italic", false);
            VStack::new()
                .padding(SPACE_4)
                .child(
                    Text::new("Hola mundo")
                        .bold(*bold.read())
                        .italic(*italic.read()),
                )
                .child(Checkbox::new("Bold", bold))
                .child(Checkbox::new("Italic", italic))
        })
        .run()
}

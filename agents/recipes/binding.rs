use bk::prelude::*;

#[derive(PartialEq)]
struct NameField {
    name: State<String>,
}

impl Component for NameField {
    fn render(&self) -> impl IntoElement {
        TextField::new(self.name, "Name")
    }
}

fn main() -> Result<(), RunError> {
    Application::new("com.example.binding")
        .window("Binding", || {
            let name = use_state(String::new);
            VStack::new()
                .child(NameField { name })
                .child(Text::new(format!("Hello, {}", name.read())))
        })
        .run()
}

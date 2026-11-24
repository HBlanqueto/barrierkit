use bk::prelude::*;

#[derive(PartialEq)]
struct Greeting {
    name: String,
}

impl Component for Greeting {
    fn render(&self) -> impl IntoElement {
        Text::new(format!("Hello, {}", self.name)).font(HEADLINE)
    }
}

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
    Application::new("com.example.components")
        .window("Components", || {
            let name = use_state(|| "Ana".to_string());
            VStack::new()
                .child(Greeting {
                    name: "Luis".to_string(),
                })
                .child(NameField { name })
                .child(Greeting {
                    name: name.read().clone(),
                })
        })
        .run()
}

use bk::prelude::*;

#[derive(PartialEq)]
struct Greeting {
    name: &'static str,
}

impl Component for Greeting {
    fn render(&self) -> impl IntoElement {
        Text::new(format!("Hello, {}", self.name)).font(HEADLINE)
    }
}

fn main() -> Result<(), RunError> {
    Application::new("greeting")
        .window("Greeting", || {
            VStack::new()
                .child(Greeting { name: "Ana" })
                .child(Greeting { name: "Luis" })
        })
        .run()
}

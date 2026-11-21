use bk::prelude::*;

#[derive(Clone, PartialEq)]
struct Account {
    name: &'static str,
}

#[derive(PartialEq)]
struct Profile;

impl Component for Profile {
    fn render(&self) -> impl IntoElement {
        let account = use_consume::<Account>();
        Text::new(format!("Signed in as {}", account.name))
    }
}

fn main() -> Result<(), RunError> {
    Application::new("environment")
        .window("Environment", || {
            use_provide_context(|| Account { name: "Ana" });
            VStack::new().child(Profile)
        })
        .run()
}

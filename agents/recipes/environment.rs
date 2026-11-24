use bk::prelude::*;

#[derive(Clone, PartialEq, Default)]
struct Account {
    name: &'static str,
}

#[derive(PartialEq)]
struct Profile;

impl Component for Profile {
    fn render(&self) -> impl IntoElement {
        let account: Account = use_environment();
        Text::new(format!("Signed in as {}", account.name))
    }
}

fn main() -> Result<(), RunError> {
    Application::new("com.example.environment")
        .window("Environment", || {
            let mut ana = use_state(|| true);
            let account = Account {
                name: if *ana.read() { "Ana" } else { "Luis" },
            };
            VStack::new()
                .child(Profile)
                .child(Button::new("Switch").on_press(move |_| {
                    let now = *ana.peek();
                    ana.set(!now);
                }))
                .environment(account)
        })
        .run()
}

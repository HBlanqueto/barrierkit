use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("conditions")
        .window("Conditions", || {
            let mut signed_in = use_state(|| false);
            let status = if *signed_in.read() {
                Text::new("Welcome back").into_element()
            } else {
                Button::new("Sign in")
                    .on_press(move |_| signed_in.set(true))
                    .into_element()
            };
            VStack::new().child(status).maybe_child(
                signed_in
                    .read()
                    .then(|| Button::new("Sign out").on_press(move |_| signed_in.set(false))),
            )
        })
        .run()
}

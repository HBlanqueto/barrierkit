use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("form")
        .window("Form", || {
            let name = use_state(String::new);
            let mut subscribed = use_state(|| false);
            let mut clicks = use_state(|| 0);
            VStack::new()
                .cross_align(Alignment::Start)
                .spacing(8.0)
                .padding(SPACE_4)
                .child(TextField::new(name, "Name"))
                .child(
                    Checkbox::new("Subscribe")
                        .checked(*subscribed.read())
                        .on_toggle(move |checked| subscribed.set(checked)),
                )
                .child(if *subscribed.read() {
                    Text::new(format!("Thanks, {}", name.read())).into_element()
                } else {
                    Text::new("Not subscribed").into_element()
                })
                .maybe_child(
                    (*clicks.read() > 2 && !*subscribed.read()).then(|| Text::new("Still here?")),
                )
                .child(Button::new("Click").on_press(move |_| *clicks.write() += 1))
                .child(
                    Text::new(if *subscribed.read() { "yes" } else { "no" })
                        .padding(12.0)
                        .width(Size::px(200.0))
                        .height(Size::px(40.0)),
                )
        })
        .run()
}

use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("button-label")
        .window("Button label", || {
            HStack::new()
                .padding(SPACE_4)
                .child(Button::new("Add").label(|| Label::new("Add", Symbol::Plus)))
                .child(
                    Button::new("Add")
                        .style(ButtonStyle::bordered_prominent)
                        .label(|| Label::new("Add", Symbol::Plus)),
                )
        })
        .run()
}

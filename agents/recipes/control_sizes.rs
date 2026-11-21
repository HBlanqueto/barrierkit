use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("sizes")
        .window("Sizes", || {
            [
                ControlSize::Mini,
                ControlSize::Small,
                ControlSize::Regular,
                ControlSize::Large,
                ControlSize::ExtraLarge,
            ]
            .into_iter()
            .fold(VStack::new(), |column, size| {
                column.child(
                    HStack::new()
                        .child(Button::new("Save").size(size))
                        .child(Checkbox::new("Remember").size(size))
                        .child(Switch::new("Wi-Fi").size(size)),
                )
            })
        })
        .run()
}

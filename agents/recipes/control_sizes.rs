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
                        .child(Button::new("Save"))
                        .child(Checkbox::new("Remember"))
                        .child(Switch::new("Wi-Fi"))
                        .control_size(size),
                )
            })
        })
        .run()
}

use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("com.example.sizes")
        .window("Sizes", || {
            let remember = use_state(|| false);
            let wifi = use_state(|| true);
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
                        .child(Checkbox::new("Remember", remember))
                        .child(Switch::new("Wi-Fi", wifi))
                        .control_size(size),
                )
            })
        })
        .run()
}

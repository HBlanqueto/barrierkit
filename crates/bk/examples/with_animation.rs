use bk::prelude::*;
use ctdesign::SPRING_SHEET_MS;

const DETAILS_HEIGHT: f32 = 60.0;

fn main() -> Result<(), RunError> {
    Application::new("com.example.details")
        .window("Details", || {
            let mut expanded = use_state(|| false);
            let progress = use_animated(f32::from(u8::from(*expanded.read())), Animation::None);
            VStack::new()
                .child(
                    Button::new(if *expanded.read() { "Hide" } else { "Show" }).on_press(
                        move |_| {
                            with_animation(
                                Animation::Spring {
                                    duration_ms: SPRING_SHEET_MS,
                                    bounce: 0.0,
                                },
                                || expanded.toggle(),
                            )
                        },
                    ),
                )
                .child(
                    rect()
                        .width(Size::fill())
                        .height(Size::px(progress * DETAILS_HEIGHT))
                        .overflow(Overflow::Clip)
                        .child(Text::new("Details")),
                )
        })
        .run()
}

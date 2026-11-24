use bk::prelude::*;
use ctdesign::SPRING_SHEET_MS;

fn main() -> Result<(), RunError> {
    Application::new("com.example.transitions")
        .window("Transitions", || {
            let mut shown = use_state(|| false);
            let mut count = use_state(|| 0u32);
            let spring = Animation::Spring {
                duration_ms: SPRING_SHEET_MS,
                bounce: 0.0,
            };
            VStack::new()
                .spacing(12.0)
                .padding(20.0)
                .child(Button::new("Toggle").on_press(move |_| {
                    with_animation(spring, || shown.toggle());
                }))
                .child(
                    Presence::new((*shown.read()).then(|| {
                        rect()
                            .padding(12.0)
                            .corner_radius(10.0)
                            .background_style(ShapeStyle::Accent)
                            .child(Text::new("Inserted from the left, removed by fading"))
                    }))
                    .transition(Transition::asymmetric(
                        Transition::move_edge(Edge::Leading).combined(Transition::OPACITY),
                        Transition::OPACITY.combined(Transition::scale(0.8)),
                    )),
                )
                .child(Button::new("Count").on_press(move |_| {
                    with_animation(spring, || *count.write() += 1);
                }))
                .child(
                    Text::new(format!("Pressed {}", count.read()))
                        .content_transition(ContentTransition::Opacity),
                )
        })
        .run()
}

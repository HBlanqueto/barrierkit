use bk::level::LevelSlider;
use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("com.example.mixer")
        .theme(
            Theme::tengen()
                .style(|s: SliderStyle| SliderStyle {
                    track_shape: Shape::Rectangle,
                    knob_shape: Shape::Rectangle,
                    ..s
                })
                .style(ButtonStyle::glass),
        )
        .window("Mixer", || {
            let volume = use_state(|| 0.4);
            let bass = use_state(|| 0.5);
            let treble = use_state(|| 0.75);
            let output = use_state(|| 0.6);
            VStack::new()
                .child(Slider::new("Volume", volume).style(|s| {
                    SliderStyle {
                        fill: StateValue::new(ShapeStyle::Role(Role::Destructive))
                            .pressed(ShapeStyle::Role(Role::SecondaryLabel)),
                        ..s
                    }
                }))
                .child(
                    HStack::new()
                        .child(Slider::new("Bass", bass))
                        .child(Slider::new("Treble", treble))
                        .control_style(|s: SliderStyle| SliderStyle {
                            axis: Axis::Vertical,
                            ..s
                        }),
                )
                .child(LevelSlider::new("Output", output).style(|s| s.fat(Symbol::SpeakerWave)))
                .child(Button::new("Reset"))
        })
        .run()
}

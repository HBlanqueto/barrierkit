use bk::level::LevelSlider;
use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("mixer")
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
            VStack::new()
                .child(Slider::stepped("Volume", 10).value(4).style(|s| {
                    SliderStyle {
                        fill: StateValue::new(ShapeStyle::Role(Role::Destructive))
                            .pressed(ShapeStyle::Role(Role::SecondaryLabel)),
                        ..s
                    }
                }))
                .child(
                    HStack::new()
                        .child(Slider::stepped("Bass", 5).value(2))
                        .child(Slider::stepped("Treble", 5).value(3))
                        .control_style(|s: SliderStyle| SliderStyle {
                            axis: Axis::Vertical,
                            ..s
                        }),
                )
                .child(
                    LevelSlider::new("Output")
                        .value(0.6)
                        .style(|s| s.fat(Symbol::SpeakerWave)),
                )
                .child(Button::new("Reset"))
        })
        .run()
}

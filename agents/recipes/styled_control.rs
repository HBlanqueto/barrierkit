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
                .child(Slider::new("Volume").value(0.4).style(|s| {
                    SliderStyle {
                        fill: StateValue::new(ShapeStyle::Role(Role::Destructive))
                            .pressed(ShapeStyle::Role(Role::SecondaryLabel)),
                        ..s
                    }
                }))
                .child(
                    HStack::new()
                        .child(Slider::new("Bass").value(0.5))
                        .child(Slider::new("Treble").value(0.75))
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

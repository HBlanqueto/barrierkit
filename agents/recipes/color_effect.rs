use bk::prelude::*;

fn darker_when_pressed(color: Rgba, state: ControlState) -> Rgba {
    if state.pressed {
        color.darken(0.2)
    } else {
        color
    }
}

fn main() -> Result<(), RunError> {
    Application::new("mixer")
        .theme(Theme::tengen().style(|s: SliderStyle| SliderStyle {
            effect: darker_when_pressed,
            ..s
        }))
        .window("Mixer", || Slider::new("Volume").value(0.4))
        .run()
}

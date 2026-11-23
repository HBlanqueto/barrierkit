use bk::animation::{use_animated, with_animation};
use bk::group::{CompositingGroup, GroupEffect, ShaderEffect};
use bk::prelude::*;
use ctdesign::{ACCENT_SWATCHES, SPRING_SHEET_MS};
use torin::prelude::Position;

const PULSE: &str = "uniform shader content; uniform float time; half4 main(float2 p) { return content.eval(p) * half(0.75 + 0.25 * sin(time * 3.0)); }";

fn disc(left: f32, index: usize) -> impl IntoElement {
    rect()
        .position(Position::new_absolute().left(left).top(0.0))
        .width(Size::px(80.0))
        .height(Size::px(80.0))
        .background(color(ACCENT_SWATCHES[index].1))
}

fn overlapping() -> impl IntoElement {
    rect()
        .width(Size::px(140.0))
        .height(Size::px(80.0))
        .child(disc(0.0, 0))
        .child(disc(60.0, 1))
}

fn main() -> Result<(), RunError> {
    Application::new("compositing_group")
        .window("Compositing group", || {
            let mut scrolled = use_state(|| false);
            let progress = use_animated(f32::from(u8::from(*scrolled.read())), Animation::None);
            HStack::new()
                .child(CompositingGroup::new(GroupEffect::Opacity(0.5)).child(overlapping()))
                .child(
                    CompositingGroup::new(GroupEffect::Shader(ShaderEffect::new(PULSE, vec![])))
                        .child(overlapping()),
                )
                .child(
                    CompositingGroup::new(GroupEffect::ScrollEdge {
                        depth: 40.0,
                        progress,
                        dark: false,
                    })
                    .child(overlapping()),
                )
                .child(Button::new("Scroll edge").on_press(move |_| {
                    with_animation(
                        Animation::Spring {
                            duration_ms: SPRING_SHEET_MS,
                            bounce: 0.0,
                        },
                        || scrolled.toggle(),
                    )
                }))
        })
        .run()
}

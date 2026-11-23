use bk::group::{CompositingGroup, GroupEffect, ShaderEffect};
use bk::prelude::*;
use ctdesign::ACCENT_SWATCHES;
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
            HStack::new()
                .child(CompositingGroup::new(GroupEffect::Opacity(0.5)).child(overlapping()))
                .child(
                    CompositingGroup::new(GroupEffect::Shader(ShaderEffect::new(PULSE, vec![])))
                        .child(overlapping()),
                )
        })
        .run()
}

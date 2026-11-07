use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("profile")
        .window("Profile", || {
            VStack::new()
                .cross_align(Alignment::Start)
                .spacing(SPACE_1)
                .child(Text::new("Ana Díaz").font(HEADLINE))
                .child(
                    HStack::new()
                        .child(Text::new("Madrid").font(CAPTION))
                        .child(Text::new("Designer").font(CAPTION)),
                )
        })
        .run()
}

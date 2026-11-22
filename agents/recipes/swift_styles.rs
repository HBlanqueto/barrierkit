use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("styles")
        .window("Styles", || {
            let mut on = use_state(|| true);
            let mut busy = use_state(|| false);
            HStack::new()
                .padding((0.0, 20.0))
                .width(Size::fill())
                .child(Text::new("Large").font(LARGE_TITLE))
                .child(Text::new("Small").font(CAPTION))
                .child(
                    Button::new("Plain")
                        .style(ButtonStyle::plain)
                        .on_press(move |_| busy.toggle()),
                )
                .child(
                    Button::new("Borderless")
                        .style(ButtonStyle::borderless)
                        .disabled(*busy.read())
                        .on_press(move |_| busy.toggle()),
                )
                .child(
                    Switch::new("Switch")
                        .on(*on.read())
                        .on_toggle(move |checked| on.set(checked)),
                )
        })
        .run()
}

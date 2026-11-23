use bk::gesture::{Attachable, DragGesture, LongPressGesture, TapGesture, use_gesture_state};
use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("gestures")
        .window("Gestures", || {
            let mut taps = use_state(|| 0);
            let mut held = use_state(|| false);
            let offset = use_gesture_state(0.0f32);
            VStack::new()
                .padding(SPACE_4)
                .child(Text::new(format!("{} taps", taps.read())))
                .child(
                    rect().offset_x(offset.get()).child(
                        rect()
                            .width(Size::px(120.0))
                            .height(Size::px(120.0))
                            .background(Color::from_rgb(0, 122, 255))
                            .gesture(
                                LongPressGesture::new()
                                    .on_ended(move |_| held.set(true))
                                    .exclusively(
                                        TapGesture::new().on_ended(move |_| *taps.write() += 1),
                                    ),
                            )
                            .simultaneous_gesture(DragGesture::new().updating(
                                offset,
                                |value, state| {
                                    *state = value.translation.0;
                                },
                            )),
                    ),
                )
                .child(Text::new(if *held.read() { "held" } else { "not held" }))
        })
        .options(|window| window.size((420, 300)).min_size(Some(WINDOW_MIN_SIZE)))
        .run()
}

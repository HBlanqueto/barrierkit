use bk::drag::{DragPayload, use_drag_source};
use bk::pasteboard::Pasteboard;
use bk::prelude::*;

const PICTURE: &[u8] = include_bytes!("../../docs/barrierkit/assets/states/state-normal.png");

fn rich() -> DragPayload {
    DragPayload::Html {
        text: "Hello, world".into(),
        html: "<b>Hello</b>, <i>world</i>".into(),
    }
}

fn describe(payload: Option<DragPayload>) -> String {
    match payload {
        Some(DragPayload::Files(paths)) => format!("{} file(s)", paths.len()),
        Some(DragPayload::Image(bytes)) => format!("An image of {} bytes", bytes.len()),
        Some(DragPayload::Html { html, .. }) => format!("Rich text: {html}"),
        Some(DragPayload::Text(text)) => format!("Text: {text}"),
        None => "The pasteboard is empty".to_owned(),
    }
}

fn main() -> Result<(), RunError> {
    Application::new("com.example.pasteboard")
        .window("Pasteboard", || {
            let mut found = use_state(|| "Nothing pasted yet".to_owned());
            let drag = use_drag_source();
            VStack::new()
                .padding(SPACE_4)
                .spacing(SPACE_3)
                .child(
                    HStack::new()
                        .child(Button::new("Copy text").on_press(|_| {
                            Pasteboard::general().copy(&"Hello, world".to_owned());
                        }))
                        .child(Button::new("Copy rich text").on_press(|_| {
                            Pasteboard::general().copy(&rich());
                        }))
                        .child(Button::new("Copy image").on_press(|_| {
                            Pasteboard::general().copy(&DragPayload::Image(PICTURE.to_vec()));
                        })),
                )
                .child(
                    Button::new("Paste")
                        .on_press(move |_| found.set(describe(Pasteboard::general().paste()))),
                )
                .child(Text::new(found.read().clone()))
                .child(
                    drag.attach(
                        rect()
                            .padding(SPACE_3)
                            .child(Text::new("Drag the rich text")),
                        rich,
                    ),
                )
        })
        .options(|window| window.size((460, 260)).min_size(Some(WINDOW_MIN_SIZE)))
        .run()
}

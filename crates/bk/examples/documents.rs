use std::fs;

use bk::prelude::*;

#[derive(PartialEq)]
struct Notes;

impl Component for Notes {
    fn render(&self) -> impl IntoElement {
        let text = use_state(String::new);
        let saved = use_state(String::new);
        let opened = use_state(|| String::from("Nothing opened"));
        let document = use_document(move |path| {
            fs::write(path, text.peek().as_bytes())?;
            let mut saved = saved;
            saved.set(text.peek().clone());
            Ok(())
        });
        document.set_edited(*text.read() != *saved.read());
        let to_save = document.clone();
        let to_open = document.clone();
        VStack::new()
            .child(Text::new(opened.read().clone()))
            .child(TextField::new(text, "Note"))
            .child(Button::new("Save").on_press(move |_| {
                let document = to_save.clone();
                spawn(async move {
                    document.save().await;
                });
            }))
            .on_open_url(move |url: OpenedUrl| {
                if let OpenedUrl::File(path) = url {
                    let contents = fs::read_to_string(&path).unwrap_or_default();
                    let (mut text, mut saved, mut opened) = (text, saved, opened);
                    text.set(contents.clone());
                    saved.set(contents);
                    opened.set(path.display().to_string());
                    to_open.set_url(Some(path));
                }
            })
    }
}

fn main() -> Result<(), RunError> {
    Application::new("com.example.documents")
        .window("Notes", || Notes)
        .options(|window| window.size((420, 300)))
        .run()
}

use std::path::PathBuf;

use bk::prelude::*;

#[derive(PartialEq)]
struct Pictures;

impl Component for Pictures {
    fn render(&self) -> impl IntoElement {
        let mut importing = use_state(|| false);
        let mut exporting = use_state(|| false);
        let mut chosen = use_state(|| String::from("Nothing chosen"));
        VStack::new()
            .child(Button::new("Import…").on_press(move |_| importing.set(true)))
            .child(Button::new("Export…").on_press(move |_| exporting.set(true)))
            .child(Text::new(chosen.read().clone()))
            .file_importer(
                importing,
                vec![ContentType::extension("png"), ContentType::extension("jpg")],
                true,
                move |paths: Vec<PathBuf>| chosen.set(format!("{} picture(s)", paths.len())),
            )
            .file_exporter(
                exporting,
                "Picture.png",
                vec![ContentType::extension("png")],
                move |path: PathBuf| chosen.set(path.display().to_string()),
            )
    }
}

fn main() -> Result<(), RunError> {
    Application::new("com.example.file-dialogs")
        .window("Pictures", || {
            Pictures.file_dialog_default_directory(std::env::temp_dir())
        })
        .options(|window| window.size((420, 300)))
        .run()
}

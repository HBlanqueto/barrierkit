use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("com.example.scenes")
        .window("Notes", || {
            let open = use_open_window();
            let open_note = open.clone();
            VStack::new()
                .child(Button::new("Open the inspector").on_press(move |_| open.open("inspector")))
                .child(Button::new("Open note 7").on_press(move |_| open_note.open_value(7_u32)))
        })
        .scene(Window::new("inspector", "Inspector", || {
            Text::new("Inspector")
        }))
        .scene(WindowGroup::for_value("note", "Note", |id: u32| {
            Text::new(format!("Note {id}"))
        }))
        .scene(Settings::new(|| Text::new("Settings")))
        .commands(Commands::new())
        .run()
}

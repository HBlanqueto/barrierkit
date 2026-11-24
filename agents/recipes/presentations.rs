use bk::prelude::*;
use bk::views::feedback::Alert;

fn main() -> Result<(), RunError> {
    Application::new("com.example.presentations")
        .window("Presentations", || {
            let mut deleting = use_state(|| false);
            let mut showing_info = use_state(|| false);
            let mut deleted = use_state(|| false);
            VStack::new()
                .spacing(12.0)
                .padding(20.0)
                .child(Button::new("Delete…").on_press(move |_| deleting.set(true)))
                .child(
                    rect()
                        .child(Button::new("Info").on_press(move |_| showing_info.set(true)))
                        .popover(showing_info, (200.0, 80.0), || {
                            rect()
                                .padding(20.0)
                                .child(Text::new("Updated just now"))
                                .into()
                        }),
                )
                .child(Text::new(if *deleted.read() { "Deleted" } else { "Kept" }))
                .alert(deleting, move || {
                    Alert::new("Delete this file?", "You can't undo this.")
                        .destructive()
                        .confirm("Delete", move |_| deleted.set(true))
                })
        })
        .run()
}

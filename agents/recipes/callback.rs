use bk::prelude::*;

#[derive(PartialEq)]
struct SaveBar {
    on_save: Callback<(), ()>,
}

impl SaveBar {
    fn new(on_save: impl Into<Callback<(), ()>>) -> Self {
        Self {
            on_save: on_save.into(),
        }
    }
}

impl Component for SaveBar {
    fn render(&self) -> impl IntoElement {
        let on_save = self.on_save.clone();
        Button::new("Save").on_press(move |_| on_save.call(()))
    }
}

fn main() -> Result<(), RunError> {
    Application::new("callback")
        .window("Callback", || {
            let mut saves = use_state(|| 0);
            VStack::new()
                .child(SaveBar::new(move |_| *saves.write() += 1))
                .child(Text::new(format!("Saved {} times", saves.read())))
        })
        .run()
}

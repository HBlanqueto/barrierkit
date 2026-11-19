use bk::prelude::*;
use bk::views::form::{Form, Row, Section};

fn main() -> Result<(), RunError> {
    Application::new("settings-sample")
        .window("Settings", || {
            let mut sounds = use_state(|| true);
            Form::new().section(Section::titled("General").row(Row::toggle(
                "Sounds",
                *sounds.read(),
                move |on| sounds.set(on),
            )))
        })
        .run()
}

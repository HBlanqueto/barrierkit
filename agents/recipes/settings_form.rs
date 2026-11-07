use bk::prelude::*;
use bk::views::form::{Form, Row, Section};

fn main() -> Result<(), RunError> {
    Application::new("settings-sample")
        .window("Settings", || {
            let mut sounds = use_state(|| true);
            Form::new().section(
                Section::titled("General").row(
                    Row::new("Sounds").control(
                        Switch::new("Sounds")
                            .on(*sounds.read())
                            .on_toggle(move |on| sounds.set(on)),
                    ),
                ),
            )
        })
        .run()
}

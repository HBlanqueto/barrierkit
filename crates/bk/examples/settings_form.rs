use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("com.example.settings-sample")
        .window("Settings", || {
            let sounds = use_state(|| true);
            Form::new().section(Section::titled("General").row(Row::toggle("Sounds", sounds)))
        })
        .run()
}

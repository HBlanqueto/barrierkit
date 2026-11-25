use bk::prelude::*;

fn settings(notifications: State<bool>) -> Form {
    Form::new()
        .section(
            Section::titled("Account")
                .row(Row::value("Name", "Ada Lovelace"))
                .row(Row::toggle("Notifications", notifications))
                .row(Row::navigation("Language", |()| {}).detail("Arabic")),
        )
        .section(Section::new().footer("Changes apply to every window."))
}

fn panel(title: &'static str, notifications: State<bool>) -> impl IntoElement {
    VStack::new()
        .width(Size::px(300.0))
        .cross_align(Alignment::Start)
        .child(Text::new(title).font(HEADLINE))
        .child(
            HStack::new()
                .child(Image::system(Symbol::Bell))
                .child(Text::new("Reminders"))
                .child(Spacer::new())
                .child(Button::new("Add")),
        )
        .child(settings(notifications))
}

fn main() -> Result<(), RunError> {
    Application::new("com.example.right-to-left")
        .window("Right to left", || {
            let notifications = use_state(|| true);
            HStack::new()
                .padding(SPACE_4)
                .spacing(SPACE_4)
                .child(panel("Left to right", notifications))
                .child(
                    panel("Right to left", notifications)
                        .layout_direction(LayoutDirection::RightToLeft),
                )
        })
        .run()
}

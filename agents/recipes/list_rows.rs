use bk::adornments::{Avatar, Badge};
use bk::prelude::*;
use bk::views::list::List;
use bk::views::table::{ListRowModifiers, Table, Visibility};
use ctdesign::SETTINGS;

const MAIL: [(&str, &str, &str); 3] = [
    ("Ada Lovelace", "Notes on the engine", "3"),
    ("Grace Hopper", "Compiler draft", "1"),
    ("Alan Turing", "Tape lengths", "7"),
];
const SERVICES: [&str; 3] = ["Backups", "Sync", "Sharing"];

fn main() -> Result<(), RunError> {
    Application::new("com.example.list-rows")
        .window("List rows", || {
            let mut selected = use_state(Vec::<usize>::new);
            let mut enabled = use_state(|| vec![true, false, true]);
            let row_height = SETTINGS.account_avatar as f32 + 2.0 * SPACE_2;
            VStack::new()
                .padding(SPACE_4)
                .spacing(SPACE_4)
                .child(
                    List::new(
                        "Mail",
                        MAIL.map(|(_, subject, _)| subject.to_string()).to_vec(),
                    )
                    .row(|index| {
                        let (from, subject, unread) = MAIL[index];
                        HStack::new()
                            .width(Size::fill())
                            .main_align(Alignment::SpaceBetween)
                            .child(
                                HStack::new()
                                    .child(Avatar::named(from))
                                    .child(Text::new(subject)),
                            )
                            .child(Badge::new(unread))
                    })
                    .list_row_background(|row| match MAIL[row].2 {
                        "7" => ShapeStyle::Role(Role::SystemFill),
                        _ => ShapeStyle::Clear,
                    })
                    .list_row_separator(|row| match row {
                        0 => Visibility::Hidden,
                        _ => Visibility::Automatic,
                    })
                    .list_row_insets(|_| SPACE_4)
                    .selected(selected.read().clone())
                    .on_select(move |rows: Vec<usize>| selected.set(rows))
                    .style(move |style| TableStyle {
                        row_height,
                        ..style
                    }),
                )
                .child(
                    Table::new("Services", ["Service", "Enabled"])
                        .rows(SERVICES.map(|name| vec![name.to_string()]).to_vec())
                        .cell(1, move |row| {
                            rect()
                                .on_press(|event: Event<PressEventData>| event.stop_propagation())
                                .child(
                                    Switch::new(SERVICES[row])
                                        .on(enabled.read()[row])
                                        .on_toggle(move |on: bool| enabled.write()[row] = on),
                                )
                        }),
                )
        })
        .run()
}

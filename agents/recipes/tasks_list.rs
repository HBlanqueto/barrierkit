use bk::compact::IconButton;
use bk::prelude::*;
use bk::views::list::List;

const TASKS: [(&str, Symbol); 4] = [
    ("Buy milk", Symbol::House),
    ("Call Ada", Symbol::Phone),
    ("Write report", Symbol::DocText),
    ("Book flights", Symbol::PaperPlane),
];

fn main() -> Result<(), RunError> {
    run_window(
        || {
            rect()
                .expanded()
                .direction(Direction::Vertical)
                .child(
                    Titlebar::new().title("Tasks").right(
                        ToolGroup::new()
                            .child(IconButton::new(Symbol::Plus, "Add"))
                            .child(IconButton::new(Symbol::Ellipsis, "More")),
                    ),
                )
                .child(
                    List::new("Tasks", TASKS.map(|(title, _)| title.to_string()).to_vec())
                        .row(|index| Label::new(TASKS[index].0, TASKS[index].1)),
                )
        },
        WindowOptions::new("Tasks", "tasks")
            .size((760, 480))
            .window_style(WindowStyle::Plain),
    )
}

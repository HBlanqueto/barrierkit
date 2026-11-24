use bk::prelude::*;
use bk::views::list::List;

struct Fruit {
    id: i64,
    name: &'static str,
}

const FRUITS: [Fruit; 2] = [
    Fruit {
        id: 1,
        name: "Apple",
    },
    Fruit {
        id: 2,
        name: "Banana",
    },
];

fn main() -> Result<(), RunError> {
    Application::new("com.example.lists")
        .window("Lists", || {
            VStack::new()
                .children(FRUITS.into_iter().map(|fruit| {
                    HStack::new()
                        .spacing(4.0)
                        .child(Text::new(fruit.name))
                        .child(Spacer::new())
                        .child(Text::new(fruit.id.to_string()))
                        .key(fruit.id)
                }))
                .child(Divider::new())
                .children((0..3).map(|index| Text::new(format!("Row {index}")).key(index)))
                .child(
                    List::new(
                        "List",
                        FRUITS.iter().map(|fruit| fruit.name.to_string()).collect(),
                    )
                    .row(|index| {
                        let fruit = &FRUITS[index];
                        Text::new(fruit.name)
                    }),
                )
        })
        .run()
}

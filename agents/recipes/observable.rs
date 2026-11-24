use bk::prelude::*;

#[derive(Clone, Copy, PartialEq)]
struct Counter {
    count: State<i32>,
    step: State<i32>,
}

impl Counter {
    fn new() -> Self {
        Self {
            count: use_state(|| 0),
            step: use_state(|| 1),
        }
    }
}

#[derive(PartialEq)]
struct CountLabel;

impl Component for CountLabel {
    fn render(&self) -> impl IntoElement {
        let counter = use_consume::<Counter>();
        Text::new(format!("Count {}", counter.count.read()))
    }
}

#[derive(PartialEq)]
struct AddButton;

impl Component for AddButton {
    fn render(&self) -> impl IntoElement {
        let mut counter = use_consume::<Counter>();
        Button::new("Add").on_press(move |_| {
            let step = *counter.step.read();
            *counter.count.write() += step;
        })
    }
}

fn main() -> Result<(), RunError> {
    Application::new("com.example.observable")
        .window("Observable", || {
            let counter = Counter::new();
            use_provide_context(|| counter);
            VStack::new().child(CountLabel).child(AddButton)
        })
        .run()
}

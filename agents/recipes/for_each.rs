use bk::prelude::*;

const FRUITS: [(u32, &str); 3] = [(1, "Apple"), (2, "Banana"), (3, "Cherry")];

fn main() -> Result<(), RunError> {
    Application::new("fruits")
        .window("Fruits", || {
            VStack::new().children(FRUITS.iter().map(|&(id, name)| Text::new(name).key(id)))
        })
        .run()
}

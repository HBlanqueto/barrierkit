use bk::prelude::*;
use bk::views::navigation_stack::{NavigationPath, NavigationStack, use_navigation_path};

#[derive(Clone, PartialEq)]
enum Route {
    Detail(u32),
}

#[derive(PartialEq)]
struct Home;

impl Component for Home {
    fn render(&self) -> impl IntoElement {
        let mut path = use_navigation_path::<Route>();
        VStack::new()
            .child(Text::new("Home"))
            .child(Button::new("Open 1").on_press(move |_| path.write().push(Route::Detail(1))))
    }
}

#[derive(PartialEq)]
struct Detail(u32);

impl Component for Detail {
    fn render(&self) -> impl IntoElement {
        let mut path = use_navigation_path::<Route>();
        VStack::new()
            .child(Text::new(format!("Detail {}", self.0)))
            .child(Button::new("Open next").on_press({
                let next = self.0 + 1;
                move |_| path.write().push(Route::Detail(next))
            }))
            .child(Button::new("Back").on_press(move |_| {
                path.write().pop();
            }))
    }
}

fn main() -> Result<(), RunError> {
    Application::new("navigation_stack")
        .window("Navigation stack", || {
            let path = use_state(NavigationPath::<Route>::new);
            NavigationStack::new(path, Home, |route| match route {
                Route::Detail(n) => Detail(*n),
            })
        })
        .run()
}

use freya_components::input::Input;
use freya_core::integration::{App, AppComponent};
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Direction, Size2D};

struct FocusApp {
    value_a: State<String>,
    value_b: State<String>,
}

impl App for FocusApp {
    fn render(&self) -> impl IntoElement {
        rect()
            .expanded()
            .direction(Direction::Vertical)
            .child(Input::new(self.value_a).auto_focus(true))
            .child(Input::new(self.value_b))
    }
}

#[test]
fn auto_focused_input_receives_keys_and_tab_moves_focus() {
    let value_a = State::create_global(String::new());
    let value_b = State::create_global(String::new());

    let app = AppComponent::new(FocusApp { value_a, value_b });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(300.0, 200.0), |_| {}, 1.0);

    test.sync_and_update();

    test.write_text("a");
    assert_eq!(value_a.peek().as_str(), "a");
    assert!(value_b.peek().is_empty());

    test.press_key(Key::Named(NamedKey::Tab));
    test.sync_and_update();

    test.write_text("b");
    assert_eq!(value_a.peek().as_str(), "a");
    assert_eq!(value_b.peek().as_str(), "b");
}

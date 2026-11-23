use bk::prelude::*;
use freya_core::integration::AppComponent;
use freya_testing::TestingRunner;
use torin::prelude::Size2D;

struct Case;

impl App for Case {
    fn render(&self) -> impl IntoElement {
        let note = use_state(String::new);
        VStack::new()
            .child(
                Button::new("+")
                    .accessibility_label("Add a task")
                    .accessibility_hint("Adds one to the list")
                    .accessibility_identifier("add"),
            )
            .child(
                Text::new("Tasks")
                    .accessibility_add_traits(AccessibilityTraits::IS_HEADER)
                    .accessibility_identifier("title"),
            )
            .child(
                TextField::new(note, "Note")
                    .accessibility_label("Task note")
                    .accessibility_identifier("note"),
            )
    }
}

fn node_of(id: &str) -> accesskit::Node {
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Case),
        Size2D::new(400.0, 300.0),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    test.sync_and_update();
    test.find(|_, element| {
        let builder = element.accessibility().builder.clone();
        (builder.author_id() == Some(id)).then_some(builder)
    })
    .unwrap_or_else(|| panic!("no node with the identifier {id}"))
}

#[test]
fn a_control_takes_the_label_hint_and_identifier_of_its_modifiers() {
    let button = node_of("add");
    assert_eq!(button.role(), accesskit::Role::Button);
    assert_eq!(button.label(), Some("Add a task"));
    assert_eq!(button.description(), Some("Adds one to the list"));
}

#[test]
fn a_text_takes_the_role_of_its_trait() {
    assert_eq!(node_of("title").role(), accesskit::Role::Heading);
}

#[test]
fn a_text_field_carries_its_label_for_the_input_node() {
    assert_eq!(node_of("note").label(), Some("Task note"));
}

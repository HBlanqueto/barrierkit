use bk::prelude::*;
use ctdesign::{Rgba, SANS_FAMILIES};
use freya_core::integration::AppComponent;
use freya_testing::TestingRunner;
use torin::prelude::{Alignment, Size2D};

struct Case(Box<dyn Fn() -> Element>);

impl App for Case {
    fn render(&self) -> impl IntoElement {
        rect()
            .expanded()
            .background((255, 255, 255))
            .main_align(Alignment::Start)
            .cross_align(Alignment::Start)
            .child((self.0)())
    }
}

fn runner(build: impl Fn() -> Element + 'static) -> TestingRunner {
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Case(Box::new(build))),
        Size2D::new(400.0, 200.0),
        |_| {},
        1.0,
    );
    test.set_default_fonts(&SANS_FAMILIES.map(std::borrow::Cow::Borrowed));
    test.sync_and_update();
    test.sync_and_update();
    test
}

fn measured(text: impl Fn() -> Text + 'static) -> (f32, f32) {
    let test = runner(move || {
        text()
            .a11y_builder(|node| node.set_author_id("text"))
            .into_element()
    });
    test.find(|node, element| {
        (element.accessibility().builder.author_id() == Some("text")).then(|| {
            let area = node.layout().area;
            (area.width(), area.height())
        })
    })
    .expect("the text is probed")
}

fn has_pixel(test: &mut TestingRunner, wanted: impl Fn([u8; 4]) -> bool) -> bool {
    let (stride, _, image) = crate::image::decode(test.render().as_bytes());
    image.chunks(4).any(|px| wanted(px.try_into().unwrap())) && stride > 0
}

fn red([r, g, b, _]: [u8; 4]) -> bool {
    r > 200 && g < 80 && b < 80
}

fn blue([r, g, b, _]: [u8; 4]) -> bool {
    b > 200 && r < 80 && g < 80
}

#[test]
fn a_sum_of_texts_flows_on_one_line_as_wide_as_its_parts() {
    let (plain, _) = measured(|| Text::new("Hello"));
    let (bold, _) = measured(|| Text::new("world").bold());
    let (plain_world, _) = measured(|| Text::new("world"));
    let (sum, height) = measured(|| Text::new("Hello") + Text::new("world").bold());
    assert!(bold > plain_world, "bold is wider");
    assert!((sum - (plain + bold)).abs() < 3.0, "{sum} {plain} {bold}");
    assert!(height < 24.0, "one line, not two: {height}");
}

#[test]
fn markdown_drops_its_marks_and_bolds_what_they_wrap() {
    let (marked, _) = measured(|| Text::new("**ab**"));
    let (bold, _) = measured(|| Text::markdown("**ab**"));
    let (plain, _) = measured(|| Text::new("ab"));
    assert!(bold < marked);
    assert!(bold > plain);
}

#[test]
fn markdown_without_marks_is_the_plain_text() {
    assert_eq!(
        measured(|| Text::markdown("2 * 3 * 4")),
        measured(|| Text::new("2 * 3 * 4"))
    );
}

#[test]
fn a_selectable_text_keeps_the_size_of_the_plain_one() {
    let plain = measured(|| Text::new("Copy me"));
    let selectable = measured(|| Text::new("Copy me").text_selection(true));
    assert!(
        (plain.0 - selectable.0).abs() < 1.0 && (plain.1 - selectable.1).abs() < 1.0,
        "{plain:?} {selectable:?}"
    );
}

#[test]
fn a_foreground_on_the_sum_colors_the_parts_that_set_none_and_an_inner_one_wins() {
    let inner = Rgba::rgba8(0, 0, 255, 1.0);
    let outer = Rgba::rgba8(255, 0, 0, 1.0);
    let mut test = runner(move || {
        (Text::new("MMMM").foreground_style(ShapeStyle::Color(inner)) + Text::new("MMMM"))
            .foreground_style(ShapeStyle::Color(outer))
            .into_element()
    });
    assert!(has_pixel(&mut test, blue), "the inner color stays");
    assert!(
        has_pixel(&mut test, red),
        "the outer color reaches the rest"
    );
}

fn role_and_focus(text: impl Fn() -> Text + 'static) -> (accesskit::Role, bool) {
    let test = runner(move || {
        text()
            .a11y_builder(|node| node.set_author_id("text"))
            .into_element()
    });
    test.find(|_, element| {
        (element.accessibility().builder.author_id() == Some("text")).then(|| {
            let data = element.accessibility();
            (
                data.builder.role(),
                data.a11y_focusable == freya_core::prelude::Focusable::Enabled,
            )
        })
    })
    .expect("the text is probed")
}

#[test]
fn only_a_selectable_text_takes_focus_and_it_keeps_the_label_role_the_edit_chords_look_for() {
    assert_eq!(
        role_and_focus(|| Text::new("Copy me").text_selection(true)),
        (accesskit::Role::Label, true)
    );
    assert!(!role_and_focus(|| Text::new("Copy me")).1);
}

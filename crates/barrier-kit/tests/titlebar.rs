//! The client-side titlebar honours the compositor's decoration answer.

use std::io::Cursor;

use barrier_kit::platform::{DecorationResponse, WindowStatus};
use barrier_kit::theme;
use barrier_kit::titlebar::Titlebar;
use freya_components::theming::hooks::use_init_theme;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::Size2D;

struct Case {
    status: WindowStatus,
}

impl App for Case {
    fn render(&self) -> impl IntoElement {
        Titlebar::preview(self.status, "Ventana", false)
    }
}

fn render(status: WindowStatus) -> Vec<u8> {
    let app = AppComponent::new(Case { status });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(300.0, 120.0), |_| {}, 1.0);
    test.sync_and_update();
    let png = test.render();
    let mut decoder = png::Decoder::new(Cursor::new(png.as_bytes()));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().unwrap();
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).unwrap();
    buffer[..info.buffer_size()].to_vec()
}

fn status(decoration: DecorationResponse) -> WindowStatus {
    WindowStatus {
        activated: true,
        maximized: false,
        decoration,
        fixed: false,
    }
}

#[test]
fn a_server_configure_has_no_titlebar_and_a_client_one_does() {
    let server = render(status(DecorationResponse::Server));
    let client = render(status(DecorationResponse::Client));
    assert_ne!(server, client, "the client configure must draw a bar");
}

struct StatefulCase;

impl App for StatefulCase {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(false));
        let status = use_consume::<State<WindowStatus>>();
        Titlebar::preview(*status.read(), "Ventana", false)
    }
}

#[test]
fn a_server_to_client_decoration_change_does_not_panic() {
    let app = AppComponent::new(StatefulCase);
    let (mut test, mut status_state) = TestingRunner::new(
        app,
        Size2D::new(300.0, 120.0),
        |runner| runner.provide_root_context(|| State::create(status(DecorationResponse::Server))),
        1.0,
    );
    test.sync_and_update();
    status_state.set(status(DecorationResponse::Client));
    test.sync_and_update();
    test.sync_and_update();
    assert!(!test.render().as_ref().is_empty());
}

const LONG_TITLE: &str =
    "Una ventana con un título deliberadamente largo que supera los cuarenta y dos caracteres";

struct TitleCase {
    title: &'static str,
}

impl App for TitleCase {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(false));
        Titlebar::preview(status(DecorationResponse::Client), self.title, false)
    }
}

fn title_label_style(test: &TestingRunner, title: &str) -> TextStyleData {
    test.find(|_node, el| {
        let label = Label::try_downcast(el)?;
        (label.text == title).then(|| el.text_style().into_owned())
    })
    .expect("the full title label exists")
}

fn title_label_width(test: &TestingRunner, title: &str) -> f32 {
    test.find(|node, el| {
        let label = Label::try_downcast(el)?;
        (label.text == title).then(|| node.layout().area.size.width)
    })
    .expect("the full title label exists")
}

fn title_area_width(test: &TestingRunner) -> f32 {
    test.find(|node, el| {
        (el.accessibility().builder.label() == Some("titlebar-title"))
            .then(|| node.layout().area.size.width)
    })
    .expect("the title slot exists")
}

#[test]
fn a_long_title_is_not_cut_to_42_characters_when_it_fits() {
    let app = AppComponent::new(TitleCase { title: LONG_TITLE });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(900.0, 120.0), |_| {}, 1.0);
    test.sync_and_update();
    let style = title_label_style(&test, LONG_TITLE);
    assert_eq!(style.text_overflow, Some(TextOverflow::Ellipsis));
}

#[test]
fn a_long_title_is_ellipsized_to_the_available_width() {
    let app = AppComponent::new(TitleCase { title: LONG_TITLE });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(200.0, 120.0), |_| {}, 1.0);
    test.sync_and_update();
    let style = title_label_style(&test, LONG_TITLE);
    assert_eq!(style.text_overflow, Some(TextOverflow::Ellipsis));
    assert!(
        title_label_width(&test, LONG_TITLE) <= title_area_width(&test) + 0.5,
        "the title label must be constrained to the title slot"
    );
}

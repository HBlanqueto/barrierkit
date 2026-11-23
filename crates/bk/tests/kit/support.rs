use std::rc::Rc;

use bk::theme;
use ctappearance::{AppearanceConfig, ColorScheme, resolve};
use ctdesign::SANS_FAMILIES;
use freya_components::theming::hooks::use_init_theme;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Area, Size2D};

pub(crate) fn area(test: &TestingRunner, id: &str) -> Area {
    test.find(|node, element| {
        (element.accessibility().builder.author_id() == Some(id)).then(|| node.layout().area)
    })
    .unwrap_or_else(|| panic!("no node probed as {id}"))
}

struct Themed {
    dark: bool,
    build: Rc<dyn Fn() -> Element>,
}

impl App for Themed {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(self.dark));
        rect().expanded().child((self.build)())
    }
}

pub(crate) fn runner(
    size: (f32, f32),
    config: AppearanceConfig,
    build: impl Fn() -> Element + 'static,
) -> TestingRunner {
    let dark = config.color_scheme.is_dark();
    let resolved = resolve(
        &AppearanceConfig {
            reduce_transparency: true,
            reduced_motion: true,
            ..config
        },
        None,
    );
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Themed {
            dark,
            build: Rc::new(build),
        }),
        Size2D::new(size.0, size.1),
        move |runner| runner.provide_root_context(move || State::create(resolved)),
        1.0,
    );
    test.set_default_fonts(&SANS_FAMILIES.map(std::borrow::Cow::Borrowed));
    test.sync_and_update();
    test.sync_and_update();
    test
}

pub(crate) fn differing(a: &[u8], b: &[u8]) -> usize {
    a.iter()
        .zip(b)
        .filter(|(a, b)| a.abs_diff(**b) > crate::controls::TOLERANCE)
        .count()
}

pub(crate) fn scheme(dark: bool) -> AppearanceConfig {
    AppearanceConfig {
        color_scheme: if dark {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        },
        ..AppearanceConfig::default()
    }
}

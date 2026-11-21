use bk::backdrop::{Accent, Backdrop, BackdropSource, Tint};
use bk::controls::{Checkbox, Slider, Switch};
use bk::progress::ProgressView;
use bk::views::form::{Form, Row, Section};
use ctappearance::{AppearanceConfig, ColorScheme, resolve};
use ctdesign::Rgba;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Size, Size2D};

const SIZE: (f32, f32) = (420.0, 300.0);
const COVER: [Rgba; 3] = [
    Rgba::rgb8(122, 59, 94),
    Rgba::rgb8(45, 110, 106),
    Rgba::rgb8(180, 112, 63),
];

struct Case {
    source: BackdropSource,
}

impl App for Case {
    fn render(&self) -> impl IntoElement {
        let controls = Form::new().section(
            Section::titled("Mezcla")
                .row(
                    Row::new("Fundido")
                        .control(Switch::new("Fundido").on(true).tint(Tint::Backdrop)),
                )
                .row(Row::new("Letras").control(Checkbox::new("Letras").checked(true)))
                .row(
                    Row::new("Graves")
                        .control(Slider::stepped("Graves", 5).value(3.0).tint(Tint::Backdrop)),
                ),
        );
        Backdrop::new(
            self.source.clone(),
            rect()
                .width(Size::fill())
                .padding(16.0)
                .spacing(12.0)
                .child(controls)
                .child(
                    ProgressView::determinate("Exportando", 0.6)
                        .tint(Tint::Color(Rgba::rgb8(240, 164, 58))),
                ),
        )
        .accent(Accent::FromBackdrop)
    }
}

fn render(app: AppComponent, scheme: ColorScheme) -> Vec<u8> {
    let resolved = resolve(
        &AppearanceConfig {
            color_scheme: scheme,
            reduced_motion: true,
            ..Default::default()
        },
        None,
    );
    let (mut test, _) = TestingRunner::new(
        app,
        Size2D::new(SIZE.0, SIZE.1),
        move |runner| {
            runner.provide_root_context(move || State::create(resolved));
        },
        1.0,
    );
    test.sync_and_update();
    test.sync_and_update();
    test.render().as_bytes().to_vec()
}

#[test]
fn a_tinted_backdrop_looks_the_same_in_light_and_dark() {
    let light = render(
        AppComponent::new(Case {
            source: BackdropSource::Colors(COVER),
        }),
        ColorScheme::Light,
    );
    let dark = render(
        AppComponent::new(Case {
            source: BackdropSource::Colors(COVER),
        }),
        ColorScheme::Dark,
    );
    assert!(
        light == dark,
        "the tinted window changed with the system scheme"
    );
}

#[test]
fn without_a_tint_the_window_follows_the_system_scheme() {
    let light = render(
        AppComponent::new(Case {
            source: BackdropSource::None,
        }),
        ColorScheme::Light,
    );
    let dark = render(
        AppComponent::new(Case {
            source: BackdropSource::None,
        }),
        ColorScheme::Dark,
    );
    assert!(light != dark);
}

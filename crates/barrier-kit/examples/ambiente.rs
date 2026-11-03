use barrier_kit::backdrop::{Accent, Backdrop, BackdropSource, Tint};
use barrier_kit::controls::Switch;
use barrier_kit::level::LevelSlider;
use barrier_kit::platform::{WindowOptions, run_window};
use barrier_kit::theme::use_appearance;
use barrier_kit::titlebar::Titlebar;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{Direction, Gaps, Size};
use vow_appearance::ColorScheme;
use vow_design::{ACCENT_SWATCHES, SPACE_3, SPACE_4};

struct Ambiente;

impl App for Ambiente {
    fn render(&self) -> impl IntoElement {
        use_appearance(|config| config.color_scheme = ColorScheme::Dark);
        let colores = [
            ACCENT_SWATCHES[1].1,
            ACCENT_SWATCHES[4].1,
            ACCENT_SWATCHES[0].1,
        ];
        Backdrop::new(BackdropSource::Colors(colores), Controles).accent(Accent::FromBackdrop)
    }
}

#[derive(PartialEq)]
struct Controles;

impl Component for Controles {
    fn render(&self) -> impl IntoElement {
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .child(Titlebar::new().title("Ambiente"))
            .child(
                rect()
                    .direction(Direction::Vertical)
                    .padding(Gaps::new_all(SPACE_4))
                    .spacing(SPACE_3)
                    .child(Switch::new("Fundido").on(true).tint(Tint::Backdrop))
                    .child(
                        LevelSlider::new("Volumen")
                            .value(0.6)
                            .tint(Tint::Color(ACCENT_SWATCHES[5].1)),
                    ),
            )
    }
}

fn main() {
    run_window(
        AppComponent::new(Ambiente),
        WindowOptions::new("Ambiente", "ambiente").size((480, 320)),
    )
    .expect("ambiente");
}

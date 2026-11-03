use barrier_kit::controls::{Button, ButtonStyle};
use barrier_kit::platform::{WindowOptions, run_window};
use barrier_kit::style::{Palette, text};
use barrier_kit::theme::color;
use barrier_kit::titlebar::Titlebar;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Size};
use vow_design::{SPACE_3, TITLE_1, WINDOW_MIN_SIZE};

struct Contador;

impl App for Contador {
    fn render(&self) -> impl IntoElement {
        let mut cuenta = use_state(|| 0);
        let palette = Palette::current();
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .background(color(palette.window_background))
            .child(Titlebar::new().title("Contador"))
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::fill())
                    .direction(Direction::Vertical)
                    .main_align(Alignment::Center)
                    .cross_align(Alignment::Center)
                    .spacing(SPACE_3)
                    .child(text(cuenta.read().to_string(), TITLE_1, palette.label))
                    .child(
                        Button::new("Sumar uno")
                            .style(ButtonStyle::Primary)
                            .on_press(move |_| *cuenta.write() += 1),
                    ),
            )
    }
}

fn main() {
    run_window(
        AppComponent::new(Contador),
        WindowOptions::new("Contador", "contador")
            .size((420, 300))
            .min_size(Some(WINDOW_MIN_SIZE)),
    )
    .expect("contador");
}

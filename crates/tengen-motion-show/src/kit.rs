use barrier_kit::backdrop::{Accent, Backdrop, BackdropSource, Tint};
use barrier_kit::controls::{Button, Switch};
use barrier_kit::level::LevelSlider;
use barrier_kit::progress::{ProgressBar, ProgressRing};
use barrier_kit::sidebar::{Sidebar, SidebarItem, SidebarSection};
use barrier_kit::symbol::Symbol;
use freya_animation::prelude::{AnimNum, Ease, Function, use_animation};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Size};
use vow_design::{ACCENT_SWATCHES, DURATION_SLOW_MS, Rgba, SPACE_2, SPLIT_VIEW};

use crate::motion::{time, use_reduced_motion};

const DOWNLOAD_MS: u64 = DURATION_SLOW_MS * 6;

pub const SCENES: [(&str, [Rgba; 3]); 3] = [
    (
        "Atardecer",
        [
            Rgba::rgb8(122, 59, 94),
            Rgba::rgb8(45, 110, 106),
            Rgba::rgb8(180, 112, 63),
        ],
    ),
    (
        "Noche",
        [
            Rgba::rgb8(20, 33, 61),
            Rgba::rgb8(58, 79, 122),
            Rgba::rgb8(150, 110, 40),
        ],
    ),
    (
        "Pálido",
        [
            Rgba::rgb8(236, 214, 196),
            Rgba::rgb8(200, 222, 214),
            Rgba::rgb8(240, 226, 170),
        ],
    ),
];

pub fn sidebar_section() -> Rect {
    let tint = |index: usize| ACCENT_SWATCHES[index].1;
    rect()
        .width(Size::px(SPLIT_VIEW.sidebar_min))
        .height(Size::px(SPLIT_VIEW.inspector * 0.75))
        .child(
            Sidebar::new().child(
                SidebarSection::new("Favoritos")
                    .child(SidebarItem::new("Escritorio").symbol(Symbol::SquareGrid, tint(0)))
                    .child(SidebarItem::new("Música").symbol(Symbol::Play, tint(1))),
            ),
        )
}

pub fn scene(index: usize) -> Rect {
    rect()
        .width(Size::px(SPLIT_VIEW.sidebar_max))
        .height(Size::px(SPLIT_VIEW.inspector / 2.0))
        .corner_radius(12.0)
        .overflow(Overflow::Clip)
        .child(
            Backdrop::new(
                BackdropSource::Colors(SCENES[index].1),
                rect()
                    .width(Size::fill())
                    .padding(16.0)
                    .spacing(SPACE_2)
                    .child(Switch::new("Encendido").on(true).tint(Tint::Accent))
                    .child(LevelSlider::new("Nivel").value(0.4).tint(Tint::Accent)),
            )
            .accent(Accent::FromBackdrop),
        )
}

#[derive(PartialEq)]
pub struct Download;

impl Component for Download {
    fn render(&self) -> impl IntoElement {
        let reduced_motion = use_reduced_motion();
        let download = use_animation(move |_| {
            AnimNum::new(0.0, 1.0)
                .time(time(reduced_motion, DOWNLOAD_MS))
                .ease(Ease::InOut)
                .function(Function::Sine)
        });
        let done = download.get().value();
        let mut replay = download.clone();
        rect()
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(SPACE_2)
            .child(
                rect()
                    .width(Size::px(SPLIT_VIEW.inspector))
                    .child(ProgressBar::determinate("Descargando", done)),
            )
            .child(ProgressRing::new("Descargando", done))
            .child(Button::new("Descargar").on_press(move |_| replay.start()))
    }
}

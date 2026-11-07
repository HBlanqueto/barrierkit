use std::sync::LazyLock;

use barrier_kit::controls::Button;
use barrier_kit::progress::{ProgressBar, ProgressRing};
use barrier_kit::sidebar::{Sidebar, SidebarItem, SidebarSection};
use barrier_kit::symbol::Symbol;
use freya_animation::prelude::{AnimNum, Ease, Function, use_animation};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Size};
use vow_design::{ACCENT_SWATCHES, DURATION_SLOW_MS, Rgba, SPACE_2, SPLIT_VIEW};

use crate::localize::tr;
use barrier_kit::animation::{motion_ms, use_reduced_motion};

const DOWNLOAD_MS: u64 = DURATION_SLOW_MS * 6;

pub static SCENES: LazyLock<[(&str, [Rgba; 3]); 3]> = LazyLock::new(|| {
    [
        (
            tr!("dusk"),
            [
                Rgba::rgb8(122, 59, 94),
                Rgba::rgb8(45, 110, 106),
                Rgba::rgb8(180, 112, 63),
            ],
        ),
        (
            tr!("night"),
            [
                Rgba::rgb8(20, 33, 61),
                Rgba::rgb8(58, 79, 122),
                Rgba::rgb8(150, 110, 40),
            ],
        ),
        (
            tr!("pale"),
            [
                Rgba::rgb8(236, 214, 196),
                Rgba::rgb8(200, 222, 214),
                Rgba::rgb8(240, 226, 170),
            ],
        ),
    ]
});

pub fn sidebar_section() -> Rect {
    let tint = |index: usize| ACCENT_SWATCHES[index].1;
    rect()
        .width(Size::px(SPLIT_VIEW.sidebar_min))
        .height(Size::px(SPLIT_VIEW.inspector * 0.75))
        .child(
            Sidebar::new().child(
                SidebarSection::new(tr!("favorites"))
                    .child(SidebarItem::new(tr!("desktop")).symbol(Symbol::SquareGrid, tint(0)))
                    .child(SidebarItem::new(tr!("music")).symbol(Symbol::Play, tint(1))),
            ),
        )
}

#[derive(PartialEq)]
pub struct Download;

impl Component for Download {
    fn render(&self) -> impl IntoElement {
        let reduced_motion = use_reduced_motion();
        let download = use_animation(move |_| {
            AnimNum::new(0.0, 1.0)
                .time(motion_ms(reduced_motion, DOWNLOAD_MS))
                .ease(Ease::InOut)
                .function(Function::Sine)
        });
        let done = download.get().value();
        let mut replay = download;
        rect()
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(SPACE_2)
            .child(
                rect()
                    .width(Size::px(SPLIT_VIEW.inspector))
                    .child(ProgressBar::determinate(tr!("downloading"), done)),
            )
            .child(ProgressRing::new(tr!("downloading"), done))
            .child(Button::new(tr!("download")).on_press(move |_| replay.start()))
    }
}

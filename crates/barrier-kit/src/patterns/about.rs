use std::borrow::Cow;

use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Gaps, Size};
use vow_design::{CAPTION, DIALOG_PADDING, SPACE_1, SPACE_3, TEXT, TITLE_1};

use crate::style::{Palette, text};

pub fn about_window(
    icon: impl IntoElement,
    name: impl Into<Cow<'static, str>>,
    version: Option<impl Into<Cow<'static, str>>>,
    copyright: impl Into<Cow<'static, str>>,
) -> Rect {
    let palette = Palette::current();
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .direction(Direction::Vertical)
        .cross_align(Alignment::Center)
        .main_align(Alignment::Center)
        .padding(Gaps::new_all(DIALOG_PADDING))
        .spacing(SPACE_3)
        .child(icon)
        .child(
            rect()
                .direction(Direction::Vertical)
                .cross_align(Alignment::Center)
                .spacing(SPACE_1)
                .child(text(name, TITLE_1, palette.label))
                .maybe_child(version.map(|version| text(version, TEXT, palette.secondary_label))),
        )
        .child(text(copyright, CAPTION, palette.tertiary_label))
}

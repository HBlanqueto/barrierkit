use std::borrow::Cow;

use ctdesign::{BODY, CAPTION, DIALOG_PADDING, SPACE_1, SPACE_3, TITLE};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Gaps};

use crate::style::{Palette, text};

pub struct AboutView {
    icon: Element,
    name: Cow<'static, str>,
    version: Option<Cow<'static, str>>,
    copyright: Cow<'static, str>,
}

impl AboutView {
    pub fn new(
        icon: impl IntoElement,
        name: impl Into<Cow<'static, str>>,
        copyright: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            icon: icon.into_element(),
            name: name.into(),
            version: None,
            copyright: copyright.into(),
        }
    }

    pub fn version(mut self, version: impl Into<Cow<'static, str>>) -> Self {
        self.version = Some(version.into());
        self
    }
}

impl From<AboutView> for Element {
    fn from(about: AboutView) -> Self {
        about_window(about.icon, about.name, about.version, about.copyright).into()
    }
}

fn about_window(
    icon: impl IntoElement,
    name: impl Into<Cow<'static, str>>,
    version: Option<impl Into<Cow<'static, str>>>,
    copyright: impl Into<Cow<'static, str>>,
) -> Rect {
    let palette = Palette::current();
    rect()
        .expanded()
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
                .child(text(name, TITLE, palette.label))
                .maybe_child(version.map(|version| text(version, BODY, palette.secondary_label))),
        )
        .child(text(copyright, CAPTION, palette.tertiary_label))
}

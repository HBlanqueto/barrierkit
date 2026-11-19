use std::borrow::Cow;

use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Gaps};

use crate::style::text;
use crate::styles::{AboutViewStyle, ControlState, Paints, ShapeStyle, StateValue, Theme, resolve};

pub struct AboutView {
    icon: Element,
    name: Cow<'static, str>,
    version: Option<Cow<'static, str>>,
    copyright: Cow<'static, str>,
    style: Theme,
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
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(AboutViewStyle) -> AboutViewStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn version(mut self, version: impl Into<Cow<'static, str>>) -> Self {
        self.version = Some(version.into());
        self
    }
}

impl From<AboutView> for Element {
    fn from(about: AboutView) -> Self {
        about_window(about).into()
    }
}

fn about_window(about: AboutView) -> Rect {
    let style: AboutViewStyle = resolve(&about.style);
    let state = ControlState::default();
    let paints = Paints::current(state, style.effect);
    let ink = |paint: &StateValue<ShapeStyle>| paints.color(paint.resolve(state));
    rect()
        .expanded()
        .direction(Direction::Vertical)
        .cross_align(Alignment::Center)
        .main_align(Alignment::Center)
        .padding(Gaps::new_all(style.padding))
        .spacing(style.spacing)
        .child(about.icon)
        .child(
            rect()
                .direction(Direction::Vertical)
                .cross_align(Alignment::Center)
                .spacing(style.name_spacing)
                .child(text(about.name, style.name_font, ink(&style.name)))
                .maybe_child(
                    about
                        .version
                        .map(|version| text(version, style.version_font, ink(&style.version))),
                ),
        )
        .child(text(
            about.copyright,
            style.copyright_font,
            ink(&style.copyright),
        ))
}

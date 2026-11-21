use std::borrow::Cow;
use std::path::PathBuf;

use freya_components::image_viewer::ImageViewer;
use freya_core::prelude::*;
use torin::prelude::{Gaps, Size};

use crate::controls::{dimmed, shadowed};
use crate::elements::{Overrides, forward_container, forward_overrides};
use crate::interaction::{control_state, use_interaction};
use crate::style::{text, with_font};
use crate::styles::Painted;
use crate::styles::{
    AvatarStyle, BadgeStyle, ControlState, DividerStyle, LayoutDirection, LinkStyle, Paints, Theme,
    resolve,
};

pub struct Badge(Rect, Cow<'static, str>, Theme);

impl Badge {
    pub fn new(content: impl Into<Cow<'static, str>>) -> Self {
        Self::styled(content.into(), Theme::tengen())
    }

    pub fn style(self, restyle: impl Fn(BadgeStyle) -> BadgeStyle + 'static) -> Self {
        Self::styled(self.1, self.2.style(restyle))
    }

    fn styled(content: Cow<'static, str>, theme: Theme) -> Self {
        let style: BadgeStyle = resolve(&theme);
        let paints = Paints::current(ControlState::default(), style.effect);
        let ink = paints.color(style.label.resolve(ControlState::default()));
        let badge = rect()
            .height(Size::px(style.height))
            .min_width(Size::px(style.height))
            .padding(Gaps::new(0.0, style.padding_x, 0.0, style.padding_x))
            .corner_radius(style.shape.corner_radius(
                style.height,
                style.height,
                LayoutDirection::LeftToRight,
            ))
            .center()
            .painted(&paints, &style.background)
            .bordered(&paints, style.border.as_ref())
            .child(text(content.clone(), style.font, ink));
        Self(shadowed(badge, &style.shadows), content, theme)
    }
}

forward_container!(Badge);

pub struct Divider(Rect, Theme);

impl Divider {
    pub fn new() -> Self {
        Self::styled(Theme::tengen())
    }

    pub fn style(self, restyle: impl Fn(DividerStyle) -> DividerStyle + 'static) -> Self {
        Self::styled(self.1.style(restyle))
    }

    fn styled(theme: Theme) -> Self {
        let style: DividerStyle = resolve(&theme);
        let paints = Paints::current(ControlState::default(), style.effect);
        Self(
            rect()
                .a11y_role(AccessibilityRole::Splitter)
                .width(Size::fill())
                .height(Size::px(style.thickness))
                .background(paints.fill(&style.line)),
            theme,
        )
    }
}

impl Default for Divider {
    fn default() -> Self {
        Self::new()
    }
}

forward_container!(Divider);

pub struct Avatar(Rect, String, Theme, Option<PathBuf>);

pub fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

impl Avatar {
    pub fn new(initials: &str) -> Self {
        Self::styled(initials.to_string(), Theme::tengen(), None)
    }

    pub fn named(name: &str) -> Self {
        Self::new(&initials(name))
    }

    pub fn picture(self, path: impl Into<PathBuf>) -> Self {
        Self::styled(self.1, self.2, Some(path.into()))
    }

    pub fn style(self, restyle: impl Fn(AvatarStyle) -> AvatarStyle + 'static) -> Self {
        Self::styled(self.1, self.2.style(restyle), self.3)
    }

    fn styled(initials: String, theme: Theme, picture: Option<PathBuf>) -> Self {
        let style: AvatarStyle = resolve(&theme);
        let paints = Paints::current(ControlState::default(), style.effect);
        let ink = paints.color(style.label.resolve(ControlState::default()));
        let avatar = rect()
            .a11y_role(AccessibilityRole::Image)
            .a11y_alt(initials.clone())
            .width(Size::px(style.size))
            .height(Size::px(style.size))
            .corner_radius(style.shape.corner_radius(
                style.size,
                style.size,
                LayoutDirection::LeftToRight,
            ))
            .center()
            .painted(&paints, &style.background)
            .bordered(&paints, style.border.as_ref());
        let avatar = match &picture {
            Some(path) => avatar.overflow(Overflow::Clip).child(
                ImageViewer::new(path.clone())
                    .width(Size::fill())
                    .height(Size::fill())
                    .aspect_ratio(AspectRatio::Max)
                    .image_cover(ImageCover::Center),
            ),
            None => avatar.child(text(initials.clone(), style.font, ink)),
        };
        Self(shadowed(avatar, &style.shadows), initials, theme, picture)
    }
}

forward_container!(Avatar);

#[derive(PartialEq)]
pub struct Link {
    text: Cow<'static, str>,
    on_press: Option<Callback<(), ()>>,
    style: Theme,
    overrides: Overrides,
}

impl Link {
    pub fn new(text: impl Into<Cow<'static, str>>) -> Self {
        Self {
            text: text.into(),
            on_press: None,
            style: Theme::tengen(),
            overrides: Overrides::default(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(LinkStyle) -> LinkStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn on_press(mut self, callback: impl Into<Callback<(), ()>>) -> Self {
        self.on_press = Some(callback.into());
        self
    }
}

forward_overrides!(Link);

impl Component for Link {
    fn render(&self) -> impl IntoElement {
        let style: LinkStyle = resolve(&self.style);
        let interaction = use_interaction();
        let paints = Paints::current(control_state(interaction, false), style.effect);
        let on_press = self.on_press.clone();
        let link = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Link)
            .a11y_alt(&*self.text)
            .map(on_press, |el, on_press| {
                el.on_press(move |_| on_press.call(()))
            })
            .child(
                with_font(label().text(self.text.clone()), style.font)
                    .max_lines(1)
                    .color(paints.fill(&style.label)),
            );
        self.overrides
            .apply(interaction.focus_ring(dimmed(link, style.opacity), 0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::initials;

    #[test]
    fn initials_take_the_first_letter_of_the_first_two_words() {
        assert_eq!(initials("ana lopez"), "AL");
        assert_eq!(initials("pedro"), "P");
        assert_eq!(initials("Maria de la Cruz"), "MD");
        assert_eq!(initials(""), "");
    }
}

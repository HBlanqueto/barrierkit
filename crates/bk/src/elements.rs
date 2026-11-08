use std::borrow::Cow;

use ctdesign::{Font, SPACE_3};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction};

use crate::style::with_font;
use crate::styles::{ControlState, Paints, TextStyle, Theme, resolve};

macro_rules! forward_element {
    ($outer:ident) => {
        impl From<$outer> for ::freya_core::prelude::Element {
            fn from(value: $outer) -> Self {
                value.0.into()
            }
        }

        impl ::freya_core::prelude::KeyExt for $outer {
            fn key_mut(&mut self) -> &mut ::freya_core::prelude::DiffKey {
                self.0.key_mut()
            }
        }

        impl ::freya_core::prelude::EventHandlersExt for $outer {
            fn event_handlers_mut(&mut self) -> &mut ::freya_core::integration::EventHandlers {
                self.0.event_handlers_mut()
            }
        }

        impl ::freya_core::prelude::AccessibilityExt for $outer {
            fn accessibility_data_mut(&mut self) -> &mut ::freya_core::prelude::AccessibilityData {
                self.0.accessibility_data_mut()
            }
        }

        impl ::freya_core::prelude::TextStyleExt for $outer {
            fn text_style_data_mut(&mut self) -> &mut ::freya_core::prelude::TextStyleData {
                self.0.text_style_data_mut()
            }
        }

        impl ::freya_core::prelude::LayerExt for $outer {
            fn layer_mut(&mut self) -> &mut ::freya_core::prelude::Layer {
                self.0.layer_mut()
            }
        }

        impl ::freya_core::prelude::LayoutExt for $outer {
            fn layout_mut(&mut self) -> &mut ::freya_core::prelude::LayoutData {
                self.0.layout_mut()
            }
        }

        impl ::freya_core::prelude::EffectExt for $outer {
            fn effect_mut(&mut self) -> &mut ::freya_core::prelude::EffectData {
                ::freya_core::prelude::EffectExt::effect_mut(&mut self.0)
            }
        }

        impl ::freya_core::prelude::ContainerExt for $outer {}

        impl ::freya_core::prelude::MaybeExt for $outer {}
    };
}

macro_rules! forward_container {
    ($outer:ident) => {
        $crate::elements::forward_element!($outer);

        impl ::freya_core::prelude::ChildrenExt for $outer {
            fn children_mut(&mut self) -> &mut Vec<::freya_core::prelude::Element> {
                self.0.children_mut()
            }
        }

        impl ::freya_core::prelude::StyleExt for $outer {
            fn style_mut(&mut self) -> &mut ::freya_core::prelude::StyleState {
                self.0.style_mut()
            }
        }

        impl ::freya_core::prelude::ContainerWithContentExt for $outer {}

        impl ::freya_core::prelude::ScrollableExt for $outer {
            fn effect_mut(&mut self) -> &mut ::freya_core::prelude::EffectData {
                ::freya_core::prelude::ScrollableExt::effect_mut(&mut self.0)
            }
        }

        impl ::freya_core::prelude::BackgroundExt for $outer {}

        impl ::freya_core::prelude::DecorationExt for $outer {}
    };
}

pub(crate) use forward_container;
pub(crate) use forward_element;

pub struct Text(Label, Cow<'static, str>, Theme);

impl Text {
    pub fn new(content: impl Into<Cow<'static, str>>) -> Self {
        Self::styled(content.into(), Theme::tengen())
    }

    pub fn style(self, restyle: impl Fn(TextStyle) -> TextStyle + 'static) -> Self {
        Self::styled(self.1, self.2.style(restyle))
    }

    pub fn font(self, font: Font) -> Self {
        let font = font.scaled(crate::theme::resolved().text_size);
        Self(with_font(self.0, font), self.1, self.2)
    }

    pub fn line_limit(self, lines: usize) -> Self {
        Self(self.0.max_lines(lines), self.1, self.2)
    }

    fn styled(content: Cow<'static, str>, theme: Theme) -> Self {
        let style: TextStyle = resolve(&theme);
        let paints = Paints::current(ControlState::default(), style.effect);
        let view = with_font(label().text(content.clone()), style.font)
            .max_lines(None)
            .color(paints.fill(&style.foreground));
        Self(view, content, theme)
    }
}

forward_element!(Text);

impl TextOverflowExt for Text {}

pub struct VStack(Rect);

impl VStack {
    pub fn new() -> Self {
        Self(stack(Direction::Vertical))
    }
}

impl Default for VStack {
    fn default() -> Self {
        Self::new()
    }
}

forward_container!(VStack);

pub struct HStack(Rect);

impl HStack {
    pub fn new() -> Self {
        Self(stack(Direction::Horizontal))
    }
}

impl Default for HStack {
    fn default() -> Self {
        Self::new()
    }
}

forward_container!(HStack);

fn stack(direction: Direction) -> Rect {
    rect()
        .direction(direction)
        .main_align(Alignment::Center)
        .cross_align(Alignment::Center)
        .spacing(SPACE_3)
}

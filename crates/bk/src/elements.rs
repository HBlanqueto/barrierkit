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
            fn write_key(&mut self) -> &mut ::freya_core::prelude::DiffKey {
                self.0.write_key()
            }
        }

        impl ::freya_core::prelude::EventHandlersExt for $outer {
            fn get_event_handlers(
                &mut self,
            ) -> &mut ::freya_core::integration::FxHashMap<
                ::freya_core::events::name::EventName,
                ::freya_core::integration::EventHandlerType,
            > {
                self.0.get_event_handlers()
            }
        }

        impl ::freya_core::prelude::AccessibilityExt for $outer {
            fn get_accessibility_data(&mut self) -> &mut ::freya_core::prelude::AccessibilityData {
                self.0.get_accessibility_data()
            }
        }

        impl ::freya_core::prelude::TextStyleExt for $outer {
            fn get_text_style_data(&mut self) -> &mut ::freya_core::prelude::TextStyleData {
                self.0.get_text_style_data()
            }
        }

        impl ::freya_core::prelude::LayerExt for $outer {
            fn get_layer(&mut self) -> &mut ::freya_core::prelude::Layer {
                self.0.get_layer()
            }
        }

        impl ::freya_core::prelude::LayoutExt for $outer {
            fn get_layout(&mut self) -> &mut ::freya_core::prelude::LayoutData {
                self.0.get_layout()
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
            fn get_children(&mut self) -> &mut Vec<::freya_core::prelude::Element> {
                self.0.get_children()
            }
        }

        impl ::freya_core::prelude::StyleExt for $outer {
            fn get_style(&mut self) -> &mut ::freya_core::prelude::StyleState {
                self.0.get_style()
            }
        }

        impl ::freya_core::prelude::ContainerWithContentExt for $outer {}

        impl ::freya_core::prelude::ScrollableExt for $outer {
            fn get_effect(&mut self) -> &mut ::freya_core::prelude::EffectData {
                ::freya_core::prelude::ScrollableExt::get_effect(&mut self.0)
            }
        }

        impl ::freya_core::prelude::EffectExt for $outer {
            fn get_effect(&mut self) -> &mut ::freya_core::prelude::EffectData {
                ::freya_core::prelude::EffectExt::get_effect(&mut self.0)
            }
        }
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
        Self(with_font(self.0, font), self.1, self.2)
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

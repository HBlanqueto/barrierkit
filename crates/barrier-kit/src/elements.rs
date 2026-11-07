use std::borrow::Cow;

use ct_design::{BODY, Font, SPACE_3};
use freya_core::events::name::EventName;
use freya_core::integration::{EventHandlerType, FxHashMap};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction};

use crate::style::{Palette, text, with_font};

macro_rules! forward_element {
    ($outer:ident) => {
        impl From<$outer> for Element {
            fn from(value: $outer) -> Self {
                value.0.into()
            }
        }

        impl KeyExt for $outer {
            fn write_key(&mut self) -> &mut DiffKey {
                self.0.write_key()
            }
        }

        impl EventHandlersExt for $outer {
            fn get_event_handlers(&mut self) -> &mut FxHashMap<EventName, EventHandlerType> {
                self.0.get_event_handlers()
            }
        }

        impl AccessibilityExt for $outer {
            fn get_accessibility_data(&mut self) -> &mut AccessibilityData {
                self.0.get_accessibility_data()
            }
        }

        impl TextStyleExt for $outer {
            fn get_text_style_data(&mut self) -> &mut TextStyleData {
                self.0.get_text_style_data()
            }
        }

        impl LayerExt for $outer {
            fn get_layer(&mut self) -> &mut Layer {
                self.0.get_layer()
            }
        }

        impl LayoutExt for $outer {
            fn get_layout(&mut self) -> &mut LayoutData {
                self.0.get_layout()
            }
        }

        impl ContainerExt for $outer {}

        impl MaybeExt for $outer {}
    };
}

macro_rules! forward_container {
    ($outer:ident) => {
        forward_element!($outer);

        impl ChildrenExt for $outer {
            fn get_children(&mut self) -> &mut Vec<Element> {
                self.0.get_children()
            }
        }

        impl StyleExt for $outer {
            fn get_style(&mut self) -> &mut StyleState {
                self.0.get_style()
            }
        }

        impl ContainerWithContentExt for $outer {}

        impl ScrollableExt for $outer {
            fn get_effect(&mut self) -> &mut EffectData {
                ScrollableExt::get_effect(&mut self.0)
            }
        }

        impl EffectExt for $outer {
            fn get_effect(&mut self) -> &mut EffectData {
                EffectExt::get_effect(&mut self.0)
            }
        }
    };
}

pub struct Text(Label);

impl Text {
    pub fn new(content: impl Into<Cow<'static, str>>) -> Self {
        Self(text(content, BODY, Palette::current().label).max_lines(None))
    }

    pub fn font(self, font: Font) -> Self {
        Self(with_font(self.0, font))
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

use std::borrow::Cow;

use ctdesign::{
    Font, LABEL_GAP, LABEL_ICON_FRAME, LABEL_ICON_HEIGHT, LABEL_ICON_WIDTH, SPACE_1, SPACE_2,
};
use freya_core::integration::EventHandlers;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Size};

use crate::style::Palette;
use crate::style::with_font;
use crate::styles::{
    ControlState, InTextStyleScope, Paints, ShapeStyle, TextStyle, Theme, resolve,
};
use crate::symbol::{Image, Symbol};
use crate::views::list::ListRow;

macro_rules! forward_element {
    ($outer:ident) => {
        impl From<$outer> for ::freya_core::prelude::Element {
            fn from(value: $outer) -> Self {
                $crate::layout::resolve_spacers(value.0.into())
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

macro_rules! forward_overrides {
    ($outer:ident) => {
        impl ::freya_core::prelude::LayoutExt for $outer {
            fn layout_mut(&mut self) -> &mut ::freya_core::prelude::LayoutData {
                &mut self.overrides.layout
            }
        }

        impl ::freya_core::prelude::StyleExt for $outer {
            fn style_mut(&mut self) -> &mut ::freya_core::prelude::StyleState {
                &mut self.overrides.style
            }
        }

        impl ::freya_core::prelude::EffectExt for $outer {
            fn effect_mut(&mut self) -> &mut ::freya_core::prelude::EffectData {
                &mut self.overrides.effect
            }
        }

        impl ::freya_core::prelude::EventHandlersExt for $outer {
            fn event_handlers_mut(&mut self) -> &mut ::freya_core::integration::EventHandlers {
                &mut self.overrides.handlers
            }
        }

        impl ::freya_core::prelude::ContainerExt for $outer {}

        impl ::freya_core::prelude::BackgroundExt for $outer {}

        impl ::freya_core::prelude::DecorationExt for $outer {}
    };
}

pub(crate) use forward_container;
pub(crate) use forward_element;
pub(crate) use forward_overrides;

#[derive(Default, Clone, PartialEq)]
pub(crate) struct Overrides {
    pub(crate) layout: LayoutData,
    pub(crate) style: StyleState,
    pub(crate) effect: EffectData,
    pub(crate) handlers: EventHandlers,
}

macro_rules! take_set {
    ($root:expr, $set:expr, $unset:expr; $($field:ident),+) => {
        $(if $set.$field != $unset.$field {
            $root.$field = $set.$field.clone();
        })+
    };
}

impl Overrides {
    // A field equal to its default counts as unset, so the control keeps its own value.
    // An Event cannot be cloned, so handlers cannot chain with the control's own: they sit around it.
    pub(crate) fn apply<E>(&self, mut root: E) -> Element
    where
        E: LayoutExt + StyleExt + EffectExt + IntoElement,
    {
        let unset = Self::default();
        take_set!(root.layout_mut().layout, self.layout.layout, unset.layout.layout;
            width, height, minimum_width, minimum_height, maximum_width, maximum_height,
            visible_width, visible_height, padding, margin, position);
        take_set!(root.style_mut(), self.style, unset.style; background, corner_radius);
        root.style_mut()
            .borders
            .extend(self.style.borders.iter().cloned());
        root.style_mut()
            .shadows
            .extend(self.style.shadows.iter().cloned());
        if self.effect != unset.effect {
            take_set!(root.effect_mut(), self.effect, unset.effect;
                cursor, cursor_visible, overflow, rotation, scale, transform_origin, opacity, blur);
        }
        if self.handlers.is_empty() {
            root.into_element()
        } else {
            rect()
                .event_handlers(self.handlers.clone())
                .child(root)
                .into_element()
        }
    }

    pub(crate) fn delegate<C>(&self, inner: C) -> C
    where
        C: LayoutExt + StyleExt + EffectExt + EventHandlersExt,
    {
        inner
            .layout(self.layout.clone())
            .style(self.style.clone())
            .effect(self.effect.clone())
            .event_handlers(self.handlers.clone())
    }
}

pub struct Text(freya_core::prelude::Label, Cow<'static, str>, Theme);

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

pub struct Label(Rect);

impl Label {
    pub fn new(title: impl Into<Cow<'static, str>>, image: Symbol) -> Self {
        let style: TextStyle = resolve(&Theme::tengen());
        let paints = Paints::current(ControlState::default(), style.effect);
        let in_list = try_consume_context::<ListRow>().is_some();
        let compact = !in_list && try_consume_context::<InTextStyleScope>().is_some();
        let tint = ShapeStyle::Color(Palette::current().accent);
        let foreground = if in_list {
            &tint
        } else {
            style.foreground.resolve(ControlState::default())
        };
        let icon = Image::system(image)
            .size(style.font.size as f32 * if compact { 1.0 } else { LABEL_ICON_FRAME })
            .color(paints.color(foreground));
        let icon = if compact {
            icon.into_element()
        } else {
            rect()
                .width(Size::px(LABEL_ICON_WIDTH))
                .height(Size::px(LABEL_ICON_HEIGHT))
                .center()
                .child(icon)
                .into_element()
        };
        Self(
            stack(Direction::Horizontal)
                .spacing(if compact { SPACE_1 } else { LABEL_GAP })
                .child(icon)
                .child(Text::new(title)),
        )
    }
}

forward_container!(Label);

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
        .spacing(SPACE_2)
}

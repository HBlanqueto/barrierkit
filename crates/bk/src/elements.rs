use std::any::TypeId;
use std::borrow::Cow;

use ctdesign::{
    Font, LABEL_GAP, LABEL_ICON_FRAME, LABEL_ICON_HEIGHT, LABEL_ICON_WIDTH, SPACE_1, SPACE_2,
    STACK_COLUMN_GAPS,
};
use freya_core::integration::EventHandlers;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Size};

use crate::controls::Checkbox;
use crate::rich_text::styled_spans;
use crate::style::Palette;
use crate::style::with_font;
use crate::styled_text::{Inline, Piece, explicit, layered};
use crate::styles::{
    ControlState, InTextStyleScope, Paints, ShapeStyle, TextStyle, Theme, resolve,
};
use crate::symbol::{Image, Symbol};
use crate::views::list::ListRow;

macro_rules! forward_element {
    ($outer:ident) => {
        $crate::elements::forward_element!($outer, $crate::layout::resolve_spacers);
    };
    ($outer:ident, custom_from) => {
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
    ($outer:ident, $resolve:path) => {
        impl From<$outer> for ::freya_core::prelude::Element {
            fn from(value: $outer) -> Self {
                $resolve(value.0.into())
            }
        }

        $crate::elements::forward_element!($outer, custom_from);
    };
}

macro_rules! forward_container {
    ($outer:ident) => {
        $crate::elements::forward_container!($outer, $crate::layout::resolve_spacers);
    };
    ($outer:ident, $resolve:path) => {
        $crate::elements::forward_element!($outer, $resolve);

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

        impl ::freya_core::prelude::AccessibilityExt for $outer {
            fn accessibility_data_mut(&mut self) -> &mut ::freya_core::prelude::AccessibilityData {
                &mut self.overrides.accessibility
            }
        }

        impl ::freya_core::prelude::ContainerExt for $outer {}

        impl ::freya_core::prelude::BackgroundExt for $outer {}

        impl ::freya_core::prelude::DecorationExt for $outer {}
    };
}

macro_rules! forward_key {
    ($outer:ident) => {
        impl ::freya_core::prelude::KeyExt for $outer {
            fn key_mut(&mut self) -> &mut ::freya_core::prelude::DiffKey {
                &mut self.key
            }
        }
    };
    () => {
        fn render_key(&self) -> ::freya_core::prelude::DiffKey {
            self.key.clone().or(self.default_key())
        }
    };
}

pub(crate) use forward_container;
pub(crate) use forward_element;
pub(crate) use forward_key;
pub(crate) use forward_overrides;

#[derive(Default, Clone, PartialEq)]
pub(crate) struct Overrides {
    pub(crate) layout: LayoutData,
    pub(crate) style: StyleState,
    pub(crate) effect: EffectData,
    pub(crate) handlers: EventHandlers,
    pub(crate) accessibility: AccessibilityData,
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
        E: LayoutExt + StyleExt + EffectExt + AccessibilityExt + IntoElement,
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
        crate::accessibility::merge(
            &mut root.accessibility_data_mut().builder,
            &self.accessibility.builder,
        );
        if self.handlers.is_empty() {
            root.into_element()
        } else {
            rect()
                .event_handlers(self.handlers.clone())
                .child(root)
                .into_element()
        }
    }

    pub(crate) fn delegate<C>(&self, mut inner: C) -> C
    where
        C: LayoutExt + StyleExt + EffectExt + EventHandlersExt + AccessibilityExt,
    {
        crate::accessibility::merge(
            &mut inner.accessibility_data_mut().builder,
            &self.accessibility.builder,
        );
        inner
            .layout(self.layout.clone())
            .style(self.style.clone())
            .effect(self.effect.clone())
            .event_handlers(self.handlers.clone())
    }
}

pub struct Text(freya_core::prelude::Label, Cow<'static, str>, Theme, Inline);

impl Text {
    pub fn new(content: impl Into<Cow<'static, str>>) -> Self {
        Self::styled(content.into(), Theme::tengen())
    }

    /// Reads inline Markdown (`**bold**`, `*italic*`, `~~strike~~`, `` `code` ``, `[title](url)`); `new` shows its string as it is.
    pub fn markdown(source: &str) -> Self {
        let (content, runs) = crate::rich_text::markdown(source);
        let mut text = Self::new(content.clone());
        if !runs.is_empty() {
            text.3.pieces = styled_spans(
                &content,
                &runs,
                &Paints::current(ControlState::default()),
                Palette::current().accent,
            );
            for (span, _) in text.3.pieces.iter_mut().filter(|(_, link)| link.is_some()) {
                span.text_style_data.text_decoration = None;
            }
        }
        text
    }

    pub fn style(self, restyle: impl Fn(TextStyle) -> TextStyle + 'static) -> Self {
        let inline = self.3;
        let mut text = Self::styled(self.1, self.2.style(restyle));
        text.3.pieces = inline.pieces;
        text.3.selectable = inline.selectable;
        text
    }

    pub fn font(mut self, font: Font) -> Self {
        let font = font.scaled(crate::theme::resolved().text_size);
        self.3.line_height = font.line_height as f32 / font.size as f32;
        Self(with_font(self.0, font), self.1, self.2, self.3)
    }

    pub fn line_limit(mut self, lines: usize) -> Self {
        self.3.max_lines = Some(lines);
        Self(self.0.max_lines(lines), self.1, self.2, self.3)
    }

    pub fn bold(self) -> Self {
        self.font_weight(FontWeight::BOLD)
    }

    pub fn italic(self) -> Self {
        self.font_slant(FontSlant::Italic)
    }

    pub fn underline(self) -> Self {
        self.text_decoration(TextDecoration::Underline)
    }

    pub fn strikethrough(self) -> Self {
        self.text_decoration(TextDecoration::LineThrough)
    }

    /// Lets the reader select the text with the pointer and copy it; a link of a selectable text does not open.
    pub fn text_selection(mut self, enabled: bool) -> Self {
        self.3.selectable = enabled;
        self
    }

    fn styled(content: Cow<'static, str>, theme: Theme) -> Self {
        let style: TextStyle = resolve(&theme);
        let paints = Paints::current(ControlState::default());
        let inline = Inline {
            line_height: style.font.line_height as f32 / style.font.size as f32,
            ..Inline::default()
        };
        let view = with_font(label().text(content.clone()), style.font)
            .max_lines(None)
            .color(paints.fill(&style.foreground));
        Self(view, content, theme, inline)
    }

    // The runs of this text with what its author set on it, ready to sit beside the runs of another.
    fn runs(mut self) -> Vec<Piece> {
        let mut default = Self::new("");
        let own = explicit(
            self.0.text_style_data_mut(),
            default.0.text_style_data_mut(),
        );
        if self.3.pieces.is_empty() {
            let mut span = Span::new(self.1.into_owned());
            span.text_style_data = own;
            return vec![(span, None)];
        }
        let mut pieces = self.3.pieces;
        for (span, _) in &mut pieces {
            span.text_style_data = layered(&own, &span.text_style_data);
        }
        pieces
    }
}

impl std::ops::Add for Text {
    type Output = Text;

    fn add(self, other: Text) -> Text {
        let mut joined = Text::new(format!("{}{}", self.1, other.1));
        joined.3.selectable = self.3.selectable || other.3.selectable;
        joined.3.pieces = self.runs().into_iter().chain(other.runs()).collect();
        joined
    }
}

impl From<Text> for Element {
    fn from(mut value: Text) -> Self {
        if value.3.is_plain() {
            return crate::layout::resolve_mirrored(value.0.into());
        }
        crate::styled_text::element(&mut value.0, &value.1, value.3)
    }
}

forward_element!(Text, custom_from);

impl TextOverflowExt for Text {}

pub struct Label(Rect);

impl Label {
    pub fn new(title: impl Into<Cow<'static, str>>, image: Symbol) -> Self {
        let style: TextStyle = resolve(&Theme::tengen());
        let paints = Paints::current(ControlState::default());
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

forward_container!(Label, crate::layout::resolve_mirrored);

// What a view is to its neighbours in a column: SwiftUI spaces a text, a symbol, a label, a checkbox and any other
// control by their own preferred distances.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Neighbor {
    Text,
    Image,
    Label,
    Checkbox,
    Control,
}

impl Neighbor {
    fn of<C: 'static>() -> Self {
        let id = TypeId::of::<C>();
        if id == TypeId::of::<Text>() {
            Self::Text
        } else if id == TypeId::of::<Image>() {
            Self::Image
        } else if id == TypeId::of::<Label>() {
            Self::Label
        } else if id == TypeId::of::<Checkbox>() {
            Self::Checkbox
        } else {
            Self::Control
        }
    }
}

pub(crate) fn column_gap(above: Neighbor, below: Neighbor) -> f32 {
    STACK_COLUMN_GAPS[above as usize * 5 + below as usize]
}

// The rect, the kind of the last child, where the gaps sit among the children and whether the gaps adapt.
pub struct VStack(Rect, Option<Neighbor>, Vec<usize>, bool);

impl VStack {
    pub fn new() -> Self {
        Self(
            stack(Direction::Vertical).spacing(0.0),
            None,
            Vec::new(),
            true,
        )
    }

    pub fn child<C: IntoElement + 'static>(self, child: C) -> Self {
        self.push(Neighbor::of::<C>(), child.into_element())
    }

    pub fn maybe_child<C: IntoElement + 'static>(self, child: Option<C>) -> Self {
        match child {
            Some(child) => self.child(child),
            None => self,
        }
    }

    pub fn children<C: IntoElement>(self, children: impl IntoIterator<Item = C>) -> Self {
        children.into_iter().fold(self, |column, child| {
            column.push(Neighbor::Control, child.into_element())
        })
    }

    // A spacing set by hand replaces the distances of the neighbours.
    pub fn spacing(mut self, spacing: f32) -> Self {
        for index in self.2.drain(..).rev() {
            self.0.children_mut().remove(index);
        }
        self.3 = false;
        self.0 = self.0.spacing(spacing);
        self
    }

    fn push(mut self, kind: Neighbor, child: Element) -> Self {
        if let Some(above) = self.1.filter(|_| self.3) {
            let gap = column_gap(above, kind);
            if gap > 0.0 {
                self.2.push(self.0.children_mut().len());
                self.0
                    .children_mut()
                    .push(rect().height(Size::px(gap)).into_element());
            }
        }
        self.0.children_mut().push(child);
        self.1 = Some(kind);
        self
    }
}

impl Default for VStack {
    fn default() -> Self {
        Self::new()
    }
}

forward_container!(VStack, crate::layout::resolve_mirrored);

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

forward_container!(HStack, crate::layout::resolve_mirrored);

fn stack(direction: Direction) -> Rect {
    rect()
        .direction(direction)
        .main_align(Alignment::Center)
        .cross_align(Alignment::Center)
        .spacing(SPACE_2)
}

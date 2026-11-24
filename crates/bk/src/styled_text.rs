use freya_components::selectable_text::SelectableText;
use freya_core::integration::EventHandlers;
use freya_core::prelude::*;

use crate::elements::forward_key;
use crate::text_editor::char_under;

/// A run of equal style and the link it opens, if any.
pub(crate) type Piece = (Span<'static>, Option<String>);

#[derive(Clone, Default, PartialEq)]
pub(crate) struct Inline {
    pub(crate) pieces: Vec<Piece>,
    pub(crate) selectable: bool,
    pub(crate) line_height: f32,
    pub(crate) max_lines: Option<usize>,
}

impl Inline {
    pub(crate) fn is_plain(&self) -> bool {
        self.pieces.is_empty() && !self.selectable
    }
}

// The fields that differ from the default text: what the author set on this Text and the theme did not.
pub(crate) fn explicit(own: &TextStyleData, default: &TextStyleData) -> TextStyleData {
    fn set<T: Clone + PartialEq + Default>(own: &T, default: &T) -> T {
        if own == default {
            T::default()
        } else {
            own.clone()
        }
    }
    TextStyleData {
        color: set(&own.color, &default.color),
        font_size: set(&own.font_size, &default.font_size),
        font_families: set(&own.font_families, &default.font_families),
        text_decoration: set(&own.text_decoration, &default.text_decoration),
        font_slant: set(&own.font_slant, &default.font_slant),
        font_weight: set(&own.font_weight, &default.font_weight),
        font_width: set(&own.font_width, &default.font_width),
        letter_spacing: set(&own.letter_spacing, &default.letter_spacing),
        text_shadows: set(&own.text_shadows, &default.text_shadows),
        ..TextStyleData::default()
    }
}

// `over` where it is set, else `under`.
pub(crate) fn layered(under: &TextStyleData, over: &TextStyleData) -> TextStyleData {
    fn pick<T: Clone + Default + PartialEq>(under: &T, over: &T) -> T {
        if *over == T::default() {
            under.clone()
        } else {
            over.clone()
        }
    }
    TextStyleData {
        color: pick(&under.color, &over.color),
        font_size: pick(&under.font_size, &over.font_size),
        font_families: pick(&under.font_families, &over.font_families),
        text_decoration: pick(&under.text_decoration, &over.text_decoration),
        font_slant: pick(&under.font_slant, &over.font_slant),
        font_weight: pick(&under.font_weight, &over.font_weight),
        font_width: pick(&under.font_width, &over.font_width),
        letter_spacing: pick(&under.letter_spacing, &over.letter_spacing),
        text_shadows: pick(&under.text_shadows, &over.text_shadows),
        ..under.clone()
    }
}

pub(crate) fn link_in(lengths: &[(usize, Option<String>)], mut index: usize) -> Option<&str> {
    for (length, link) in lengths {
        if index < *length {
            return link.as_deref();
        }
        index -= length;
    }
    None
}

pub(crate) fn element(label: &mut Label, content: &str, mut inline: Inline) -> Element {
    if inline.pieces.is_empty() {
        inline.pieces.push((Span::new(content.to_owned()), None));
    }
    StyledText {
        base: label.text_style_data_mut().clone(),
        layout: label.layout_mut().clone(),
        effect: EffectExt::effect_mut(label).clone(),
        accessibility: label.accessibility_data_mut().clone(),
        handlers: label.event_handlers_mut().clone(),
        layer: *label.layer_mut(),
        key: label.key_mut().clone(),
        inline,
    }
    .into()
}

#[derive(Clone, PartialEq)]
struct StyledText {
    base: TextStyleData,
    layout: LayoutData,
    effect: EffectData,
    accessibility: AccessibilityData,
    handlers: EventHandlers,
    layer: Layer,
    key: DiffKey,
    inline: Inline,
}

impl Component for StyledText {
    forward_key!();

    fn render(&self) -> impl IntoElement {
        let holder = use_state(ParagraphHolder::default);
        let mut over_link = use_state(|| false);
        let spans = self.inline.pieces.iter().map(|(span, _)| span.clone());
        if self.inline.selectable {
            let text = spans.fold(SelectableText::new(), |text, span| text.span(span));
            return rect()
                .effect(self.effect.clone())
                .child(
                    text.layout(self.layout.clone())
                        .accessibility(self.accessibility.clone())
                        .text_style(self.base.clone())
                        .event_handlers(self.handlers.clone())
                        .layer(self.layer)
                        .max_lines(self.inline.max_lines)
                        .line_height(self.inline.line_height),
                )
                .into_element();
        }
        let lengths: Vec<(usize, Option<String>)> = self
            .inline
            .pieces
            .iter()
            .map(|(span, link)| (span.text.chars().count(), link.clone()))
            .collect();
        let text: String = self
            .inline
            .pieces
            .iter()
            .map(|(span, _)| span.text.as_ref())
            .collect();
        let linked = lengths.iter().any(|(_, link)| link.is_some());
        let link_at = {
            let lengths = lengths.clone();
            move |holder: &ParagraphHolder, location| {
                let index = char_under(holder, &text, location)?;
                link_in(&lengths, index).map(str::to_owned)
            }
        };
        let hovered = link_at.clone();
        paragraph()
            .layout(self.layout.clone())
            .effect(self.effect.clone())
            .accessibility(self.accessibility.clone())
            .text_style(self.base.clone())
            .event_handlers(self.handlers.clone())
            .layer(self.layer)
            .max_lines(self.inline.max_lines)
            .line_height(self.inline.line_height)
            .holder(holder.read().clone())
            .spans_iter(spans)
            .maybe(linked, |paragraph| {
                paragraph
                    .cursor(if *over_link.read() {
                        CursorIcon::Pointer
                    } else {
                        CursorIcon::Default
                    })
                    .on_pointer_move(move |event: Event<PointerEventData>| {
                        let over = hovered(&holder.peek(), event.element_location()).is_some();
                        if over != *over_link.peek() {
                            over_link.set(over);
                        }
                    })
                    .on_pointer_down(move |event: Event<PointerEventData>| {
                        if let Some(link) = link_at(&holder.peek(), event.element_location()) {
                            spawn(async move { crate::portal::open_uri(&link, false).await });
                        }
                    })
            })
            .into_element()
    }
}

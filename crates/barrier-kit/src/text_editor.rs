use freya_components::cursor_blink::use_cursor_blink;
use freya_core::prelude::*;
use freya_edit::{EditableConfig, EditableEvent, EditorLine, TextEditor as _, use_editable};
use freya_engine::prelude::{RectHeightStyle, RectWidthStyle};
use torin::prelude::{Alignment, Area, Content, CursorPoint, Direction, Gaps, Size};
use vow_design::{BODY, CAPTION, SPACE_1, SPACE_2, TEXT_FIELD};

use crate::compact::IconButton;
use crate::controls::{Button, ControlSize, SearchField};
use crate::font_panel::show_font_panel;
use crate::i18n::text;
use crate::rich_text::{
    Emphasis, FontChoice, StyleRun, apply_font, char_styles, font_at, link_at, restyle_after_edit,
    toggle,
};
use crate::style::text;
use crate::symbol::Symbol;

use crate::context_menu::{Spelling, edit_menu};
use crate::controls::text_style;
use crate::interaction::ring;
use crate::scroll::{ScrollView, use_scroll_controller};
use crate::spelling::{Squiggles, char_index, dictionary, known, misspelled, utf16_range};
use crate::style::Palette;
use crate::theme::{color, resolved};

#[derive(PartialEq)]
pub struct TextEditor {
    value: State<String>,
    placeholder: &'static str,
    height: Size,
    styles: Option<State<Vec<StyleRun>>>,
}

impl TextEditor {
    pub fn new(value: State<String>, placeholder: &'static str) -> Self {
        Self {
            value,
            placeholder,
            height: Size::fill(),
            styles: None,
        }
    }

    pub fn styles(mut self, styles: State<Vec<StyleRun>>) -> Self {
        self.styles = Some(styles);
        self
    }

    pub fn height(mut self, height: Size) -> Self {
        self.height = height;
        self
    }
}

fn char_under(holder: &ParagraphHolder, text: &str, location: CursorPoint) -> Option<usize> {
    let holder = holder.0.borrow();
    let inner = holder.as_ref()?;
    let scale = inner.scale_factor;
    let (x, y) = ((location.x * scale) as f32, (location.y * scale) as f32);
    let caret = inner
        .paragraph
        .get_glyph_position_at_coordinate((x, y))
        .position
        .max(0) as usize;
    [caret, caret.saturating_sub(1)]
        .into_iter()
        .find_map(|utf16| {
            let index = char_index(text, utf16);
            let range = utf16_range(text, &(index..index + 1));
            inner
                .paragraph
                .get_rects_for_range(range, RectHeightStyle::Max, RectWidthStyle::Tight)
                .iter()
                .any(|found| {
                    let rect = found.rect;
                    (rect.left..rect.right).contains(&x) && (rect.top..rect.bottom).contains(&y)
                })
                .then_some(index)
        })
}

fn link_under(
    holder: &ParagraphHolder,
    text: &str,
    styles: Option<State<Vec<StyleRun>>>,
    location: CursorPoint,
) -> Option<String> {
    let index = char_under(holder, text, location)?;
    link_at(&styles?.peek(), index).map(str::to_owned)
}

impl Component for TextEditor {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let a11y_id = use_hook(AccessibilityId::new_unique);
        let focused = use_focus(a11y_id).read().is_focused();
        let holder = use_state(ParagraphHolder::default);
        let mut area = use_state(Area::default);
        let mut hovering = use_state(|| false);
        let mut dragging = use_state(|| false);
        let mut value = self.value;
        let styles = self.styles;
        let mut editable = use_editable(|| value.read().to_string(), EditableConfig::new);
        let (mut blink, cursor) = use_cursor_blink(focused, color(palette.label));
        let mut finding = use_state(|| false);
        let mut history = use_state(|| (false, false));
        let mut spell_index = use_state(|| None);
        let mut over_link = use_state(|| false);
        let query = use_state(String::new);
        let current = use_state(|| 0usize);
        let scroll = use_scroll_controller();
        use_side_effect(move || {
            let found = find_matches(&value.peek(), &query.read());
            let index = (*current.read()).min(found.len().saturating_sub(1));
            if *finding.peek()
                && let Some(&range) = found.get(index)
            {
                let mut editor = editable.editor_mut().write();
                let range = (
                    editor.char_to_utf16_cu(range.0),
                    editor.char_to_utf16_cu(range.1),
                );
                editor.set_selection(range);
                if let Some(inner) = holder.peek().0.borrow().as_ref() {
                    let scale = inner.scale_factor as f32;
                    let margin_y = (TEXT_FIELD.height - BODY.line_height as f32) / 2.0;
                    if let Some(rect) = inner
                        .paragraph
                        .get_rects_for_range(
                            range.0..range.1,
                            RectHeightStyle::Max,
                            RectWidthStyle::Tight,
                        )
                        .first()
                    {
                        scroll.reveal_y(
                            rect.rect.top / scale + margin_y,
                            rect.rect.bottom / scale + margin_y,
                        );
                    }
                }
            }
        });

        if *value.read() != editable.editor().read().committed_text() {
            let mut editor = editable.editor_mut().write();
            editor.clear_preedit();
            editor.set(&value.read());
            // freya-edit 0.4.3 set() checks the UTF-16 cursor against a byte length: https://docs.rs/crate/freya-edit/0.4.3/source/src/rope_editor.rs#273
            let end = editor.len_utf16_cu();
            if editor.cursor_pos() > end {
                editor.move_cursor_to(end);
            }
            editor.editor_history().clear();
            editor.clear_selection();
            history.set_if_modified((false, false));
        }

        let on_key_down = move |event: Event<KeyboardEventData>| {
            if event.key == Key::Character("f".into())
                && event.modifiers.contains(Modifiers::CONTROL)
            {
                finding.set(true);
                return;
            }
            if event.key == Key::Named(NamedKey::Escape) {
                a11y_id.request_unfocus();
                return;
            }
            if let (Some(mut styles), true) = (
                styles,
                event.key == Key::Character("t".into())
                    && event.modifiers.contains(Modifiers::CONTROL),
            ) {
                let editor = editable.editor().peek();
                let start = editor
                    .get_selection()
                    .map(|(from, to)| editor.utf16_cu_to_char(from.min(to)))
                    .unwrap_or_else(|| editor.utf16_cu_to_char(editor.cursor_pos()));
                let initial = font_at(&styles.peek(), editor.rope().len_chars(), start);
                show_font_panel(
                    *area.peek(),
                    initial,
                    Callback::new(move |font: FontChoice| {
                        let editor = editable.editor().peek();
                        let (from, to) = editor
                            .get_selection()
                            .map(|(from, to)| (from.min(to), from.max(to)))
                            .unwrap_or((editor.cursor_pos(), editor.cursor_pos()));
                        let range = editor.utf16_cu_to_char(from)..editor.utf16_cu_to_char(to);
                        let length = editor.rope().len_chars();
                        let next = apply_font(&styles.peek(), length, range, &font);
                        styles.set(next);
                    }),
                );
                return;
            }
            let emphasis = match &event.key {
                Key::Character(c) if c == "b" => Some(Emphasis::Bold),
                Key::Character(c) if c == "i" => Some(Emphasis::Italic),
                _ => None,
            };
            if let (Some(mut styles), Some(emphasis), true) = (
                styles,
                emphasis,
                event.modifiers.contains(Modifiers::CONTROL),
            ) {
                let editor = editable.editor().peek();
                if let Some((from, to)) = editor.get_selection() {
                    let (from, to) = (editor.utf16_cu_to_char(from), editor.utf16_cu_to_char(to));
                    let length = editor.rope().len_chars();
                    let next = toggle(&styles.peek(), length, from.min(to)..from.max(to), emphasis);
                    styles.set(next);
                }
                return;
            }
            blink.reset();
            let redo = Key::Character("y".into());
            let key = match &event.key {
                Key::Character(c)
                    if c.eq_ignore_ascii_case("z")
                        && event.modifiers.contains(Modifiers::SHIFT) =>
                {
                    &redo
                }
                key => key,
            };
            editable.process_event(EditableEvent::KeyDown {
                key,
                modifiers: event.modifiers,
            });
            let mut editor = editable.editor_mut().write();
            let undoable = editor.editor_history();
            history.set_if_modified((undoable.can_undo(), undoable.can_redo()));
            drop(editor);
            let text = editable.editor().read().committed_text();
            if *value.peek() != text {
                if let Some(mut styles) = styles {
                    let next = restyle_after_edit(&styles.peek(), &value.peek(), &text);
                    styles.set(next);
                }
                value.set(text);
                spell_index.set_if_modified(None);
            }
        };
        let on_press = move |event: Event<PointerEventData>| {
            let location = event.element_location();
            match event.data() {
                PointerEventData::Mouse(mouse) if mouse.button == Some(MouseButton::Right) => {
                    if let Some(inner) = holder.peek().0.borrow().as_ref() {
                        let scale = inner.scale_factor;
                        let position = inner
                            .paragraph
                            .get_glyph_position_at_coordinate((
                                (location.x * scale) as f32,
                                (location.y * scale) as f32,
                            ))
                            .position;
                        spell_index.set(Some(char_index(&value.peek(), position.max(0) as usize)));
                    }
                    return;
                }
                PointerEventData::Mouse(mouse) if mouse.button != Some(MouseButton::Left) => return,
                _ => {}
            }
            event.stop_propagation();
            event.prevent_default();
            spell_index.set_if_modified(None);
            if let Some(link) = link_under(&holder.peek(), &value.peek(), styles, location) {
                spawn(async move { crate::portal::open_uri(&link, false).await });
                return;
            }
            dragging.set_if_modified(true);
            blink.reset();
            if !value.peek().is_empty() {
                editable.process_event(EditableEvent::Down {
                    location,
                    editor_line: EditorLine::SingleParagraph,
                    holder: &holder.read(),
                });
            }
            a11y_id.request_focus();
        };
        let on_drag = move |event: Event<PointerEventData>| {
            if focused && *dragging.read() {
                let origin = area.read().origin;
                let mut location = event.global_location();
                location.x -= f64::from(origin.x);
                location.y -= f64::from(origin.y);
                editable.process_event(EditableEvent::Move {
                    location,
                    editor_line: EditorLine::SingleParagraph,
                    holder: &holder.read(),
                });
            }
        };
        let on_release = move |_: Event<PointerEventData>| {
            editable.process_event(EditableEvent::Release);
            if *dragging.peek() {
                dragging.set(false);
            } else if focused && !*hovering.peek() {
                a11y_id.request_unfocus();
            }
        };

        let empty = value.read().is_empty() && !editable.editor().read().has_preedit();
        let editor = editable.editor().read();
        let margin_y = (TEXT_FIELD.height - BODY.line_height as f32) / 2.0;
        let paragraph = paragraph()
            .holder(holder.read().clone())
            .width(Size::fill())
            .margin(Gaps::new_symmetric(margin_y, TEXT_FIELD.padding_x))
            .on_sized(move |event: Event<SizedEventData>| area.set(event.visible_area))
            .on_pointer_down(on_press)
            .on_pointer_move(move |event: Event<PointerEventData>| {
                let over = link_under(
                    &holder.peek(),
                    &value.peek(),
                    styles,
                    event.element_location(),
                )
                .is_some();
                if over != *over_link.peek() {
                    over_link.set(over);
                    Cursor::set(if over {
                        CursorIcon::Pointer
                    } else {
                        CursorIcon::Text
                    });
                }
            })
            .cursor_index(focused.then(|| editor.cursor_pos()))
            .cursor_color(cursor)
            .highlights(
                (focused || *finding.read())
                    .then(|| editor.get_visible_selection(EditorLine::SingleParagraph))
                    .flatten()
                    .map(|selection| vec![selection]),
            )
            .color(color(if empty {
                palette.tertiary_label
            } else {
                palette.label
            }))
            .spans_iter(if empty {
                vec![Span::new(self.placeholder.to_owned())].into_iter()
            } else {
                styled_spans(
                    &editor.rope().to_string(),
                    &self
                        .styles
                        .map(|styles| styles.read().clone())
                        .unwrap_or_default(),
                    palette.accent,
                )
                .into_iter()
            });
        let ranges = dictionary()
            .filter(|_| !empty)
            .map(|dictionary| {
                let text = value.read();
                misspelled(&text, |word| known(dictionary, word))
                    .iter()
                    .map(|range| utf16_range(&text, range))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let squiggles = (!ranges.is_empty()).then(|| {
            crate::glass::leaf(Squiggles {
                holder: holder.read().clone(),
                input: None,
                text_left: Default::default(),
                ranges,
                color: palette.destructive,
                offset: (TEXT_FIELD.padding_x, margin_y),
                layout: crate::glass::layout(
                    Size::fill(),
                    Size::fill(),
                    torin::prelude::Position::new_absolute(),
                ),
            })
        });
        let has_selection = editor.get_selection().is_some();
        let field = edit_menu(
            text_style(rect()),
            a11y_id,
            has_selection,
            *history.read(),
            Some(Spelling {
                value,
                index: spell_index,
                styles,
            }),
        )
        .a11y_id(a11y_id)
        .a11y_focusable(true)
        .a11y_role(AccessibilityRole::MultilineTextInput)
        .a11y_alt(if empty {
            self.placeholder.to_owned()
        } else {
            value.read().clone()
        })
        .width(Size::fill())
        .height(self.height.clone())
        .corner_radius(TEXT_FIELD.radius)
        .background(color(palette.content_background))
        .border(
            Border::new()
                .width(vow_design::HAIRLINE)
                .alignment(BorderAlignment::Inner)
                .fill(color(palette.separator)),
        )
        .on_key_down(on_key_down)
        .on_key_up(move |event: Event<KeyboardEventData>| {
            editable.process_event(EditableEvent::KeyUp { key: &event.key });
        })
        .on_ime_preedit(move |event: Event<ImePreeditEventData>| {
            let mut editor = editable.editor_mut().write();
            if event.data().text.is_empty() {
                editor.clear_preedit();
            } else {
                editor.set_preedit(&event.data().text);
            }
        })
        .on_pointer_enter(move |_| {
            hovering.set(true);
            Cursor::set(CursorIcon::Text);
        })
        .on_pointer_leave(move |_| {
            hovering.set(false);
            Cursor::set(CursorIcon::default());
        })
        .on_pointer_press(move |_| a11y_id.request_focus())
        .on_global_pointer_move(on_drag)
        .on_global_pointer_press(on_release)
        .child(
            ScrollView::new().controller(scroll).expanded().child(
                rect()
                    .width(Size::fill())
                    .child(paragraph)
                    .maybe_child(squiggles),
            ),
        );
        let field = if focused {
            ring(field, TEXT_FIELD.radius, resolved().accent)
        } else {
            field
        };
        if !*finding.read() {
            return field;
        }
        let total = find_matches(&value.read(), &query.read()).len();
        let shown = (*current.read()).min(total.saturating_sub(1));
        let status = match total {
            _ if query.read().is_empty() => String::new(),
            0 => text!("find-none").to_owned(),
            _ => text!(
                "find-count",
                current = (shown + 1).to_string(),
                total = total.to_string()
            ),
        };
        let step = move |delta: isize| {
            let mut current = current;
            let total = find_matches(&value.peek(), &query.peek()).len();
            if total > 0 {
                let next = (*current.peek() as isize + delta).rem_euclid(total as isize);
                current.set(next as usize);
            }
        };
        let bar = rect()
            .width(Size::fill())
            .direction(Direction::Horizontal)
            .content(Content::flex())
            .cross_align(Alignment::Center)
            .spacing(SPACE_2)
            .child(
                rect().width(Size::flex(1.0)).child(
                    SearchField::new(query, text!("search"))
                        .auto_focus(true)
                        .on_navigate(move |key| {
                            step(if key == NamedKey::ArrowUp { -1 } else { 1 })
                        }),
                ),
            )
            .child(text(status, CAPTION, palette.secondary_label))
            .child(
                IconButton::new(Symbol::ChevronUp, text!("find-previous"))
                    .disabled(total == 0)
                    .on_press(move |_| step(-1)),
            )
            .child(
                IconButton::new(Symbol::ChevronDown, text!("find-next"))
                    .disabled(total == 0)
                    .on_press(move |_| step(1)),
            )
            .child(
                Button::new(text!("find-done"))
                    .size(ControlSize::Small)
                    .on_press(move |_| {
                        finding.set(false);
                        a11y_id.request_focus();
                    }),
            );
        rect()
            .width(Size::fill())
            .height(self.height.clone())
            .spacing(SPACE_1)
            .child(bar)
            .child(field.height(Size::flex(1.0)))
            .content(Content::flex())
    }
}

fn styled_spans(text: &str, runs: &[StyleRun], link: vow_design::Rgba) -> Vec<Span<'static>> {
    let chars: Vec<char> = text.chars().collect();
    let styles = char_styles(runs, chars.len());
    let mut spans = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        let style = &styles[start];
        let end = (start..chars.len())
            .find(|&index| styles[index] != *style)
            .unwrap_or(chars.len());
        let mut span = Span::new(chars[start..end].iter().collect::<String>());
        if style.bold {
            span = span.font_weight(FontWeight::BOLD);
        }
        if style.italic {
            span = span.font_slant(FontSlant::Italic);
        }
        if let Some(family) = &style.family {
            span = span.font_family(family.clone());
        }
        if let Some(size) = style.size {
            span = span.font_size(size);
        }
        if style.link.is_some() {
            span = span
                .color(color(link))
                .text_decoration(TextDecoration::Underline);
        }
        spans.push(span);
        start = end;
    }
    spans
}

pub fn find_matches(text: &str, query: &str) -> Vec<(usize, usize)> {
    let fold = |c: char| c.to_lowercase().next().unwrap_or(c);
    let text: Vec<char> = text.chars().map(fold).collect();
    let query: Vec<char> = query.chars().map(fold).collect();
    let mut found = Vec::new();
    let mut start = 0;
    while !query.is_empty() && start + query.len() <= text.len() {
        if text[start..start + query.len()] == query[..] {
            found.push((start, start + query.len()));
            start += query.len();
        } else {
            start += 1;
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::find_matches;

    #[test]
    fn find_ignores_case_counts_characters_and_does_not_overlap() {
        assert_eq!(find_matches("Ñandú y ñandú", "ÑAN"), [(0, 3), (8, 11)]);
        assert_eq!(find_matches("aaaa", "aa"), [(0, 2), (2, 4)]);
        assert!(find_matches("hola", "").is_empty());
    }
}

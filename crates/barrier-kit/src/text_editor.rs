use freya_components::cursor_blink::use_cursor_blink;
use freya_core::prelude::*;
use freya_edit::{EditableConfig, EditableEvent, EditorLine, TextEditor as _, use_editable};
use freya_engine::prelude::{RectHeightStyle, RectWidthStyle};
use torin::prelude::{Alignment, Area, Content, Direction, Gaps, Size};
use vow_design::{CAPTION, SPACE_1, SPACE_2, TEXT, TEXT_FIELD};

use crate::compact::IconButton;
use crate::controls::{Button, SearchField};
use crate::i18n::text;
use crate::style::text;
use crate::symbol::Symbol;

use crate::context_menu::edit_menu;
use crate::controls::text_style;
use crate::interaction::ring;
use crate::scroll::{ScrollView, use_scroll_controller};
use crate::style::Palette;
use crate::theme::{color, resolved};

#[derive(PartialEq)]
pub struct TextEditor {
    value: State<String>,
    placeholder: &'static str,
    height: Size,
}

impl TextEditor {
    pub fn new(value: State<String>, placeholder: &'static str) -> Self {
        Self {
            value,
            placeholder,
            height: Size::fill(),
        }
    }

    pub fn height(mut self, height: Size) -> Self {
        self.height = height;
        self
    }
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
        let mut editable = use_editable(|| value.read().to_string(), EditableConfig::new);
        let (mut blink, cursor) = use_cursor_blink(focused, color(palette.label));
        let mut finding = use_state(|| false);
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
                    let margin_y = (TEXT_FIELD.height - TEXT.line_height as f32) / 2.0;
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
            editor.editor_history().clear();
            editor.clear_selection();
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
            blink.reset();
            editable.process_event(EditableEvent::KeyDown {
                key: &event.key,
                modifiers: event.modifiers,
            });
            let text = editable.editor().read().committed_text();
            if *value.peek() != text {
                value.set(text);
            }
        };
        let on_press = move |event: Event<FocusPressEventData>| {
            event.stop_propagation();
            event.prevent_default();
            dragging.set_if_modified(true);
            blink.reset();
            if !value.peek().is_empty() {
                editable.process_event(EditableEvent::Down {
                    location: event.element_location(),
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
        let margin_y = (TEXT_FIELD.height - TEXT.line_height as f32) / 2.0;
        let paragraph = paragraph()
            .holder(holder.read().clone())
            .width(Size::fill())
            .margin(Gaps::new_symmetric(margin_y, TEXT_FIELD.padding_x))
            .on_sized(move |event: Event<SizedEventData>| area.set(event.visible_area))
            .on_focus_press(on_press)
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
            .span(if empty {
                self.placeholder.to_owned()
            } else {
                editor.rope().to_string()
            });
        let has_selection = editor.get_selection().is_some();
        let field = edit_menu(text_style(rect()), a11y_id, has_selection)
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
                ScrollView::new()
                    .controller(scroll)
                    .width(Size::fill())
                    .height(Size::fill())
                    .child(paragraph),
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
                    .small(true)
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

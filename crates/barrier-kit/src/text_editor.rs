use freya_components::cursor_blink::use_cursor_blink;
use freya_core::prelude::*;
use freya_edit::{EditableConfig, EditableEvent, EditorLine, TextEditor as _, use_editable};
use torin::prelude::{Area, Gaps, Size};
use vow_design::{TEXT, TEXT_FIELD};

use crate::context_menu::edit_menu;
use crate::controls::text_style;
use crate::interaction::ring;
use crate::scroll::ScrollView;
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

        if *value.read() != editable.editor().read().committed_text() {
            let mut editor = editable.editor_mut().write();
            editor.clear_preedit();
            editor.set(&value.read());
            editor.editor_history().clear();
            editor.clear_selection();
        }

        let on_key_down = move |event: Event<KeyboardEventData>| {
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
                focused
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
        let field = edit_menu(text_style(rect()), a11y_id)
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
                    .width(Size::fill())
                    .height(Size::fill())
                    .child(paragraph),
            );
        if focused {
            ring(field, TEXT_FIELD.radius, resolved().accent)
        } else {
            field
        }
    }
}

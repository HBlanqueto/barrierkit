use freya_core::prelude::*;
use freya_engine::prelude::{FontStyle, Slant};
use torin::prelude::{Area, Direction, Gaps, Size};
use vow_design::{BUTTON, CAPTION, DISCLOSURE, FONT_PANEL, HAIRLINE, SPACE_1, SPACE_2, TEXT};

use crate::controls::Button;
use crate::fonts::{family_names, styles};
use crate::i18n::text;
use crate::popover::show_popover;
use crate::rich_text::FontChoice;
use crate::scroll::ScrollView;
use crate::style::{Palette, text};
use crate::theme::color;
use crate::views::tree::{TreeNode, row};

const FACES: [(bool, bool); 4] = [(false, false), (false, true), (true, false), (true, true)];

pub fn faces(styles: &[FontStyle]) -> Vec<(bool, bool)> {
    FACES
        .into_iter()
        .filter(|&(bold, italic)| {
            !bold && !italic
                || styles.iter().any(|style| {
                    (*style.weight() >= 600) == bold && (style.slant() != Slant::Upright) == italic
                })
        })
        .collect()
}

fn face_name(face: (bool, bool)) -> &'static str {
    match face {
        (false, false) => text!("typeface-regular"),
        (false, true) => text!("typeface-italic"),
        (true, false) => text!("typeface-bold"),
        (true, true) => text!("typeface-bold-italic"),
    }
}

pub(crate) fn panel_size() -> (f32, f32) {
    (
        DISCLOSURE.column_width * 2.0 + FONT_PANEL.size_column,
        DISCLOSURE.row_height * 10.0,
    )
}

pub(crate) fn show_font_panel(
    anchor: Area,
    initial: FontChoice,
    on_change: Callback<FontChoice, ()>,
) {
    show_popover(anchor, panel_size(), move || {
        FontPanel {
            initial: initial.clone(),
            on_change: on_change.clone(),
        }
        .into_element()
    });
}

#[derive(PartialEq)]
struct FontPanel {
    initial: FontChoice,
    on_change: Callback<FontChoice, ()>,
}

impl Component for FontPanel {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let initial = self.initial.clone();
        let mut font = use_state(|| initial);
        let families = use_hook(family_names);
        let current = font.read().clone();
        let on_change = self.on_change.clone();
        let set = move |next: FontChoice| {
            font.set(next.clone());
            on_change.call(next);
        };
        let available = faces(&styles(&current.family));

        let family_rows = families.iter().map(|family| {
            let chosen = *family == current.family;
            let mut set = set.clone();
            let next = FontChoice {
                family: family.clone(),
                ..current.clone()
            };
            row(&TreeNode::new(family.clone()), chosen, None, None)
                .on_press(move |_| {
                    let faces = faces(&styles(&next.family));
                    let (bold, italic) = if faces.contains(&(next.bold, next.italic)) {
                        (next.bold, next.italic)
                    } else {
                        (false, false)
                    };
                    set(FontChoice {
                        bold,
                        italic,
                        ..next.clone()
                    });
                })
                .into_element()
        });
        let face_rows = available.into_iter().map(|face| {
            let chosen = face == (current.bold, current.italic);
            let mut set = set.clone();
            let next = FontChoice {
                bold: face.0,
                italic: face.1,
                ..current.clone()
            };
            row(&TreeNode::new(face_name(face)), chosen, None, None)
                .on_press(move |_| set(next.clone()))
                .into_element()
        });
        let size_rows = FONT_PANEL.sizes.into_iter().map(|size| {
            let chosen = size == current.size;
            let mut set = set.clone();
            let next = FontChoice {
                size,
                ..current.clone()
            };
            row(&TreeNode::new(size.to_string()), chosen, None, None)
                .on_press(move |_| set(next.clone()))
                .into_element()
        });
        let column = |label: &'static str, width: Size, rows: Vec<Element>| {
            rect()
                .a11y_role(AccessibilityRole::List)
                .a11y_alt(label)
                .width(width)
                .height(Size::fill())
                .direction(Direction::Vertical)
                .content(torin::prelude::Content::flex())
                .spacing(SPACE_1)
                .child(text(label, CAPTION, palette.secondary_label))
                .child(
                    ScrollView::new()
                        .width(Size::fill())
                        .height(Size::flex(1.0))
                        .children(rows),
                )
        };
        let mut preview = text(
            format!("{} {}", current.family, current.size),
            TEXT,
            palette.label,
        )
        .font_family(current.family.clone())
        .font_size(current.size)
        .max_lines(1);
        if current.bold {
            preview = preview.font_weight(FontWeight::BOLD);
        }
        if current.italic {
            preview = preview.font_slant(FontSlant::Italic);
        }
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .direction(Direction::Vertical)
            .spacing(SPACE_2)
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::px(DISCLOSURE.row_height * 2.0))
                    .corner_radius(BUTTON.radius)
                    .border(
                        Border::new()
                            .width(HAIRLINE)
                            .alignment(BorderAlignment::Inner)
                            .fill(color(palette.card_border)),
                    )
                    .overflow(Overflow::Clip)
                    .center()
                    .child(preview),
            )
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::flex(1.0))
                    .direction(Direction::Horizontal)
                    .spacing(SPACE_1)
                    .padding(Gaps::new(0.0, SPACE_1, 0.0, SPACE_1))
                    .content(torin::prelude::Content::flex())
                    .child(column(
                        text!("font-family"),
                        Size::flex(1.0),
                        family_rows.collect(),
                    ))
                    .child(column(
                        text!("font-typeface"),
                        Size::flex(1.0),
                        face_rows.collect(),
                    ))
                    .child(column(
                        text!("font-size"),
                        Size::px(FONT_PANEL.size_column),
                        size_rows.collect(),
                    )),
            )
    }
}

#[derive(PartialEq)]
pub struct FontPicker {
    font: FontChoice,
    on_change: Option<Callback<FontChoice, ()>>,
}

impl FontPicker {
    pub fn new(font: FontChoice) -> Self {
        Self {
            font,
            on_change: None,
        }
    }

    pub fn on_change(mut self, callback: impl Into<Callback<FontChoice, ()>>) -> Self {
        self.on_change = Some(callback.into());
        self
    }
}

impl Component for FontPicker {
    fn render(&self) -> impl IntoElement {
        let mut area = use_state(Area::default);
        let font = self.font.clone();
        let on_change = self.on_change.clone();
        rect()
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .child(
                Button::new(format!("{} {}", font.family, font.size)).on_press(move |_| {
                    if let Some(on_change) = &on_change {
                        show_font_panel(*area.peek(), font.clone(), on_change.clone());
                    }
                }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use freya_engine::prelude::{Weight, Width};

    #[test]
    fn regular_is_always_offered_and_other_faces_only_when_the_family_has_them() {
        let bold = FontStyle::new(Weight::BOLD, Width::NORMAL, Slant::Upright);
        assert_eq!(faces(&[]), [(false, false)]);
        assert_eq!(faces(&[bold]), [(false, false), (true, false)]);
    }
}

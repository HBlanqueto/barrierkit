use ctdesign::{BODY, BUTTON, CAPTION, DISCLOSURE, FONT_PANEL, HAIRLINE, SPACE_1, SPACE_2};
use freya_core::prelude::*;
use freya_engine::prelude::{FontStyle, Slant};
use torin::prelude::{Area, Direction, Gaps, Size};

use crate::controls::{Button, TextField};
use crate::fonts::{family_names, fixed_pitch, styles};
use crate::i18n::text;
use crate::popover::Popover;
use crate::rich_text::FontChoice;
use crate::scroll::ScrollView;
use crate::style::{Palette, text};
use crate::styles::{ButtonStyle, Theme};
use crate::theme::color;
use crate::views::tree::{TreeNode, row};

thread_local! {
    static RECENT: State<Vec<String>> = State::create_global(Vec::new());
}

#[derive(Clone, Copy, PartialEq)]
enum Collection {
    All,
    Recent,
    FixedWidth,
}

const COLLECTIONS: [Collection; 3] = [Collection::All, Collection::Recent, Collection::FixedWidth];

fn collection_name(collection: Collection) -> &'static str {
    match collection {
        Collection::All => text!("font-collection-all"),
        Collection::Recent => text!("font-collection-recent"),
        Collection::FixedWidth => text!("font-collection-fixed"),
    }
}

pub fn remember(recent: &mut Vec<String>, family: &str) {
    recent.retain(|name| name != family);
    recent.insert(0, family.to_owned());
}

pub fn parse_size(typed: &str) -> Option<f32> {
    typed
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|size| size.is_finite() && *size > 0.0)
}

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
        DISCLOSURE.column_width * 3.0 + FONT_PANEL.size_column,
        DISCLOSURE.row_height * 10.0,
    )
}

pub(crate) fn show_font_panel(
    anchor: Area,
    initial: FontChoice,
    on_change: Callback<FontChoice, ()>,
) {
    Popover::new(panel_size(), move || {
        FontPanel {
            initial: initial.clone(),
            on_change: on_change.clone(),
        }
        .into_element()
    })
    .show(anchor);
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
        let fixed = use_hook(|| {
            families
                .iter()
                .filter(|family| fixed_pitch(family))
                .cloned()
                .collect::<Vec<_>>()
        });
        let mut collection = use_state(|| Collection::All);
        let recent = RECENT.with(|recent| *recent);
        let current = font.read().clone();
        let on_change = self.on_change.clone();
        let set = move |next: FontChoice| {
            let mut recent = recent;
            remember(&mut recent.write(), &next.family);
            font.set(next.clone());
            on_change.call(next);
        };
        let mut typed = use_state(|| current.size.to_string());
        let mut apply_typed = set.clone();
        use_side_effect(move || {
            if let Some(size) = parse_size(&typed.read())
                && size != font.peek().size
            {
                apply_typed(FontChoice {
                    size,
                    ..font.peek().clone()
                });
            }
        });
        let available = faces(&styles(&current.family));
        let shown: Vec<String> = match *collection.read() {
            Collection::All => families.clone(),
            Collection::Recent => recent.read().clone(),
            Collection::FixedWidth => fixed.clone(),
        };

        let collection_rows = COLLECTIONS.into_iter().map(|choice| {
            row(
                &TreeNode::new(collection_name(choice)),
                choice == *collection.read(),
                None,
                None,
            )
            .on_press(move |_| collection.set(choice))
            .into_element()
        });
        let family_rows = shown.iter().map(|family| {
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
                .on_press(move |_| {
                    typed.set(size.to_string());
                    set(next.clone());
                })
                .into_element()
        });
        let column =
            |label: &'static str, width: Size, top: Option<Element>, rows: Vec<Element>| {
                rect()
                    .a11y_role(AccessibilityRole::List)
                    .a11y_alt(label)
                    .width(width)
                    .height(Size::fill())
                    .direction(Direction::Vertical)
                    .content(torin::prelude::Content::flex())
                    .spacing(SPACE_1)
                    .child(text(label, CAPTION, palette.secondary_label))
                    .maybe_child(top)
                    .child(
                        ScrollView::new()
                            .width(Size::fill())
                            .height(Size::flex(1.0))
                            .children(rows),
                    )
            };
        let mut preview = text(
            format!("{} {}", current.family, current.size),
            BODY,
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
            .expanded()
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
                        text!("font-collection"),
                        Size::flex(1.0),
                        None,
                        collection_rows.collect(),
                    ))
                    .child(column(
                        text!("font-family"),
                        Size::flex(1.0),
                        None,
                        family_rows.collect(),
                    ))
                    .child(column(
                        text!("font-typeface"),
                        Size::flex(1.0),
                        None,
                        face_rows.collect(),
                    ))
                    .child(column(
                        text!("font-size"),
                        Size::px(FONT_PANEL.size_column),
                        Some(TextField::new(typed, text!("font-size")).into_element()),
                        size_rows.collect(),
                    )),
            )
    }
}

#[derive(PartialEq)]
pub struct FontPicker {
    font: FontChoice,
    on_change: Option<Callback<FontChoice, ()>>,
    style: Theme,
}

impl FontPicker {
    pub fn new(font: FontChoice) -> Self {
        Self {
            font,
            on_change: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(ButtonStyle) -> ButtonStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
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
        let mut button = Button::new(format!("{} {}", font.family, font.size));
        button.style = self.style.clone();
        rect()
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .child(button.on_press(move |_| {
                if let Some(on_change) = &on_change {
                    show_font_panel(*area.peek(), font.clone(), on_change.clone());
                }
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use freya_engine::prelude::{Weight, Width};

    #[test]
    fn a_used_family_moves_to_the_front_of_recent_without_duplicates() {
        let mut recent = vec!["Inter".to_string(), "Mono".to_string()];
        remember(&mut recent, "Mono");
        assert_eq!(recent, ["Mono", "Inter"]);
    }

    #[test]
    fn only_a_positive_finite_number_is_a_size() {
        assert_eq!(parse_size(" 15.5 "), Some(15.5));
        assert_eq!(parse_size("0"), None);
        assert_eq!(parse_size("doce"), None);
    }

    #[test]
    fn regular_is_always_offered_and_other_faces_only_when_the_family_has_them() {
        let bold = FontStyle::new(Weight::BOLD, Width::NORMAL, Slant::Upright);
        assert_eq!(faces(&[]), [(false, false)]);
        assert_eq!(faces(&[bold]), [(false, false), (true, false)]);
    }
}

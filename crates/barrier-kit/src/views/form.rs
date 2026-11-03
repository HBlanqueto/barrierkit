//! Grouped form: sections of rows on a Tengen list-group card.

use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Size};
use vow_design::{CAPTION, LIST_GROUP, Rgba, SPACE_2, SPACE_3, SPACE_4, TEXT};

use crate::interaction::use_interaction;
use crate::style::{Palette, text};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

#[derive(Clone, PartialEq)]
enum Accessory {
    None,
    Control(Element),
    Value(String),
    Navigation(Callback<(), ()>),
    Destructive(Callback<(), ()>),
}

#[derive(Clone, PartialEq)]
pub struct Row {
    pub title: &'static str,
    pub keywords: &'static [&'static str],
    detail: Option<&'static str>,
    accessory: Accessory,
}

impl Row {
    pub fn new(title: &'static str) -> Self {
        Self {
            title,
            keywords: &[],
            detail: None,
            accessory: Accessory::None,
        }
    }

    pub fn value(title: &'static str, value: impl Into<String>) -> Self {
        Self::new(title).with(Accessory::Value(value.into()))
    }

    pub fn navigation(title: &'static str, on_press: impl Into<Callback<(), ()>>) -> Self {
        Self::new(title).with(Accessory::Navigation(on_press.into()))
    }

    pub fn destructive(title: &'static str, on_press: impl Into<Callback<(), ()>>) -> Self {
        Self::new(title).with(Accessory::Destructive(on_press.into()))
    }

    pub fn control(self, control: impl IntoElement) -> Self {
        self.with(Accessory::Control(control.into_element()))
    }

    pub fn detail(mut self, detail: &'static str) -> Self {
        self.detail = Some(detail);
        self
    }

    pub fn keywords(mut self, keywords: &'static [&'static str]) -> Self {
        self.keywords = keywords;
        self
    }

    fn with(mut self, accessory: Accessory) -> Self {
        self.accessory = accessory;
        self
    }
}

#[derive(Clone, PartialEq)]
pub struct Section {
    pub title: Option<&'static str>,
    pub rows: Vec<Row>,
    footer: Option<&'static str>,
}

impl Section {
    pub fn new() -> Self {
        Self {
            title: None,
            rows: Vec::new(),
            footer: None,
        }
    }

    pub fn titled(title: &'static str) -> Self {
        Self {
            title: Some(title),
            ..Self::new()
        }
    }

    pub fn row(mut self, row: Row) -> Self {
        self.rows.push(row);
        self
    }

    pub fn footer(mut self, footer: &'static str) -> Self {
        self.footer = Some(footer);
        self
    }
}

impl Default for Section {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, PartialEq, Default)]
pub struct Form {
    pub sections: Vec<Section>,
    highlighted: Option<&'static str>,
}

impl Form {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn section(mut self, section: Section) -> Self {
        self.sections.push(section);
        self
    }

    pub fn highlighted(mut self, row_title: Option<&'static str>) -> Self {
        self.highlighted = row_title;
        self
    }
}

impl Component for Form {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let mut width = use_state(|| f32::MAX);
        let stacked = *width.read() < LIST_GROUP.stack_below;
        let [top, right, bottom, left] = LIST_GROUP.header_padding;
        let sections = self.sections.iter().map(|section| {
            let rows = section.rows.iter().enumerate().map(|(index, row)| {
                FormRow {
                    row: row.clone(),
                    first: index == 0,
                    highlighted: self.highlighted == Some(row.title),
                    stacked,
                }
                .into_element()
            });
            rect()
                .width(Size::fill())
                .direction(Direction::Vertical)
                .maybe_child(section.title.map(|title| {
                    rect()
                        .padding(Gaps::new(top, right, bottom, left))
                        .child(text(title.to_uppercase(), CAPTION, palette.secondary_label))
                }))
                .child(
                    rect()
                        .width(Size::fill())
                        .direction(Direction::Vertical)
                        .corner_radius(LIST_GROUP.radius)
                        .overflow(Overflow::Clip)
                        .background(color(palette.content_background))
                        .border(
                            Border::new()
                                .width(1.0)
                                .alignment(BorderAlignment::Inner)
                                .fill(color(palette.card_border)),
                        )
                        .children(rows),
                )
                .maybe_child(section.footer.map(|footer| {
                    rect()
                        .padding(Gaps::new(bottom, right, 0.0, left))
                        .child(text(footer, CAPTION, palette.secondary_label))
                }))
                .into_element()
        });
        rect()
            .width(Size::fill())
            .padding(Gaps::new(
                SPACE_2,
                LIST_GROUP.row_padding_x,
                SPACE_2,
                LIST_GROUP.row_padding_x,
            ))
            .child(
                rect()
                    .width(Size::fill())
                    .direction(Direction::Vertical)
                    .spacing(SPACE_4)
                    .on_sized(move |event: Event<SizedEventData>| {
                        width.set_if_modified(event.area.size.width);
                    })
                    .children(sections),
            )
    }
}

#[derive(PartialEq)]
struct FormRow {
    row: Row,
    first: bool,
    highlighted: bool,
    stacked: bool,
}

impl Component for FormRow {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let row = &self.row;
        let pressable = match &row.accessory {
            Accessory::Navigation(cb) | Accessory::Destructive(cb) => Some(cb.clone()),
            _ => None,
        };
        let title_ink = match row.accessory {
            Accessory::Destructive(_) => palette.destructive,
            _ => palette.label,
        };
        let accessory: Option<Element> = match &row.accessory {
            Accessory::None | Accessory::Destructive(_) => None,
            Accessory::Control(element) => Some(element.clone()),
            Accessory::Value(value) => {
                Some(text(value.clone(), TEXT, palette.secondary_label).into_element())
            }
            Accessory::Navigation(_) => Some(
                symbol(
                    Symbol::ChevronRight,
                    CAPTION.size as f32,
                    palette.tertiary_label,
                )
                .into_element(),
            ),
        };

        let first_line = LIST_GROUP.row_min_height - 2.0 * SPACE_2;
        let aligned_to_first_line = matches!(row.accessory, Accessory::Control(_));
        let stacked = self.stacked && aligned_to_first_line;
        let wrapping = |content: &'static str, face, ink| {
            text(content, face, ink).width(Size::fill()).max_lines(None)
        };
        let mut labels = rect()
            .width(if stacked {
                Size::fill()
            } else {
                Size::flex(1.0)
            })
            .min_width(Size::percent(LIST_GROUP.label_min_share))
            .direction(Direction::Vertical)
            .maybe(aligned_to_first_line, |el| {
                el.padding(Gaps::new(
                    (first_line - TEXT.line_height as f32) / 2.0,
                    0.0,
                    0.0,
                    0.0,
                ))
            })
            .child(wrapping(row.title, TEXT, title_ink));
        if let Some(detail) = row.detail {
            labels = labels.child(wrapping(detail, CAPTION, palette.secondary_label));
        }

        let mut element = rect()
            .width(Size::fill())
            .direction(if stacked {
                Direction::Vertical
            } else {
                Direction::Horizontal
            })
            .content(Content::flex())
            .cross_align(if aligned_to_first_line {
                Alignment::Start
            } else {
                Alignment::Center
            })
            .padding(Gaps::new(
                SPACE_2,
                LIST_GROUP.row_padding_x,
                SPACE_2,
                LIST_GROUP.row_padding_x,
            ))
            .spacing(SPACE_3)
            .child(labels)
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .cross_align(Alignment::Center)
                    .child(rect().width(Size::px(0.0)).height(Size::px(first_line)))
                    .maybe_child(accessory),
            );
        if let Some(on_press) = pressable {
            element = interaction
                .attach(element, true)
                .a11y_role(AccessibilityRole::Button)
                .a11y_alt(row.title)
                .maybe(interaction.hovered(), |el| {
                    el.background(color(palette.control_hover))
                })
                .on_press(move |_| on_press.call(()));
        }
        if self.highlighted {
            element = element.background(color(Rgba {
                a: LIST_GROUP.highlight,
                ..palette.accent
            }));
        }
        let separator = (!self.first).then(|| {
            rect()
                .width(Size::fill())
                .padding(Gaps::new(0.0, 0.0, 0.0, LIST_GROUP.separator_inset))
                .child(
                    rect()
                        .width(Size::fill())
                        .height(Size::px(1.0))
                        .background(color(palette.separator)),
                )
        });
        rect()
            .width(Size::fill())
            .direction(Direction::Vertical)
            .maybe_child(separator)
            .child(element)
    }
}

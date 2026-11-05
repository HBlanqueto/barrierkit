//! Table with sortable columns and virtualised rows.

use freya_components::scrollviews::VirtualScrollView;
use freya_core::prelude::*;
use keyboard_types::{Key, NamedKey};
use torin::prelude::{Alignment, Content, Direction, Gaps, Size};
use vow_design::{CAPTION, Rgba, SPACE_1, TABLE, TEXT};

use crate::interaction::use_interaction;
use crate::style::{Palette, text};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SortOrder {
    pub column: usize,
    pub ascending: bool,
}

#[derive(PartialEq)]
pub struct Table {
    label: &'static str,
    columns: Vec<&'static str>,
    rows: Vec<Vec<String>>,
    sort: Option<SortOrder>,
    selected: Option<usize>,
    on_sort: Option<Callback<SortOrder, ()>>,
    on_select: Option<Callback<usize, ()>>,
}

impl Table {
    pub fn new(label: &'static str, columns: Vec<&'static str>) -> Self {
        Self {
            label,
            columns,
            rows: Vec::new(),
            sort: None,
            selected: None,
            on_sort: None,
            on_select: None,
        }
    }

    pub fn rows(mut self, rows: Vec<Vec<String>>) -> Self {
        self.rows = rows;
        self
    }

    pub fn sort(mut self, sort: Option<SortOrder>) -> Self {
        self.sort = sort;
        self
    }

    pub fn selected(mut self, row: Option<usize>) -> Self {
        self.selected = row;
        self
    }

    pub fn on_sort(mut self, callback: impl Into<Callback<SortOrder, ()>>) -> Self {
        self.on_sort = Some(callback.into());
        self
    }

    pub fn on_select(mut self, callback: impl Into<Callback<usize, ()>>) -> Self {
        self.on_select = Some(callback.into());
        self
    }
}

pub fn next_sort(current: Option<SortOrder>, column: usize) -> SortOrder {
    match current {
        Some(order) if order.column == column => SortOrder {
            column,
            ascending: !order.ascending,
        },
        _ => SortOrder {
            column,
            ascending: true,
        },
    }
}

fn cells(values: impl Iterator<Item = Element>) -> Rect {
    rect()
        .width(Size::fill())
        .direction(Direction::Horizontal)
        .content(Content::flex())
        .cross_align(Alignment::Center)
        .children(values.map(|cell| {
            rect()
                .width(Size::flex(1.0))
                .padding(Gaps::new(0.0, TABLE.padding_x, 0.0, TABLE.padding_x))
                .child(cell)
                .into_element()
        }))
}

impl Component for Table {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let sort = self.sort;
        let interaction = use_interaction();
        let header = cells(self.columns.iter().enumerate().map(|(index, title)| {
            let arrow = match sort {
                Some(order) if order.column == index && order.ascending => Some(Symbol::ChevronUp),
                Some(order) if order.column == index => Some(Symbol::ChevronDown),
                _ => None,
            };
            HeaderCell {
                title,
                arrow,
                on_press: self
                    .on_sort
                    .clone()
                    .map(|on_sort| Callback::new(move |_| on_sort.call(next_sort(sort, index)))),
            }
            .into_element()
        }))
        .height(Size::px(TABLE.header_height))
        .border(
            Border::new()
                .width(vow_design::HAIRLINE)
                .alignment(BorderAlignment::Inner)
                .fill(color(palette.separator)),
        );

        let data = (
            self.rows.clone(),
            self.selected,
            self.on_select.clone(),
            palette,
        );
        let body = VirtualScrollView::new_with_data(
            data,
            |index, (rows, selected, on_select, palette)| {
                let selected_row = *selected == Some(index);
                let (background, ink) = if selected_row {
                    (Some(palette.accent), palette.on_accent)
                } else if index % 2 == 1 {
                    (
                        Some(Rgba {
                            a: TABLE.zebra,
                            ..palette.label
                        }),
                        palette.label,
                    )
                } else {
                    (None, palette.label)
                };
                let on_select = on_select.clone();
                cells(
                    rows[index]
                        .iter()
                        .map(|value| text(value.clone(), TEXT, ink).into_element()),
                )
                .a11y_role(AccessibilityRole::Row)
                .a11y_builder(move |node| node.set_selected(selected_row))
                .height(Size::px(TABLE.row_height))
                .maybe(background.is_some(), |el| {
                    el.background(color(background.unwrap_or(palette.accent)))
                })
                .on_press(move |_| {
                    if let Some(cb) = &on_select {
                        cb.call(index);
                    }
                })
                .into_element()
            },
        )
        .length(self.rows.len())
        .item_size(TABLE.row_height);

        let (rows, selected, on_select) = (self.rows.len(), self.selected, self.on_select.clone());
        let table = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Table)
            .a11y_alt(self.label)
            .width(Size::fill())
            .height(Size::fill())
            .direction(Direction::Vertical)
            .on_key_down(move |event: Event<KeyboardEventData>| {
                let down = match event.key {
                    Key::Named(NamedKey::ArrowDown) => true,
                    Key::Named(NamedKey::ArrowUp) => false,
                    _ => return,
                };
                let next = step_selection(selected, rows, down);
                if next != selected
                    && let (Some(next), Some(cb)) = (next, &on_select)
                {
                    cb.call(next);
                }
            })
            .child(header)
            .child(body);
        interaction.inner_focus_ring(table)
    }
}

fn step_selection(selected: Option<usize>, rows: usize, down: bool) -> Option<usize> {
    let last = rows.checked_sub(1)?;
    Some(match (selected, down) {
        (None, true) => 0,
        (None, false) => last,
        (Some(row), true) => (row + 1).min(last),
        (Some(row), false) => row.saturating_sub(1),
    })
}

#[derive(PartialEq)]
struct HeaderCell {
    title: &'static str,
    arrow: Option<Symbol>,
    on_press: Option<Callback<(), ()>>,
}

impl Component for HeaderCell {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let ink = if self.arrow.is_some() {
            palette.label
        } else {
            palette.secondary_label
        };
        let on_press = self.on_press.clone();
        let cell = interaction
            .attach(rect(), on_press.is_some())
            .a11y_role(AccessibilityRole::ColumnHeader)
            .a11y_alt(self.title)
            .maybe(on_press.is_some(), |el| {
                el.on_press(move |_| {
                    if let Some(cb) = &on_press {
                        cb.call(());
                    }
                })
            })
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(SPACE_1)
            .child(text(self.title, CAPTION, ink))
            .maybe_child(
                self.arrow
                    .map(|arrow| symbol(arrow, CAPTION.size as f32, ink)),
            );
        interaction.inner_focus_ring(cell)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clicking_the_sorted_column_flips_the_order_and_another_column_starts_ascending() {
        let by_name = next_sort(None, 0);
        assert_eq!(
            by_name,
            SortOrder {
                column: 0,
                ascending: true
            }
        );
        assert!(!next_sort(Some(by_name), 0).ascending);
        assert_eq!(
            next_sort(Some(by_name), 2),
            SortOrder {
                column: 2,
                ascending: true
            }
        );
    }

    #[test]
    fn arrows_select_the_first_or_last_row_then_stop_at_the_edges() {
        assert_eq!(step_selection(None, 3, true), Some(0));
        assert_eq!(step_selection(None, 3, false), Some(2));
        assert_eq!(step_selection(Some(2), 3, true), Some(2));
        assert_eq!(step_selection(Some(0), 3, false), Some(0));
        assert_eq!(step_selection(None, 0, true), None);
    }
}

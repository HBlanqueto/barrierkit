//! Table with sortable columns and virtualised rows.

use freya_components::scrollviews::VirtualScrollView;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Size};
use vow_design::{CAPTION, Rgba, SPACE_1, TABLE, TEXT};

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
        let header = cells(self.columns.iter().enumerate().map(|(index, title)| {
            let on_sort = self.on_sort.clone();
            let arrow = match sort {
                Some(order) if order.column == index && order.ascending => Some(Symbol::ChevronUp),
                Some(order) if order.column == index => Some(Symbol::ChevronDown),
                _ => None,
            };
            let ink = if arrow.is_some() {
                palette.label
            } else {
                palette.secondary_label
            };
            rect()
                .a11y_role(AccessibilityRole::ColumnHeader)
                .a11y_alt(*title)
                .on_press(move |_| {
                    if let Some(cb) = &on_sort {
                        cb.call(next_sort(sort, index));
                    }
                })
                .direction(Direction::Horizontal)
                .cross_align(Alignment::Center)
                .spacing(SPACE_1)
                .child(text(*title, CAPTION, ink))
                .maybe_child(arrow.map(|arrow| symbol(arrow, CAPTION.size as f32, ink)))
                .into_element()
        }))
        .height(Size::px(TABLE.header_height))
        .border(
            Border::new()
                .width(1.0)
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

        rect()
            .a11y_role(AccessibilityRole::Table)
            .a11y_alt(self.label)
            .width(Size::fill())
            .height(Size::fill())
            .direction(Direction::Vertical)
            .child(header)
            .child(body)
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
}

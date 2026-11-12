use ctdesign::{Font, Rgba, TABLE};
use freya_components::scrollviews::VirtualScrollView;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};

use crate::context_menu::context_menu;
use crate::interaction::{control_state, use_interaction};
use crate::interaction::{track_modifiers, use_held_modifiers};
use crate::menu::{CheckState, MenuItem};
use crate::style::text;
use crate::styles::Painted;
use crate::styles::{ControlState, Paints, ShapeStyle, StateValue, TableStyle, Theme, resolve};
use crate::symbol::{Symbol, symbol};
use crate::views::collection::select;

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
    selected: Vec<usize>,
    on_sort: Option<Callback<SortOrder, ()>>,
    on_select: Option<Callback<Vec<usize>, ()>>,
    style: Theme,
}

impl Table {
    pub fn new(label: &'static str, columns: Vec<&'static str>) -> Self {
        Self {
            label,
            columns,
            rows: Vec::new(),
            sort: None,
            selected: Vec::new(),
            on_sort: None,
            on_select: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(TableStyle) -> TableStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn rows(mut self, rows: Vec<Vec<String>>) -> Self {
        self.rows = rows;
        self
    }

    pub fn sort(mut self, sort: Option<SortOrder>) -> Self {
        self.sort = sort;
        self
    }

    pub fn selected(mut self, rows: Vec<usize>) -> Self {
        self.selected = rows;
        self
    }

    pub fn on_sort(mut self, callback: impl Into<Callback<SortOrder, ()>>) -> Self {
        self.on_sort = Some(callback.into());
        self
    }

    pub fn on_select(mut self, callback: impl Into<Callback<Vec<usize>, ()>>) -> Self {
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

#[derive(PartialEq)]
struct RowLook {
    background: Fill,
    blur: Option<f32>,
    label: Rgba,
}

fn row_look(
    paints: &Paints,
    background: &StateValue<ShapeStyle>,
    label: &StateValue<ShapeStyle>,
) -> RowLook {
    RowLook {
        background: paints.fill(background),
        blur: paints.blur(background),
        label: paints.color(label.resolve(ControlState::default())),
    }
}

fn cells(padding_x: f32, values: impl Iterator<Item = (Option<f32>, Element)>) -> Rect {
    rect()
        .width(Size::fill())
        .direction(Direction::Horizontal)
        .content(Content::flex())
        .cross_align(Alignment::Center)
        .children(values.map(|(width, cell)| {
            rect()
                .width(width.map_or(Size::flex(1.0), Size::px))
                .padding(Gaps::new(0.0, padding_x, 0.0, padding_x))
                .child(cell)
                .into_element()
        }))
}

pub fn shown_columns(hidden: &[usize], count: usize) -> Vec<usize> {
    (0..count)
        .filter(|column| !hidden.contains(column))
        .collect()
}

pub fn toggle_column(hidden: &[usize], column: usize, count: usize) -> Vec<usize> {
    if hidden.contains(&column) {
        return hidden.iter().copied().filter(|&c| c != column).collect();
    }
    let mut next = hidden.to_vec();
    if shown_columns(hidden, count).len() > 1 {
        next.push(column);
    }
    next
}

impl Component for Table {
    fn render(&self) -> impl IntoElement {
        let style: TableStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default(), style.effect);
        let padding_x = style.padding_x;
        let sort = self.sort;
        let interaction = use_interaction();
        let held = use_held_modifiers();
        let mut hidden = use_state(Vec::<usize>::new);
        let mut widths = use_state(|| vec![None::<f32>; self.columns.len()]);
        let mut measured = use_state(|| vec![0.0f32; self.columns.len()]);
        let mut drag = use_state(|| None::<(usize, f64, f32)>);
        if widths.peek().len() != self.columns.len() {
            widths.write().resize(self.columns.len(), None);
            measured.write().resize(self.columns.len(), 0.0);
        }
        let shown = shown_columns(&hidden.read(), self.columns.len());
        let width_of = move |column: usize| widths.read().get(column).copied().flatten();

        let header_cells = shown.iter().map(|&index| {
            let arrow = match sort {
                Some(order) if order.column == index && order.ascending => Some(Symbol::ChevronUp),
                Some(order) if order.column == index => Some(Symbol::ChevronDown),
                _ => None,
            };
            let cell = HeaderCell {
                title: self.columns[index],
                arrow,
                style: self.style.clone(),
                on_press: self
                    .on_sort
                    .clone()
                    .map(|on_sort| Callback::new(move |_| on_sort.call(next_sort(sort, index)))),
            };
            let handle = rect()
                .position(Position::new_absolute().right(-padding_x - TABLE.resize_handle / 2.0))
                .width(Size::px(TABLE.resize_handle))
                .height(Size::px(style.header_height))
                .cursor(CursorIcon::ColResize)
                .on_pointer_down(move |event: Event<PointerEventData>| {
                    event.stop_propagation();
                    let start = measured.peek()[index];
                    drag.set(Some((index, event.global_location().x, start)));
                });
            let content = rect()
                .width(Size::fill())
                .on_sized(move |event: Event<SizedEventData>| {
                    let width = event.area.width() + 2.0 * padding_x;
                    if measured.peek()[index] != width {
                        measured.write()[index] = width;
                    }
                })
                .child(cell)
                .child(handle);
            (width_of(index), content.into_element())
        });
        let header = cells(padding_x, header_cells)
            .height(Size::px(style.header_height))
            .bordered(&paints, style.header_border.as_ref());
        let count = self.columns.len();
        let menu = self
            .columns
            .iter()
            .enumerate()
            .map(|(index, title)| {
                MenuItem::action(index.to_string(), *title)
                    .with_check(CheckState::Checkbox(!hidden.read().contains(&index)))
            })
            .collect();
        let header = context_menu(header, menu, move |id: String| {
            if let Ok(column) = id.parse::<usize>() {
                let next = toggle_column(&hidden.peek(), column, count);
                hidden.set(next);
            }
        });

        let column_widths: Vec<Option<f32>> = shown.iter().map(|&c| width_of(c)).collect();
        let looks = [
            row_look(&paints, &style.row_background, &style.label),
            row_look(&paints, &style.alternate_row_background, &style.label),
            row_look(
                &paints,
                &style.selected_row_background,
                &style.selected_label,
            ),
        ];
        let metrics = (style.font, style.row_height, padding_x);
        let data = (
            self.rows.clone(),
            self.selected.clone(),
            self.on_select.clone(),
            looks,
            metrics,
            shown.clone(),
            column_widths,
            held,
        );
        let body = VirtualScrollView::new_with_data(
            data,
            |item, (rows, selected, on_select, looks, metrics, shown, column_widths, held)| {
                let index = item.index;
                let (font, row_height, padding_x): &(Font, f32, f32) = metrics;
                let selected_row = selected.contains(&index);
                let look = if selected_row {
                    &looks[2]
                } else {
                    &looks[index % 2]
                };
                let on_select = on_select.clone();
                let selected = selected.clone();
                let held = *held;
                let values = shown.iter().zip(column_widths).map(|(&column, width)| {
                    let value = rows[index].get(column).cloned().unwrap_or_default();
                    let value = rect()
                        .width(Size::fill())
                        .overflow(Overflow::Clip)
                        .child(text(value, *font, look.label));
                    (*width, value.into_element())
                });
                cells(*padding_x, values)
                    .a11y_role(AccessibilityRole::Row)
                    .a11y_builder(move |node| node.set_selected(selected_row))
                    .height(Size::px(*row_height))
                    .background(look.background.clone())
                    .map(look.blur, Rect::backdrop_blur)
                    .on_press(move |_| {
                        if let Some(cb) = &on_select {
                            cb.call(select(&selected, index, *held.peek()));
                        }
                    })
                    .into_element()
            },
        )
        .length(self.rows.len())
        .item_size(style.row_height);

        let (rows, on_select) = (self.rows.len(), self.on_select.clone());
        let selected = self.selected.last().copied();
        let table = track_modifiers(interaction.attach(rect(), true), held)
            .a11y_role(AccessibilityRole::Table)
            .a11y_alt(self.label)
            .expanded()
            .direction(Direction::Vertical)
            .cursor(drag.read().is_some().then_some(CursorIcon::ColResize))
            .on_global_pointer_move(move |event: Event<PointerEventData>| {
                if let Some((column, origin, start)) = *drag.peek() {
                    let width = (start + (event.global_location().x - origin) as f32)
                        .max(TABLE.min_column_width);
                    widths.write()[column] = Some(width);
                }
            })
            .on_global_pointer_up(move |_| {
                interaction.release();
                if drag.peek().is_some() {
                    drag.set(None);
                }
            })
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
                    cb.call(vec![next]);
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
    style: Theme,
}

impl Component for HeaderCell {
    fn render(&self) -> impl IntoElement {
        let style: TableStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state, style.effect);
        let label = if self.arrow.is_some() {
            &style.sorted_header_label
        } else {
            &style.header_label
        };
        let ink = paints.color(label.resolve(state));
        let font = style.header_font;
        let on_press = self.on_press.clone();
        let cell = interaction
            .attach(rect(), on_press.is_some())
            .a11y_role(AccessibilityRole::ColumnHeader)
            .a11y_alt(self.title)
            .map(on_press, |el, on_press| {
                el.on_press(move |_| on_press.call(()))
            })
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(style.header_spacing)
            .child(text(self.title, font, ink))
            .maybe_child(self.arrow.map(|arrow| symbol(arrow, font.size as f32, ink)));
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

    #[test]
    fn hiding_a_column_keeps_at_least_one_shown_and_hiding_it_again_shows_it() {
        assert_eq!(toggle_column(&[], 1, 3), [1]);
        assert_eq!(toggle_column(&[1], 1, 3), Vec::<usize>::new());
        assert_eq!(toggle_column(&[0, 1], 2, 3), [0, 1]);
        assert_eq!(shown_columns(&[0, 2], 3), [1]);
    }
}

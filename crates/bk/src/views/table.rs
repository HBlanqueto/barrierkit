use ctdesign::{Font, HAIRLINE, Rgba, TABLE};
use freya_components::scrollviews::VirtualScrollView;
use freya_core::prelude::*;
use std::borrow::Cow;
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};

use crate::context_menu::context_menu;
use crate::elements::{Overrides, forward_overrides};
use crate::interaction::{control_state, use_interaction};
use crate::interaction::{track_modifiers, use_held_modifiers};
use crate::menu::{CheckState, MenuItem};
use crate::scroll::{use_elastic, use_scroll_bar, use_scroll_controller};
use crate::style::{continuous_corner, text};
use crate::styles::{
    ControlState, Painted, Paints, Role, ScrollViewStyle, ShapeStyle, StateValue, TableStyle,
    TextStyleScope, Theme, resolve,
};
use crate::symbol::{Symbol, stroked_symbol, symbol};
use crate::views::collection::select;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SortOrder {
    pub column: usize,
    pub ascending: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    Automatic,
    Visible,
    Hidden,
}

#[derive(Clone, Default, PartialEq)]
pub struct ListRows {
    insets: Option<Callback<usize, f32>>,
    separator: Option<Callback<usize, Visibility>>,
    background: Option<Callback<usize, ShapeStyle>>,
}

impl ListRows {
    pub(crate) fn shifted(&self, offset: usize) -> Self {
        fn shift<T: 'static>(
            call: &Option<Callback<usize, T>>,
            offset: usize,
        ) -> Option<Callback<usize, T>> {
            call.clone()
                .map(|call| Callback::new(move |row| call.call(row + offset)))
        }
        Self {
            insets: shift(&self.insets, offset),
            separator: shift(&self.separator, offset),
            background: shift(&self.background, offset),
        }
    }
}

pub trait ListRowModifiers: Sized {
    fn list_rows(&mut self) -> &mut ListRows;

    fn list_row_insets(mut self, insets: impl Fn(usize) -> f32 + 'static) -> Self {
        self.list_rows().insets = Some(Callback::new(insets));
        self
    }

    fn list_row_separator(mut self, visibility: impl Fn(usize) -> Visibility + 'static) -> Self {
        self.list_rows().separator = Some(Callback::new(visibility));
        self
    }

    fn list_row_background(mut self, background: impl Fn(usize) -> ShapeStyle + 'static) -> Self {
        self.list_rows().background = Some(Callback::new(background));
        self
    }
}

#[derive(PartialEq)]
pub struct Table {
    label: Cow<'static, str>,
    columns: Vec<Cow<'static, str>>,
    rows: Vec<Vec<String>>,
    cells: Vec<(usize, Callback<usize, Element>)>,
    pub(crate) list_rows: ListRows,
    sort: Option<SortOrder>,
    selected: Vec<usize>,
    icons: Vec<Symbol>,
    scroll_bar: bool,
    on_sort: Option<Callback<SortOrder, ()>>,
    on_select: Option<Callback<Vec<usize>, ()>>,
    style: Theme,
    overrides: Overrides,
}

impl Table {
    pub fn new(
        label: impl Into<Cow<'static, str>>,
        columns: impl IntoIterator<Item = impl Into<Cow<'static, str>>>,
    ) -> Self {
        Self {
            label: label.into(),
            columns: columns.into_iter().map(Into::into).collect(),
            rows: Vec::new(),
            cells: Vec::new(),
            list_rows: ListRows::default(),
            sort: None,
            selected: Vec::new(),
            icons: Vec::new(),
            scroll_bar: true,
            on_sort: None,
            on_select: None,
            style: Theme::tengen(),
            overrides: Overrides::default(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(TableStyle) -> TableStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub(crate) fn themed(mut self, style: Theme) -> Self {
        self.style = style;
        self
    }

    pub fn rows(mut self, rows: Vec<Vec<String>>) -> Self {
        self.rows = rows;
        self
    }

    pub fn cell<V: IntoElement>(
        mut self,
        column: usize,
        cell: impl Fn(usize) -> V + 'static,
    ) -> Self {
        self.cells
            .push((column, Callback::new(move |row| cell(row).into_element())));
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

    pub fn icons(mut self, icons: Vec<Symbol>) -> Self {
        self.icons = icons;
        self
    }

    pub fn scroll_bar(mut self, shown: bool) -> Self {
        self.scroll_bar = shown;
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

impl ListRowModifiers for Table {
    fn list_rows(&mut self) -> &mut ListRows {
        &mut self.list_rows
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
    foreground: ShapeStyle,
}

fn row_look(
    paints: &Paints,
    state: ControlState,
    background: &StateValue<ShapeStyle>,
    label: &StateValue<ShapeStyle>,
) -> RowLook {
    RowLook {
        background: paints.fill(background),
        blur: paints.blur(background),
        label: paints.color(label.resolve(state)),
        foreground: label.resolve(state).clone(),
    }
}

pub(crate) fn row_divider(fill: Fill, before: f32, after: f32) -> Rect {
    rect()
        .layer(1)
        .position(Position::new_absolute().left(0.0).bottom(0.0))
        .width(Size::fill())
        .padding(Gaps::new(0.0, after, 0.0, before))
        .child(
            rect()
                .width(Size::fill())
                .height(Size::px(HAIRLINE))
                .background(fill),
        )
}

fn cells(
    padding_x: f32,
    column_shift: f32,
    values: impl Iterator<Item = (Option<f32>, Element)>,
) -> Rect {
    rect()
        .width(Size::fill())
        .direction(Direction::Horizontal)
        .content(Content::flex())
        .cross_align(Alignment::Center)
        .children(values.enumerate().map(|(position, (width, cell))| {
            let leading = padding_x + if position > 0 { column_shift } else { 0.0 };
            rect()
                .width(width.map_or(Size::flex(1.0), Size::px))
                .padding(Gaps::new(0.0, padding_x, 0.0, leading))
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

forward_overrides!(Table);

impl Component for Table {
    fn render(&self) -> impl IntoElement {
        let style: TableStyle = resolve(&self.style);
        let interaction = use_interaction();
        let focused = ControlState {
            focused: interaction.focused(),
            ..ControlState::default()
        };
        let paints = Paints::current(focused);
        let padding_x = style.padding_x;
        let header_padding_x = style.header_padding_x;
        let column_shift = style.column_shift;
        let sort = self.sort;
        let held = use_held_modifiers();
        let mut hidden = use_state(Vec::<usize>::new);
        let mut widths = use_state(|| vec![None::<f32>; self.columns.len()]);
        let mut measured = use_state(|| vec![0.0f32; self.columns.len()]);
        let mut drag = use_state(|| None::<(usize, f64, f32)>);
        let mut body_height = use_state(|| 0.0f32);
        let clicked = use_state(|| None::<usize>);
        let controller = use_scroll_controller();
        let elastic = use_elastic(controller, None);
        let scroll_style: ScrollViewStyle = resolve(&self.style);
        if widths.peek().len() != self.columns.len() {
            widths.write().resize(self.columns.len(), None);
            measured.write().resize(self.columns.len(), 0.0);
        }
        let shown = shown_columns(&hidden.read(), self.columns.len());
        let width_of = move |column: usize| widths.read().get(column).copied().flatten();

        let header_cells = shown.iter().enumerate().map(|(position, &index)| {
            let arrow = match sort {
                Some(order) if order.column == index && order.ascending => Some(Symbol::ChevronUp),
                Some(order) if order.column == index => Some(Symbol::ChevronDown),
                _ => None,
            };
            let cell = HeaderCell {
                title: self.columns[index].clone(),
                arrow,
                style: self.style.clone(),
                on_press: self
                    .on_sort
                    .clone()
                    .map(|on_sort| Callback::new(move |_| on_sort.call(next_sort(sort, index)))),
            };
            let handle = rect()
                .position(
                    Position::new_absolute().right(-header_padding_x - TABLE.resize_handle / 2.0),
                )
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
                    let width = event.area.width()
                        + 2.0 * header_padding_x
                        + if position > 0 { column_shift } else { 0.0 };
                    if measured.peek()[index] != width {
                        measured.write()[index] = width;
                    }
                })
                .child(cell)
                .child(handle);
            (width_of(index), content.into_element())
        });
        let inset = style.row_inset;
        let bars = use_scroll_bar(controller, false, self.scroll_bar, true, &scroll_style);
        controller.set_content((
            0.0,
            self.rows.len() as f32 * style.row_height + style.body_inset,
        ));
        let header =
            rect()
                .width(Size::fill())
                .height(Size::px(style.header_height))
                .padding(Gaps::new(0.0, bars.reserved, 0.0, 0.0))
                .child(
                    rect()
                        .width(Size::fill())
                        .height(Size::px(style.header_height))
                        .padding(Gaps::new(0.0, inset, 0.0, inset))
                        .child(
                            cells(header_padding_x, column_shift, header_cells)
                                .height(Size::fill()),
                        ),
                )
                .maybe_child(style.header_border.as_ref().map(|(paint, width)| {
                    rect()
                        .position(Position::new_absolute().left(0.0).bottom(1.0 - *width))
                        .width(Size::fill())
                        .height(Size::px(*width))
                        .background(paints.fill(&StateValue::new(paint.clone())))
                }))
                .children(style.header_separator.as_ref().into_iter().flat_map(
                    |(paint, height)| {
                        let edges = std::iter::once(inset)
                            .chain(shown.iter().scan(inset, |edge, &column| {
                                *edge += measured.read()[column];
                                Some(*edge)
                            }))
                            .collect::<Vec<_>>();
                        let top = (style.header_height - height) / 2.0;
                        let height = *height;
                        let fill = paints.fill(&StateValue::new(paint.clone()));
                        edges.into_iter().map(move |edge| {
                            rect()
                                .position(Position::new_absolute().left(edge - HAIRLINE).top(top))
                                .width(Size::px(HAIRLINE))
                                .height(Size::px(height))
                                .background(fill.clone())
                                .into_element()
                        })
                    },
                ));
        let count = self.columns.len();
        let menu = self
            .columns
            .iter()
            .enumerate()
            .map(|(index, title)| {
                MenuItem::action(index.to_string(), title.clone())
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
            row_look(&paints, focused, &style.row_background, &style.label),
            row_look(
                &paints,
                focused,
                &style.alternate_row_background,
                &style.label,
            ),
            row_look(
                &paints,
                focused,
                &style.selected_row_background,
                &style.selected_label,
            ),
        ];
        let stripe = looks[1].background.clone();
        let separator = (
            paints.fill(&StateValue::new(
                style
                    .row_separator
                    .clone()
                    .unwrap_or(ShapeStyle::Role(Role::Separator)),
            )),
            style.separator_margin,
        );
        let separated = style.row_separator.is_some();
        let background = self.list_rows.background.clone().map(|background| {
            let paints = Paints::current(focused);
            Callback::new(move |row| paints.fill(&StateValue::new(background.call(row))))
        });
        let list_rows = (
            self.list_rows.insets.clone(),
            self.list_rows.separator.clone(),
            background,
            separated,
        );
        let metrics = (
            style.font,
            style.row_height,
            padding_x,
            inset,
            style.row_radius,
            column_shift,
        );
        let icons = (
            self.icons.clone(),
            [
                paints.color(style.icon.resolve(focused)),
                paints.color(style.selected_icon.resolve(focused)),
            ],
            style.icon_size,
            style.icon_spacing,
        );
        let data = (
            self.rows.clone(),
            self.cells.clone(),
            self.selected.clone(),
            self.on_select.clone(),
            looks,
            metrics,
            shown.clone(),
            column_widths,
            held,
            separator,
            icons,
            list_rows,
        );
        let striped = !matches!(
            style.alternate_row_background.resolve(focused),
            ShapeStyle::Clear
        );
        let empty_rows = if striped {
            ((*body_height.read() - style.body_inset) / style.row_height).ceil() as usize
        } else {
            0
        };
        let empty_stripes = (self.rows.len()..empty_rows)
            .filter(|index| index % 2 == 1)
            .map(|index| {
                rect()
                    .layer(-1)
                    .position(
                        Position::new_absolute()
                            .left(0.0)
                            .top(index as f32 * style.row_height),
                    )
                    .width(Size::fill())
                    .padding(Gaps::new(0.0, inset, 0.0, inset))
                    .child(
                        rect()
                            .width(Size::fill())
                            .height(Size::px(style.row_height))
                            .corner_radius(continuous_corner(style.row_radius))
                            .background(stripe.clone()),
                    )
            })
            .collect::<Vec<_>>();
        let row_elastic = elastic.clone();
        let body = VirtualScrollView::new_with_data(
            data,
            move |item,
                  (
                rows,
                views,
                selected,
                on_select,
                looks,
                metrics,
                shown,
                column_widths,
                held,
                separator,
                icons,
                (row_insets, row_separator, row_background, separated),
            )| {
                let index = item.index;
                let (font, row_height, padding_x, inset, row_radius, column_shift): &(
                    Font,
                    f32,
                    f32,
                    f32,
                    f32,
                    f32,
                ) = metrics;
                let padding_x = row_insets
                    .as_ref()
                    .map_or(*padding_x, |insets| insets.call(index));
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
                    let (title, name) = match views.iter().find(|(target, _)| *target == column) {
                        Some((_, cell)) => {
                            let cell = cell.clone();
                            let scope = TextStyleScope::new(
                                *font,
                                look.foreground.clone(),
                                Callback::new(move |()| cell.call(index)),
                            );
                            (
                                scope.into_element(),
                                Some(value).filter(|name| !name.is_empty()),
                            )
                        }
                        None => (text(value, *font, look.label).into_element(), None),
                    };
                    let (symbols, inks, size, gap) = icons;
                    let content = match symbols.get(index).filter(|_| column == 0) {
                        Some(&icon) => rect()
                            .direction(Direction::Horizontal)
                            .cross_align(Alignment::Center)
                            .spacing(*gap)
                            .child(
                                rect()
                                    .width(Size::px(TABLE.icon_box))
                                    .main_align(Alignment::Center)
                                    .direction(Direction::Horizontal)
                                    .child(symbol(icon, *size, inks[usize::from(selected_row)])),
                            )
                            .child(title)
                            .into_element(),
                        None => title,
                    };
                    let value = rect()
                        .width(Size::fill())
                        .overflow(Overflow::Clip)
                        .map(name, |cell, name| cell.a11y_alt(name))
                        .child(content);
                    (*width, value.into_element())
                });
                let shown = match row_separator
                    .as_ref()
                    .map(|visibility| visibility.call(index))
                {
                    Some(Visibility::Visible) => true,
                    Some(Visibility::Hidden) => false,
                    _ => *separated,
                };
                let divided = shown
                    && !selected_row
                    && index + 1 < rows.len()
                    && !selected.contains(&(index + 1));
                let row_fill = row_background
                    .as_ref()
                    .filter(|_| !selected_row)
                    .map(|fill| fill.call(index));
                rect()
                    .key(index)
                    .width(Size::fill())
                    .height(Size::px(*row_height))
                    .padding(Gaps::new(0.0, *inset, 0.0, *inset))
                    .on_wheel(row_elastic.wheel(controller, false))
                    .a11y_role(AccessibilityRole::Row)
                    .a11y_builder(move |node| node.set_selected(selected_row))
                    .child(
                        cells(padding_x, *column_shift, values)
                            .height(Size::fill())
                            .corner_radius(continuous_corner(*row_radius))
                            .background(row_fill.unwrap_or_else(|| look.background.clone()))
                            .map(look.blur, Rect::backdrop_blur),
                    )
                    .maybe_child(divided.then(|| {
                        let (fill, (before, after)) = separator;
                        row_divider(fill.clone(), *before, *after)
                    }))
                    .on_press(move |_| {
                        let mut clicked = clicked;
                        clicked.set(Some(index));
                        if let Some(cb) = &on_select {
                            cb.call(select(&selected, index, *held.peek()));
                        }
                    })
                    .into_element()
            },
        )
        .show_scrollbar(false)
        .scroll_controller(controller.raw())
        .length(self.rows.len())
        .item_size(style.row_height);

        let (rows, on_select, selected) = (
            self.rows.len(),
            self.on_select.clone(),
            self.selected.clone(),
        );
        let table = track_modifiers(interaction.attach(rect(), true), held)
            .a11y_role(AccessibilityRole::Table)
            .a11y_alt(&*self.label)
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
                let from = cursor_row(&selected, *clicked.peek());
                let next = step_selection(from, rows, down);
                if next != from
                    && let (Some(next), Some(cb)) = (next, &on_select)
                {
                    cb.call(vec![next]);
                }
            })
            .bordered(&paints, style.border.as_ref())
            .maybe_child((style.header_height > 0.0).then_some(header))
            .child(
                rect()
                    .expanded()
                    .overflow(Overflow::Clip)
                    .on_sized(move |event: Event<SizedEventData>| {
                        body_height.set_if_modified(event.area.height());
                        controller.set_viewport((event.area.width(), event.area.height()));
                    })
                    .maybe_child(bars.track)
                    .maybe_child(bars.thumb)
                    .child(
                        rect()
                            .expanded()
                            .offset_y(elastic.shift())
                            .padding(Gaps::new(style.body_inset, bars.reserved, 0.0, 0.0))
                            .children(empty_stripes)
                            .child(body),
                    ),
            );
        self.overrides.apply(interaction.inner_focus_ring(table))
    }
}

fn cursor_row(selected: &[usize], clicked: Option<usize>) -> Option<usize> {
    clicked
        .filter(|row| selected.contains(row))
        .or(selected.last().copied())
}

fn step_selection(selected: Option<usize>, rows: usize, down: bool) -> Option<usize> {
    let last = rows.checked_sub(1)?;
    Some(match (selected, down) {
        (None, true) => 0,
        (None, false) => last,
        (Some(row), true) => (row + 1).min(last),
        (Some(row), false) => row.saturating_sub(1).min(last),
    })
}

#[derive(PartialEq)]
struct HeaderCell {
    title: Cow<'static, str>,
    arrow: Option<Symbol>,
    on_press: Option<Callback<(), ()>>,
    style: Theme,
}

impl Component for HeaderCell {
    fn render(&self) -> impl IntoElement {
        let style: TableStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state);
        let label = if self.arrow.is_some() {
            &style.sorted_header_label
        } else {
            &style.header_label
        };
        let ink = paints.color(label.resolve(state));
        let font = if self.arrow.is_some() {
            style.sorted_header_font
        } else {
            style.header_font
        };
        let on_press = self.on_press.clone();
        let cell = interaction
            .attach(rect(), on_press.is_some())
            .a11y_role(AccessibilityRole::ColumnHeader)
            .a11y_alt(&*self.title)
            .map(on_press, |el, on_press| {
                el.on_press(move |_| on_press.call(()))
            })
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(style.header_spacing)
            .child(text(self.title.clone(), font, ink))
            .maybe_child(self.arrow.map(|arrow| {
                let (size, stroke) = style.sort_arrow_size;
                let (inset, drop) = style.sort_arrow_offset;
                rect()
                    .position(
                        Position::new_absolute()
                            .right(inset)
                            .top((font.line_height as f32 - size) / 2.0 + drop),
                    )
                    .child(stroked_symbol(
                        arrow,
                        size,
                        stroke,
                        paints.color(style.sort_arrow.resolve(state)),
                    ))
            }));
        interaction.inner_focus_ring(cell)
    }
}

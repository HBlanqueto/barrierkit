use ctdesign::{Font, LABEL_GAP, LABEL_ICON_WIDTH, LIST_SECTION, SUBHEADLINE, TABLE};
use freya_core::prelude::*;
use std::any::TypeId;
use std::borrow::Cow;
use torin::prelude::{Alignment, Direction, Gaps, Size};

use crate::elements::Label;
use crate::elements::{Overrides, forward_overrides};
use crate::scroll::ScrollView;
use crate::style::text;
use crate::styles::{
    ControlState, Paints, Role, ShapeStyle, StateValue, TableStyle, Theme, resolve,
};
use crate::views::table::{ListRowModifiers, ListRows, Table, row_divider};

// Rows below a `List` draw a `Label` as SwiftUI does: the symbol tinted with the accent.
#[derive(Clone, Copy)]
pub(crate) struct ListRow;

#[derive(PartialEq)]
pub struct List {
    label: Cow<'static, str>,
    rows: Vec<String>,
    sections: Vec<(Cow<'static, str>, Vec<String>)>,
    selected: Vec<usize>,
    row: Option<Callback<usize, Element>>,
    list_rows: ListRows,
    label_rows: bool,
    scroll_bar: bool,
    on_select: Option<Callback<Vec<usize>, ()>>,
    style: Theme,
    overrides: Overrides,
}

impl List {
    pub fn new(label: impl Into<Cow<'static, str>>, rows: Vec<String>) -> Self {
        Self {
            label: label.into(),
            rows,
            sections: Vec::new(),
            selected: Vec::new(),
            row: None,
            list_rows: ListRows::default(),
            label_rows: false,
            scroll_bar: true,
            on_select: None,
            style: Theme::tengen().style(|_: TableStyle| TableStyle::inset_list()),
            overrides: Overrides::default(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(TableStyle) -> TableStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn section(mut self, title: impl Into<Cow<'static, str>>, rows: Vec<String>) -> Self {
        self.sections.push((title.into(), rows));
        self
    }

    // Rows that are labels are 24.5 high and their separators start at the title, as in SwiftUI.
    pub fn row<V: IntoElement + 'static>(mut self, row: impl Fn(usize) -> V + 'static) -> Self {
        self.label_rows = TypeId::of::<V>() == TypeId::of::<Label>();
        self.row = Some(Callback::new(move |index| row(index).into_element()));
        self
    }

    pub fn selected(mut self, rows: Vec<usize>) -> Self {
        self.selected = rows;
        self
    }

    pub fn scroll_bar(mut self, shown: bool) -> Self {
        self.scroll_bar = shown;
        self
    }

    pub fn on_select(mut self, callback: impl Into<Callback<Vec<usize>, ()>>) -> Self {
        self.on_select = Some(callback.into());
        self
    }
}

forward_overrides!(List);

impl ListRowModifiers for List {
    fn list_rows(&mut self) -> &mut ListRows {
        &mut self.list_rows
    }
}

impl Component for List {
    fn render(&self) -> impl IntoElement {
        use_hook(|| provide_context(ListRow));
        if !self.sections.is_empty() {
            return self.overrides.delegate(self.sectioned()).into_element();
        }
        let mut table = Table::new(self.label.clone(), vec![""])
            .rows(self.rows.iter().map(|row| vec![row.clone()]).collect())
            .selected(self.selected.clone())
            .scroll_bar(self.scroll_bar)
            .map(self.row.clone(), |table, row| {
                table.cell(0, move |index| row.call(index))
            })
            .map(self.on_select.clone(), |table, callback| {
                table.on_select(callback)
            });
        table.list_rows = self.list_rows.clone();
        self.overrides
            .delegate(table.themed(self.row_theme()))
            .into_element()
    }
}

impl List {
    fn row_theme(&self) -> Theme {
        let label_rows = self.label_rows;
        self.style
            .clone()
            .style(move |style: TableStyle| match label_rows {
                true => TableStyle {
                    row_height: TABLE.label_row_height,
                    separator_margin: (
                        style.padding_x + LABEL_ICON_WIDTH + LABEL_GAP,
                        style.separator_margin.1,
                    ),
                    ..style
                },
                false => style,
            })
    }

    fn sectioned(&self) -> ScrollView {
        let style: TableStyle = resolve(&self.row_theme());
        let paints = Paints::current(ControlState::default());
        let header_ink = paints.color(&ShapeStyle::Role(Role::SecondaryLabel));
        let theme = self.row_theme().style(|style: TableStyle| TableStyle {
            body_inset: 0.0,
            ..style
        });
        let mut first = 0;
        let sections = self
            .sections
            .iter()
            .enumerate()
            .map(|(index, (title, rows))| {
                let offset = first;
                first += rows.len();
                let selected = self
                    .selected
                    .iter()
                    .filter(|row| (offset..first).contains(row))
                    .map(|row| row - offset)
                    .collect();
                let mut table = Table::new(self.label.clone(), vec![""])
                    .rows(rows.iter().map(|row| vec![row.clone()]).collect())
                    .selected(selected)
                    .scroll_bar(false)
                    .map(self.row.clone(), |table, row| {
                        table.cell(0, move |local| row.call(local + offset))
                    })
                    .map(self.on_select.clone(), |table, callback| {
                        table.on_select(move |local: Vec<usize>| {
                            callback.call(local.into_iter().map(|row| row + offset).collect())
                        })
                    });
                table.list_rows = self.list_rows.shifted(offset);
                let top = if index == 0 {
                    style.body_inset
                } else {
                    LIST_SECTION.gap
                };
                rect()
                    .width(Size::fill())
                    .direction(Direction::Vertical)
                    .margin(Gaps::new(top, 0.0, 0.0, 0.0))
                    .child(
                        rect()
                            .width(Size::fill())
                            .height(Size::px(LIST_SECTION.header_height))
                            .padding(Gaps::new(0.0, style.padding_x, 0.0, style.padding_x))
                            .main_align(Alignment::Center)
                            .child(text(
                                title.clone(),
                                Font {
                                    weight: LIST_SECTION.header_weight,
                                    ..SUBHEADLINE
                                },
                                header_ink,
                            ))
                            .child(row_divider(
                                paints.fill(&StateValue::new(ShapeStyle::Role(Role::Separator))),
                                style.separator_margin.0,
                                style.separator_margin.1,
                            )),
                    )
                    .child(
                        rect()
                            .width(Size::fill())
                            .height(Size::px(rows.len() as f32 * style.row_height))
                            .child(table.themed(theme.clone())),
                    )
                    .into_element()
            });
        ScrollView::new()
            .scroll_bar(self.scroll_bar)
            .children(sections)
    }
}

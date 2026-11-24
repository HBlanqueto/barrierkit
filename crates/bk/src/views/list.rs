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
    selection: Option<State<Vec<usize>>>,
    row: Option<Callback<usize, Element>>,
    list_rows: ListRows,
    label_rows: bool,
    scroll_bar: bool,
    style: Theme,
    overrides: Overrides,
}

impl List {
    pub fn new(label: impl Into<Cow<'static, str>>, rows: Vec<String>) -> Self {
        Self {
            label: label.into(),
            rows,
            sections: Vec::new(),
            selection: None,
            row: None,
            list_rows: ListRows::default(),
            label_rows: false,
            scroll_bar: true,
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

    pub fn selection(mut self, selection: State<Vec<usize>>) -> Self {
        self.selection = Some(selection);
        self
    }

    pub fn scroll_bar(mut self, shown: bool) -> Self {
        self.scroll_bar = shown;
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
        let content_height = use_state(|| 0.0);
        let row = self
            .row
            .clone()
            .map(|row| measured_row(row, content_height));
        let theme = self.row_theme(*content_height.read());
        if !self.sections.is_empty() {
            return self
                .overrides
                .delegate(self.sectioned(row, theme))
                .into_element();
        }
        let mut table = Table::new(self.label.clone(), vec![""])
            .rows(self.rows.iter().map(|row| vec![row.clone()]).collect())
            .scroll_bar(self.scroll_bar)
            .map(self.selection, Table::selection)
            .map(row, |table, row| {
                table.cell(0, move |index| row.call(index))
            });
        table.list_rows = self.list_rows.clone();
        self.overrides.delegate(table.themed(theme)).into_element()
    }
}

impl List {
    fn row_theme(&self, content_height: f32) -> Theme {
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
                false => TableStyle {
                    row_height: style
                        .row_height
                        .max(content_height + style.row_height - style.font.line_height as f32),
                    ..style
                },
            })
    }

    fn sectioned(&self, row: Option<Callback<usize, Element>>, row_theme: Theme) -> ScrollView {
        let style: TableStyle = resolve(&row_theme);
        let paints = Paints::current(ControlState::default());
        let header_ink = paints.color(&ShapeStyle::Role(Role::SecondaryLabel));
        let theme = row_theme.style(|style: TableStyle| TableStyle {
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
                let mut table = Table::new(self.label.clone(), vec![""])
                    .rows(rows.iter().map(|row| vec![row.clone()]).collect())
                    .scroll_bar(false)
                    .map(self.selection, Table::selection)
                    .map(row.clone(), |table, row| {
                        table.cell(0, move |local| row.call(local + offset))
                    });
                table.list_rows = self.list_rows.shifted(offset);
                table.offset = offset;
                table.scrolled_outside = true;
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

// The first row's view sets every row's height: a SwiftUI row is its content plus a single line's padding.
fn measured_row(
    row: Callback<usize, Element>,
    mut content_height: State<f32>,
) -> Callback<usize, Element> {
    Callback::new(move |index| {
        rect()
            .width(Size::fill())
            .map((index == 0).then_some(()), |cell, ()| {
                cell.on_sized(move |event: Event<SizedEventData>| {
                    content_height.set_if_modified(event.area.height())
                })
            })
            .child(row.call(index))
            .into_element()
    })
}

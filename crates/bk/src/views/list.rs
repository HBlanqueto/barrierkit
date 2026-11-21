use ctdesign::{LIST_SECTION, SUBHEADLINE};
use freya_core::prelude::*;
use std::borrow::Cow;
use torin::prelude::{Alignment, Direction, Gaps, Size};

use crate::elements::{Overrides, forward_overrides};
use crate::scroll::ScrollView;
use crate::style::text;
use crate::styles::{
    ControlState, Paints, Role, ShapeStyle, StateValue, TableStyle, Theme, resolve,
};
use crate::views::table::{Table, row_divider};

#[derive(PartialEq)]
pub struct List {
    label: Cow<'static, str>,
    rows: Vec<String>,
    sections: Vec<(Cow<'static, str>, Vec<String>)>,
    selected: Vec<usize>,
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

impl Component for List {
    fn render(&self) -> impl IntoElement {
        if !self.sections.is_empty() {
            return self.overrides.delegate(self.sectioned()).into_element();
        }
        let table = Table::new(self.label.clone(), vec![""])
            .rows(self.rows.iter().map(|row| vec![row.clone()]).collect())
            .selected(self.selected.clone())
            .scroll_bar(self.scroll_bar);
        let table = match &self.on_select {
            Some(callback) => table.on_select(callback.clone()),
            None => table,
        };
        self.overrides
            .delegate(table.themed(self.style.clone()))
            .into_element()
    }
}

impl List {
    fn sectioned(&self) -> ScrollView {
        let style: TableStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default(), style.effect);
        let header_ink = paints.color(&ShapeStyle::Role(Role::SecondaryLabel));
        let theme = self.style.clone().style(|style: TableStyle| TableStyle {
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
                let table = Table::new(self.label.clone(), vec![""])
                    .rows(rows.iter().map(|row| vec![row.clone()]).collect())
                    .selected(selected)
                    .scroll_bar(false);
                let table = match &self.on_select {
                    Some(callback) => {
                        let callback = callback.clone();
                        table.on_select(move |local: Vec<usize>| {
                            callback.call(local.into_iter().map(|row| row + offset).collect())
                        })
                    }
                    None => table,
                };
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
                            .child(text(title.clone(), SUBHEADLINE, header_ink).font_weight(600))
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

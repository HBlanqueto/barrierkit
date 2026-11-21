use freya_core::prelude::*;

use crate::styles::{TableStyle, Theme};
use crate::views::table::Table;

#[derive(PartialEq)]
pub struct List {
    label: &'static str,
    rows: Vec<String>,
    selected: Vec<usize>,
    on_select: Option<Callback<Vec<usize>, ()>>,
    style: Theme,
}

impl List {
    pub fn new(label: &'static str, rows: Vec<String>) -> Self {
        Self {
            label,
            rows,
            selected: Vec::new(),
            on_select: None,
            style: Theme::tengen().style(|_: TableStyle| TableStyle::inset_list()),
        }
    }

    pub fn style(mut self, restyle: impl Fn(TableStyle) -> TableStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn selected(mut self, rows: Vec<usize>) -> Self {
        self.selected = rows;
        self
    }

    pub fn on_select(mut self, callback: impl Into<Callback<Vec<usize>, ()>>) -> Self {
        self.on_select = Some(callback.into());
        self
    }
}

impl Component for List {
    fn render(&self) -> impl IntoElement {
        let table = Table::new(self.label, vec![""])
            .rows(self.rows.iter().map(|row| vec![row.clone()]).collect())
            .selected(self.selected.clone());
        let table = match &self.on_select {
            Some(callback) => table.on_select(callback.clone()),
            None => table,
        };
        table.themed(self.style.clone())
    }
}

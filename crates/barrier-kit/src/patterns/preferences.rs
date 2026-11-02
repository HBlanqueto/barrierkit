//! Preferences window: searchable sidebar of panes, history and a form per pane.

use freya_components::scrollviews::ScrollView;
use freya_core::prelude::*;
use serde::{Deserialize, Serialize};
use torin::prelude::{Direction, Gaps, Size};
use vow_design::{LIST_GROUP, Rgba, SPACE_2};

use crate::app::{load_restoration, save_restoration};
use crate::controls::SearchField;
use crate::sidebar::{Sidebar, SidebarAccount, SidebarItem, SidebarSection};
use crate::symbol::Symbol;
use crate::titlebar::Titlebar;
use crate::views::form::Form;
use crate::views::navigation::{History, navigation_buttons};
use crate::views::search::SearchIndex;
use crate::views::split::SplitView;

#[derive(Clone, PartialEq)]
pub struct Pane {
    pub id: &'static str,
    pub title: &'static str,
    symbol: Symbol,
    tint: Rgba,
    form: Form,
}

impl Pane {
    pub fn new(
        id: &'static str,
        title: &'static str,
        symbol: Symbol,
        tint: Rgba,
        form: Form,
    ) -> Self {
        Self {
            id,
            title,
            symbol,
            tint,
            form,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct PaneGroup {
    title: Option<&'static str>,
    panes: Vec<Pane>,
}

impl PaneGroup {
    pub fn new(title: Option<&'static str>) -> Self {
        Self {
            title,
            panes: Vec::new(),
        }
    }

    pub fn pane(mut self, pane: Pane) -> Self {
        self.panes.push(pane);
        self
    }
}

#[derive(PartialEq)]
pub struct PreferencesWindow {
    restoration: Option<&'static str>,
    account: Option<(&'static str, &'static str, &'static str)>,
    groups: Vec<PaneGroup>,
}

impl PreferencesWindow {
    pub fn new() -> Self {
        Self {
            restoration: None,
            account: None,
            groups: Vec::new(),
        }
    }

    pub fn restoration(mut self, app_id: &'static str) -> Self {
        self.restoration = Some(app_id);
        self
    }

    pub fn account(
        mut self,
        initials: &'static str,
        name: &'static str,
        detail: &'static str,
    ) -> Self {
        self.account = Some((initials, name, detail));
        self
    }

    pub fn group(mut self, group: PaneGroup) -> Self {
        self.groups.push(group);
        self
    }

    fn panes(&self) -> impl Iterator<Item = &Pane> {
        self.groups.iter().flat_map(|group| group.panes.iter())
    }
}

impl Default for PreferencesWindow {
    fn default() -> Self {
        Self::new()
    }
}

type Hit = (&'static str, Option<&'static str>);

#[derive(Default, Serialize, Deserialize)]
struct Restored {
    pane: String,
}

pub fn index(panes: &[&Pane]) -> SearchIndex<Hit> {
    let mut index = SearchIndex::new();
    for pane in panes {
        index.add((pane.id, None), pane.title, &[]);
        for row in pane
            .form
            .sections
            .iter()
            .flat_map(|section| section.rows.iter())
        {
            index.add((pane.id, Some(row.title)), row.title, row.keywords);
        }
    }
    index
}

impl Component for PreferencesWindow {
    fn render(&self) -> impl IntoElement {
        let first = self.panes().next().map(|pane| pane.id).unwrap_or_default();
        let restoration = self.restoration;
        let restored = restoration
            .map(load_restoration::<Restored>)
            .and_then(|state| {
                self.panes()
                    .map(|pane| pane.id)
                    .find(|id| *id == state.pane)
            });
        let history = use_state(|| History::new(restored.unwrap_or(first)));
        use_side_effect(move || {
            let pane = history.read().current().to_string();
            if let Some(app_id) = restoration
                && let Err(error) = save_restoration(app_id, &Restored { pane })
            {
                tracing::warn!(%error, "preferencias: no se guardó el panel");
            }
        });
        let query = use_state(String::new);
        let mut highlighted: State<Option<&'static str>> = use_state(|| None);
        let current = *history.read().current();
        let panes: Vec<&Pane> = self.panes().collect();
        let pane = panes.iter().find(|pane| pane.id == current).copied();

        let open = move |id: &'static str, row: Option<&'static str>| {
            let mut history = history;
            Callback::new(move |_| {
                history.write().push(id);
                highlighted.set(row);
            })
        };
        let item = |pane: &Pane| {
            SidebarItem::new(pane.title)
                .symbol(pane.symbol, pane.tint)
                .selected(pane.id == current)
                .on_select(open(pane.id, None))
                .into_element()
        };

        let mut sidebar = Sidebar::new()
            .traffic_lights(true)
            .child(SearchField::new(query, "Buscar"));
        let text = query.read().clone();
        if text.trim().is_empty() {
            if let Some((initials, name, detail)) = self.account {
                sidebar = sidebar.child(SidebarAccount::new(initials, name, detail));
            }
            for group in &self.groups {
                let items = group.panes.iter().map(item);
                sidebar = match group.title {
                    Some(title) => sidebar.child(SidebarSection::new(title).children(items)),
                    None => sidebar.children(items),
                };
            }
        } else {
            let hits = index(&panes).search(&text).into_iter().map(|hit| {
                let (id, row) = hit.key;
                SidebarItem::new(hit.title)
                    .selected(id == current && row == *highlighted.read())
                    .on_select(open(id, row))
                    .into_element()
            });
            sidebar = sidebar.children(hits);
        }

        let detail = rect()
            .width(Size::fill())
            .height(Size::fill())
            .direction(Direction::Vertical)
            .child(
                Titlebar::new()
                    .with_sidebar(true)
                    .title(pane.map(|pane| pane.title).unwrap_or_default())
                    .left(navigation_buttons(history)),
            )
            .child(
                ScrollView::new().child(
                    rect()
                        .width(Size::fill())
                        .padding(Gaps::new(
                            SPACE_2,
                            LIST_GROUP.row_padding_x,
                            SPACE_2,
                            LIST_GROUP.row_padding_x,
                        ))
                        .maybe_child(
                            pane.map(|pane| pane.form.clone().highlighted(*highlighted.read())),
                        ),
                ),
            );

        SplitView::new(sidebar, detail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::form::{Row, Section};

    #[test]
    fn the_index_finds_panes_by_title_and_rows_by_keyword() {
        let dock = Pane::new(
            "dock",
            "Escritorio y Dock",
            Symbol::MagnifyingGlass,
            Rgba::rgb8(0, 0, 0),
            Form::new().section(Section::new().row(Row::new("Tamaño").keywords(&["iconos"]))),
        );
        let index = index(&[&dock]);
        let keys = |q: &str| {
            index
                .search(q)
                .into_iter()
                .map(|hit| hit.key)
                .collect::<Vec<_>>()
        };
        assert_eq!(keys("escritorio"), [("dock", None)]);
        assert_eq!(keys("iconos"), [("dock", Some("Tamaño"))]);
    }
}

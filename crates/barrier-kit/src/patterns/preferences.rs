//! Preferences window: searchable sidebar of panes, history and a form per pane.

use crate::scroll::ScrollView;
use freya_core::prelude::*;
use serde::{Deserialize, Serialize};
use torin::prelude::{Alignment, Direction, Gaps, Size};
use vow_design::{Rgba, SETTINGS, SIDEBAR, SPACE_2};

use crate::app::{load_restoration, save_restoration};
use crate::controls::SearchField;
use crate::sidebar::{Sidebar, SidebarAccount, SidebarItem, SidebarSection};
use crate::symbol::Symbol;
use crate::titlebar::Titlebar;
use crate::views::form::Form;
use crate::views::navigation::{History, navigation_buttons};
use crate::views::search::SearchIndex;
use crate::views::split::{SidebarLayout, SidebarToggle, SplitView};

use super::search_popover::{Entry, search_popover};

const RECENTS: usize = 3;

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
    account: Option<(String, String, String)>,
    groups: Vec<PaneGroup>,
    suggestions: Option<&'static [&'static str]>,
}

impl PreferencesWindow {
    pub fn new() -> Self {
        Self {
            restoration: None,
            account: None,
            groups: Vec::new(),
            suggestions: None,
        }
    }

    pub fn suggestions(mut self, pane_ids: &'static [&'static str]) -> Self {
        self.suggestions = Some(pane_ids);
        self
    }

    pub fn restoration(mut self, app_id: &'static str) -> Self {
        self.restoration = Some(app_id);
        self
    }

    pub fn account(
        mut self,
        initials: impl Into<String>,
        name: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        self.account = Some((initials.into(), name.into(), detail.into()));
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

#[derive(Clone, Default, Serialize, Deserialize)]
struct Restored {
    pane: String,
    #[serde(default)]
    recents: Vec<String>,
    #[serde(default)]
    sidebar: Option<SidebarLayout>,
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
        let saved = use_hook(|| {
            restoration
                .map(load_restoration::<Restored>)
                .unwrap_or_default()
        });
        let known = |id: &String| self.panes().map(|pane| pane.id).find(|known| known == id);
        let history = use_state(|| History::new(known(&saved.pane).unwrap_or(first)));
        let recents: State<Vec<&'static str>> =
            use_state(|| saved.recents.iter().filter_map(known).collect());
        let mut layout = use_state(|| saved.sidebar);
        use_side_effect(move || {
            let state = Restored {
                pane: history.read().current().to_string(),
                recents: recents.read().iter().map(|id| id.to_string()).collect(),
                sidebar: *layout.read(),
            };
            if let Some(app_id) = restoration
                && let Err(error) = save_restoration(app_id, &state)
            {
                tracing::warn!(%error, "preferencias: no se guardó el estado");
            }
        });
        let query = use_state(String::new);
        let search_id = use_a11y();
        let mut search_focused = use_state(|| false);
        let popover_hovered = use_state(|| false);
        let mut dismissed_at: State<Option<String>> = use_state(|| None);
        let mut cursor: State<Option<(String, usize)>> = use_state(|| None);
        let mut highlighted: State<Option<&'static str>> = use_state(|| None);
        let current = *history.read().current();
        let panes: Vec<&Pane> = self.panes().collect();
        let pane = panes.iter().find(|pane| pane.id == current).copied();

        let open = move |id: &'static str, row: Option<&'static str>| {
            let mut history = history;
            let mut recents = recents;
            Callback::new(move |_| {
                history.write().push(id);
                highlighted.set(row);
                dismissed_at.set(Some(query.peek().clone()));
                search_id.request_unfocus();
                let mut recents = recents.write();
                recents.retain(|recent| *recent != id);
                recents.insert(0, id);
                recents.truncate(RECENTS);
            })
        };
        let text = query.read().clone();
        let dismissed = dismissed_at.read().as_ref() == Some(&text);
        let searching = *search_focused.read() && !dismissed;
        let emphasized = !searching;
        let item = |pane: &Pane| {
            SidebarItem::new(pane.title)
                .symbol(pane.symbol, pane.tint)
                .selected(pane.id == current)
                .emphasized(emphasized)
                .on_select(open(pane.id, None))
                .into_element()
        };
        let result = |pane: &Pane, title: &'static str, row: Option<&'static str>| Entry::Result {
            symbol: pane.symbol,
            tint: pane.tint,
            title,
            subtitle: row.map(|_| pane.title),
            on_select: open(pane.id, row),
        };
        let by_id = |id: &&'static str| panes.iter().find(|pane| pane.id == *id).copied();

        let mut entries = Vec::new();
        if !text.trim().is_empty() {
            let hits = index(&panes).search(&text);
            if hits.is_empty() {
                entries.push(Entry::Header("Sin resultados"));
            }
            for hit in hits {
                let (id, row) = hit.key;
                if let Some(pane) = by_id(&id) {
                    entries.push(result(pane, hit.title, row));
                }
            }
        } else if let Some(suggestions) = self.suggestions {
            entries.push(Entry::Header("Sugerencias"));
            entries.extend(
                suggestions
                    .iter()
                    .filter_map(by_id)
                    .map(|pane| result(pane, pane.title, None)),
            );
            let recent: Vec<Entry> = recents
                .read()
                .iter()
                .filter_map(by_id)
                .map(|pane| result(pane, pane.title, None))
                .collect();
            if !recent.is_empty() {
                entries.push(Entry::Header("Recientes"));
                entries.extend(recent);
            }
        }
        let popover_open = (*search_focused.read() || *popover_hovered.read())
            && !dismissed
            && !entries.is_empty();
        let choices: Vec<Callback<(), ()>> = entries
            .iter()
            .filter_map(|entry| match entry {
                Entry::Result { on_select, .. } => Some(on_select.clone()),
                Entry::Header(_) => None,
            })
            .collect();
        let active = cursor
            .read()
            .as_ref()
            .filter(|(at, _)| *at == text)
            .map(|(_, index)| *index);
        let typed = text.clone();
        let on_navigate = move |key: NamedKey| {
            if !popover_open {
                return;
            }
            let last = choices.len().checked_sub(1);
            let next = match key {
                NamedKey::ArrowDown => last.map(|last| active.map_or(0, |i| (i + 1).min(last))),
                NamedKey::ArrowUp => active.and_then(|i| i.checked_sub(1)),
                NamedKey::Enter => {
                    if let Some(choice) = active.and_then(|i| choices.get(i)) {
                        choice.call(());
                    }
                    return;
                }
                _ => return,
            };
            cursor.set(next.map(|index| (typed.clone(), index)));
        };

        let mut sidebar = Sidebar::new().traffic_lights(true).child(
            rect()
                .width(Size::fill())
                .margin(Gaps::new(0.0, 0.0, SETTINGS.sidebar_search_gap as f32, 0.0))
                .child(
                    SearchField::new(query, "Buscar")
                        .a11y_id(search_id)
                        .on_navigate(on_navigate)
                        .on_focus_change(move |focused| {
                            search_focused.set(focused);
                            if focused {
                                dismissed_at.set(None);
                            }
                        }),
                )
                .maybe_child(popover_open.then(|| {
                    search_popover(entries, text.trim().is_empty(), active, popover_hovered)
                })),
        );
        if let Some((initials, name, detail)) = &self.account {
            sidebar = sidebar.child(SidebarAccount::new(
                initials.clone(),
                name.clone(),
                detail.clone(),
            ));
        }
        for group in &self.groups {
            let items = group.panes.iter().map(item);
            sidebar = match group.title {
                Some(title) => sidebar.child(SidebarSection::new(title).children(items)),
                None => sidebar.children(items),
            };
        }

        let detail =
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .direction(Direction::Vertical)
                .child(
                    Titlebar::new()
                        .with_sidebar(true)
                        .title(pane.map(|pane| pane.title).unwrap_or_default())
                        .left(
                            rect()
                                .direction(Direction::Horizontal)
                                .cross_align(Alignment::Center)
                                .spacing(SPACE_2)
                                .child(SidebarToggle)
                                .child(navigation_buttons(history)),
                        ),
                )
                .child(ScrollView::new().maybe_child(
                    pane.map(|pane| pane.form.clone().highlighted(*highlighted.read())),
                ));

        let restored = saved.sidebar.unwrap_or(SidebarLayout {
            width: SIDEBAR.width as f32,
            visible: true,
        });
        SplitView::new(sidebar, detail)
            .sidebar_width(restored.width)
            .sidebar_visible(restored.visible)
            .on_layout(move |sidebar| layout.set(Some(sidebar)))
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

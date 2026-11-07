use super::*;
use crate::i18n::text;

pub fn fold(text: &str) -> String {
    let mut folded = String::with_capacity(text.len());
    for c in text.chars().flat_map(char::to_lowercase) {
        match c {
            '\u{300}'..='\u{36f}' => {}
            'ß' => folded.push_str("ss"),
            'á' | 'à' | 'ä' | 'â' | 'ã' | 'å' => folded.push('a'),
            'é' | 'è' | 'ë' | 'ê' => folded.push('e'),
            'í' | 'ì' | 'ï' | 'î' => folded.push('i'),
            'ó' | 'ò' | 'ö' | 'ô' | 'õ' | 'ø' => folded.push('o'),
            'ú' | 'ù' | 'ü' | 'û' => folded.push('u'),
            'ñ' => folded.push('n'),
            'ç' => folded.push('c'),
            'š' => folded.push('s'),
            'ř' => folded.push('r'),
            'ł' => folded.push('l'),
            other => folded.push(other),
        }
    }
    folded
}

pub fn word_start(name: &str, query: &str) -> bool {
    name.char_indices().any(|(index, _)| {
        if index == 0 {
            return false;
        }
        let before = name[..index].chars().next_back();
        let boundary = before.is_none_or(|ch| !ch.is_alphanumeric());
        boundary && name[index..].starts_with(query)
    })
}

pub fn normalize_query(text: &str) -> String {
    fold(strip_mnemonic(text).trim())
}

pub(super) fn search_score(normalized: &str, query: &str, enabled: bool) -> u8 {
    let base = if normalized.starts_with(query) {
        0
    } else if word_start(normalized, query) {
        1
    } else {
        2
    };
    if enabled { base } else { base + 3 }
}

pub(super) fn filter_search_results(
    query: &str,
    searchable: &[SearchableEntry],
) -> Vec<SearchableEntry> {
    let normalized = normalize_query(query);
    let mut matches: Vec<(u8, String, &SearchableEntry)> = searchable
        .iter()
        .filter_map(|entry| {
            let label = normalize_query(entry.item.label()?);
            if !label.contains(&normalized) {
                return None;
            }
            let score = search_score(&label, &normalized, entry.item.enabled());
            Some((score, label, entry))
        })
        .collect();
    matches.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    matches
        .into_iter()
        .map(|(.., entry)| entry.clone())
        .collect()
}

pub(super) fn search_result_lists(
    searchable: Option<&[SearchableEntry]>,
    help_search: Option<&[SearchableEntry]>,
    query: &str,
) -> Option<(Vec<SearchableEntry>, Vec<SearchableEntry>)> {
    let searchable = searchable?;
    if query.is_empty() {
        return None;
    }
    Some((
        filter_search_results(query, searchable),
        help_search
            .map(|list| filter_search_results(query, list))
            .unwrap_or_default(),
    ))
}

pub(super) fn visible_items(
    items: &[MenuItem],
    menu_results: &[SearchableEntry],
    help_results: &[SearchableEntry],
    menu_scroll: usize,
    help_scroll: usize,
) -> Vec<MenuItem> {
    let mut out = Vec::new();
    let mut saw_search = false;
    for item in items {
        if matches!(item, MenuItem::SearchField { .. }) {
            out.push(item.clone());
            saw_search = true;
        }
    }
    if !saw_search {
        out.push(MenuItem::SearchField {
            placeholder: text!("search").to_string(),
        });
    }

    if !menu_results.is_empty() {
        out.push(MenuItem::Header {
            label: text!("menu-items").to_string(),
            count: Some(menu_results.len()),
        });
        for entry in menu_results
            .iter()
            .skip(menu_scroll)
            .take(MenuView::MAX_MENU_ROWS)
        {
            out.push(MenuItem::SearchResult {
                path: entry.path.clone(),
                item: Box::new(entry.item.clone()),
                source: SearchSource::Menu,
                location: entry.location.clone(),
            });
        }
    }

    if !help_results.is_empty() {
        out.push(MenuItem::Header {
            label: text!("help-topics").to_string(),
            count: Some(help_results.len()),
        });
        for entry in help_results
            .iter()
            .skip(help_scroll)
            .take(MenuView::MAX_HELP_ROWS)
        {
            out.push(MenuItem::SearchResult {
                path: entry.path.clone(),
                item: Box::new(entry.item.clone()),
                source: SearchSource::Help,
                location: None,
            });
        }
    }

    out.push(MenuItem::Separator);
    for item in items {
        if !matches!(item, MenuItem::SearchField { .. }) {
            out.push(item.clone());
        }
    }
    out
}

pub(super) fn find_result_section(
    items: &[MenuItem],
    source: SearchSource,
) -> Option<(usize, usize)> {
    let start = items.iter().position(|item| {
        matches!(item,
            MenuItem::SearchResult { source: s, .. } if *s == source
        )
    })?;
    let count = items
        .iter()
        .skip(start)
        .take_while(|item| {
            matches!(item,
                MenuItem::SearchResult { source: s, .. } if *s == source
            )
        })
        .count();
    Some((start, count))
}

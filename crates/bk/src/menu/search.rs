use super::*;
use crate::i18n::text;

pub fn fold(text: &str) -> String {
    let mut folded = String::with_capacity(text.len());
    for c in text.chars().flat_map(char::to_lowercase) {
        match c {
            '\u{300}'..='\u{36f}' => {}
            'ß' => folded.push_str("ss"),
            'á' | 'à' | 'ä' | 'â' | 'ã' | 'å' | 'ą' => folded.push('a'),
            'é' | 'è' | 'ë' | 'ê' | 'ě' | 'ę' => folded.push('e'),
            'í' | 'ì' | 'ï' | 'î' => folded.push('i'),
            'ó' | 'ò' | 'ö' | 'ô' | 'õ' | 'ø' => folded.push('o'),
            'ú' | 'ù' | 'ü' | 'û' | 'ů' => folded.push('u'),
            'ý' | 'ÿ' => folded.push('y'),
            'ñ' | 'ń' | 'ň' => folded.push('n'),
            'ç' | 'č' | 'ć' => folded.push('c'),
            'š' | 'ś' => folded.push('s'),
            'ž' | 'ź' | 'ż' => folded.push('z'),
            'ř' | 'ŕ' => folded.push('r'),
            'ł' | 'ľ' | 'ĺ' => folded.push('l'),
            'ď' => folded.push('d'),
            'ť' => folded.push('t'),
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
    if query.is_empty() {
        return None;
    }
    let results = |list: Option<&[SearchableEntry]>| {
        list.map(|list| filter_search_results(query, list))
            .unwrap_or_default()
    };
    Some((results(searchable), results(help_search)))
}

pub(super) fn visible_items(
    items: &[MenuItem],
    menu_results: &[SearchableEntry],
    help_results: &[SearchableEntry],
    menu_scroll: usize,
    help_scroll: usize,
) -> Vec<MenuItem> {
    let is_field = |item: &&MenuItem| matches!(item, MenuItem::SearchField { .. });
    let placeholder = (!items.iter().any(|item| is_field(&item))).then(|| MenuItem::SearchField {
        placeholder: text!("search").into(),
    });
    items
        .iter()
        .filter(is_field)
        .cloned()
        .chain(placeholder)
        .chain(result_section(
            menu_results,
            text!("menu-items"),
            SearchSource::Menu,
            menu_scroll,
            MenuView::MAX_MENU_ROWS,
        ))
        .chain(result_section(
            help_results,
            text!("help-topics"),
            SearchSource::Help,
            help_scroll,
            MenuView::MAX_HELP_ROWS,
        ))
        .chain([MenuItem::Separator])
        .chain(items.iter().filter(|item| !is_field(item)).cloned())
        .collect()
}

fn result_section<'a>(
    results: &'a [SearchableEntry],
    label: &'static str,
    source: SearchSource,
    scroll: usize,
    rows: usize,
) -> impl Iterator<Item = MenuItem> + 'a {
    let header = (!results.is_empty()).then(|| MenuItem::Header {
        label: label.into(),
        count: Some(results.len()),
    });
    header
        .into_iter()
        .chain(results.iter().skip(scroll).take(rows).map(move |entry| {
            MenuItem::SearchResult {
                item: Box::new(entry.item.clone()),
                source,
                location: entry
                    .location
                    .clone()
                    .filter(|_| source == SearchSource::Menu),
            }
        }))
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

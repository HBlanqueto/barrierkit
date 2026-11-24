pub use crate::menu::search::{fold, word_start};
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq)]
pub struct Entry<K> {
    pub key: K,
    pub title: Cow<'static, str>,
    pub keywords: &'static [&'static str],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SearchIndex<K> {
    entries: Vec<Entry<K>>,
}

impl<K: Clone> SearchIndex<K> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add(
        &mut self,
        key: K,
        title: impl Into<Cow<'static, str>>,
        keywords: &'static [&'static str],
    ) {
        self.entries.push(Entry {
            key,
            title: title.into(),
            keywords,
        });
    }

    pub fn search(&self, query: &str) -> Vec<Entry<K>> {
        let query = fold(query.trim());
        if query.is_empty() {
            return Vec::new();
        }
        self.entries
            .iter()
            .filter(|entry| {
                std::iter::once(entry.title.as_ref())
                    .chain(entry.keywords.iter().copied())
                    .any(|text| fold(text).contains(&query))
            })
            .cloned()
            .collect()
    }
}

pub use crate::menu::search::{fold, word_start};

#[derive(Clone, Debug, PartialEq)]
pub struct Entry<K> {
    pub key: K,
    pub title: &'static str,
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

    pub fn add(&mut self, key: K, title: &'static str, keywords: &'static [&'static str]) {
        self.entries.push(Entry {
            key,
            title,
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
                std::iter::once(entry.title)
                    .chain(entry.keywords.iter().copied())
                    .any(|text| fold(text).contains(&query))
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_query_matches_titles_and_keywords_ignoring_case_and_accents() {
        let mut index = SearchIndex::new();
        index.add("dock", "Tamaño del Dock", &["iconos"]);
        index.add("lock", "Contraseña", &["bloqueo", "seguridad"]);
        let titles = |q: &str| {
            index
                .search(q)
                .into_iter()
                .map(|e| e.key)
                .collect::<Vec<_>>()
        };
        assert_eq!(titles("tamano"), ["dock"]);
        assert_eq!(titles("BLOQUEO"), ["lock"]);
        assert_eq!(titles("   "), Vec::<&str>::new());
    }

    #[test]
    fn folding_drops_combining_marks_and_latin_variants() {
        assert_eq!(fold("Cafe\u{301}"), "cafe");
        assert_eq!(fold("Šřłø Straße"), "srlo strasse");
    }
}

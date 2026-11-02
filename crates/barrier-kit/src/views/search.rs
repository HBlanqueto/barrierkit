//! Search index over titled entries with keywords, matched without case or accents.

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

pub fn fold(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' | 'ã' | 'å' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            other => other,
        })
        .collect()
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
}

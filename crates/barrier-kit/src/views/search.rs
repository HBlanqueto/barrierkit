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

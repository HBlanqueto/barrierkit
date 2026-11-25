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

pub fn value<'a>(ini: &'a str, section: &str, key: &str) -> Option<&'a str> {
    let header = format!("[{section}]");
    ini.lines()
        .map(str::trim)
        .skip_while(|line| *line != header)
        .skip(1)
        .take_while(|line| !line.starts_with('['))
        .find_map(|line| {
            let (name, value) = line.split_once('=')?;
            (name.trim() == key).then_some(value.trim())
        })
}

pub fn set(ini: &str, section: &str, key: &str, value: Option<&str>) -> String {
    let header = format!("[{section}]");
    let entry = value.map(|value| format!("{key}={value}"));
    let mut out = Vec::new();
    let mut in_section = false;
    let mut seen_section = false;
    let mut written = entry.is_none();
    for line in ini.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if in_section && !written {
                out.extend(entry.clone());
                written = true;
            }
            in_section = trimmed == header;
            seen_section |= in_section;
            out.push(line.to_string());
            continue;
        }
        if in_section
            && let Some((name, _)) = trimmed.split_once('=')
            && name.trim() == key
        {
            if !written {
                out.extend(entry.clone());
                written = true;
            }
            continue;
        }
        out.push(line.to_string());
    }
    if !written {
        if !seen_section {
            if out.last().is_some_and(|line| !line.trim().is_empty()) {
                out.push(String::new());
            }
            out.push(header);
        }
        out.extend(entry);
    }
    let mut text = out.join("\n");
    text.push('\n');
    text
}

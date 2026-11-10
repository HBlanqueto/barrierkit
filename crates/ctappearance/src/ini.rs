//! Keys of the INI files that GTK 3 (`settings.ini`) and qt5ct and qt6ct read.

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

/// Sets `key` in `section`, or removes it with `None`; every other line stays.
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

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "gtk-application-prefer-dark-theme";

    #[test]
    fn a_key_is_set_in_place_and_a_missing_section_is_added() {
        assert_eq!(
            set("", "Settings", KEY, Some("true")),
            "[Settings]\ngtk-application-prefer-dark-theme=true\n"
        );
        let ini = "[Settings]\ngtk-theme-name=adw-gtk3\ngtk-application-prefer-dark-theme=0\n";
        assert_eq!(
            set(ini, "Settings", KEY, Some("true")),
            "[Settings]\ngtk-theme-name=adw-gtk3\ngtk-application-prefer-dark-theme=true\n"
        );
        assert_eq!(
            set("[Other]\na=1\n", "Settings", KEY, Some("false")),
            "[Other]\na=1\n\n[Settings]\ngtk-application-prefer-dark-theme=false\n"
        );
        assert_eq!(
            set(
                "[Settings]\na=1\n[Other]\nb=2\n",
                "Settings",
                KEY,
                Some("true")
            ),
            "[Settings]\na=1\ngtk-application-prefer-dark-theme=true\n[Other]\nb=2\n"
        );
    }

    #[test]
    fn removing_a_key_keeps_the_rest_and_reading_stops_at_the_next_section() {
        let ini = "[Appearance]\nstyle=Breeze\nicon_theme=Papirus\n[Fonts]\nstyle=x\n";
        assert_eq!(
            set(ini, "Appearance", "style", None),
            "[Appearance]\nicon_theme=Papirus\n[Fonts]\nstyle=x\n"
        );
        assert_eq!(value(ini, "Appearance", "icon_theme"), Some("Papirus"));
        assert_eq!(value(ini, "Fonts", "icon_theme"), None);
    }
}

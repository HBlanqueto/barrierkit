use std::path::Path;

use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(default)]
struct LanguageSetting {
    language: String,
}

pub fn read_language(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let setting: LanguageSetting = toml::from_str(&text).ok()?;
    Some(setting.language).filter(|language| !language.is_empty())
}

pub fn system_language() -> Option<String> {
    read_language(&crate::config_dir().join("language.domain"))
}

pub fn requested_languages(app_language: Option<&str>) -> Vec<String> {
    let chosen = app_language.map(str::to_owned).or_else(system_language);
    requested_from(chosen, |name| std::env::var(name).ok())
}

fn requested_from(chosen: Option<String>, var: impl Fn(&str) -> Option<String>) -> Vec<String> {
    if let Some(language) = chosen {
        return vec![language];
    }
    let Some(locale) = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .into_iter()
        .find_map(|name| var(name).filter(|value| !value.is_empty()))
        .filter(|locale| !is_posix(locale))
    else {
        return Vec::new();
    };
    // gettext order: LANGUAGE wins over the locale, but only when the locale is not C.
    let mut languages: Vec<String> = var("LANGUAGE")
        .unwrap_or_default()
        .split(':')
        .filter(|entry| !entry.is_empty())
        .map(tag)
        .collect();
    languages.push(tag(&locale));
    languages
}

fn tag(locale: &str) -> String {
    locale
        .split(['.', '@'])
        .next()
        .unwrap_or_default()
        .replace('_', "-")
}

fn is_posix(locale: &str) -> bool {
    matches!(tag(locale).as_str(), "C" | "POSIX")
}

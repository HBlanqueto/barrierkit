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

#[cfg(test)]
mod tests {
    use super::*;

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        |name| {
            pairs
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| value.to_string())
        }
    }

    #[test]
    fn a_chosen_language_overrides_the_environment() {
        let languages = requested_from(Some("en".into()), env(&[("LANG", "es_MX.UTF-8")]));
        assert_eq!(languages, ["en"]);
    }

    #[test]
    fn the_posix_locale_becomes_a_language_tag() {
        let languages = requested_from(None, env(&[("LANG", "es_MX.UTF-8@euro")]));
        assert_eq!(languages, ["es-MX"]);
    }

    #[test]
    fn lc_all_wins_over_lang_and_language_comes_first() {
        let languages = requested_from(
            None,
            env(&[
                ("LC_ALL", "es_AR.UTF-8"),
                ("LANG", "fr_FR"),
                ("LANGUAGE", "pt_BR:de"),
            ]),
        );
        assert_eq!(languages, ["pt-BR", "de", "es-AR"]);
    }

    #[test]
    fn the_c_locale_ignores_language_and_requests_nothing() {
        let languages = requested_from(None, env(&[("LANG", "C.UTF-8"), ("LANGUAGE", "es")]));
        assert!(languages.is_empty());
    }

    #[test]
    fn an_empty_language_setting_means_the_system_locale() {
        let dir = std::env::temp_dir().join(format!("ct-language-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("language.domain");
        std::fs::write(&path, "language = \"\"\n").unwrap();
        assert_eq!(read_language(&path), None);
        std::fs::write(&path, "language = \"es\"\n").unwrap();
        assert_eq!(read_language(&path).as_deref(), Some("es"));
        std::fs::remove_dir_all(dir).unwrap();
    }
}

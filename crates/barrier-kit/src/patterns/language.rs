use std::io;
use std::path::{Path, PathBuf};

use freya_core::prelude::*;
use vow_appearance::language::read_language;

use crate::app::{read_toml, write_toml};
use crate::i18n::text;
use crate::popup_button::PopUpButton;
use crate::views::form::Row;

const LANGUAGES: [Option<&str>; 3] = [None, Some("es"), Some("en")];

#[derive(PartialEq)]
pub struct LanguagePicker {
    path: PathBuf,
}

impl LanguagePicker {
    pub fn app(domain: &str) -> Self {
        Self {
            path: vow_appearance::config_dir().join(format!("{domain}.domain")),
        }
    }

    pub fn system() -> Self {
        Self {
            path: vow_appearance::config_dir().join("language.domain"),
        }
    }
}

pub fn app_language_row(domain: &str) -> Row {
    Row::new(text!("language"))
        .detail(text!("language-applies-on-reopen"))
        .control(LanguagePicker::app(domain))
}

pub fn system_language_row() -> Row {
    Row::new(text!("language"))
        .detail(text!("language-applies-to-new-apps"))
        .control(LanguagePicker::system())
}

impl Component for LanguagePicker {
    fn render(&self) -> impl IntoElement {
        let path = self.path.clone();
        let mut selected = use_state(|| {
            let current = read_language(&path);
            LANGUAGES
                .iter()
                .position(|language| *language == current.as_deref())
                .unwrap_or(0)
        });
        let path = self.path.clone();
        PopUpButton::new(text!("language"))
            .item(text!("language-automatic"))
            .item("Español")
            .item("English")
            .selected(selected())
            .on_change(move |index: usize| {
                if let Err(error) = write_language(&path, LANGUAGES[index]) {
                    tracing::warn!(path = %path.display(), %error, "could not save the language");
                }
                selected.set(index);
            })
    }
}

pub fn write_language(path: &Path, language: Option<&str>) -> io::Result<()> {
    let mut table: toml::Table = read_toml(path);
    match language {
        Some(language) => table.insert("language".into(), language.into()),
        None => table.remove("language"),
    };
    write_toml(path, &table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choosing_a_language_keeps_the_other_keys_of_the_domain() {
        let dir = std::env::temp_dir().join(format!("kit-language-{}", std::process::id()));
        let path = dir.join("marea.domain");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, "volume = 3\n").unwrap();
        write_language(&path, Some("en")).unwrap();
        assert_eq!(read_language(&path).as_deref(), Some("en"));
        write_language(&path, None).unwrap();
        assert_eq!(read_language(&path), None);
        assert!(
            std::fs::read_to_string(&path)
                .unwrap()
                .contains("volume = 3")
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}

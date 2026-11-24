use std::io;
use std::path::{Path, PathBuf};

use ctappearance::language::read_language;
use freya_core::prelude::*;

use crate::app::{try_read_toml, write_toml};
use crate::controlled::use_mirror;
use crate::elements::{Overrides, forward_overrides};
use crate::i18n::text;
use crate::popup_button::PopUpButton;
use crate::views::form::Row;

const LANGUAGES: [Option<&str>; 3] = [None, Some("es"), Some("en")];

#[derive(PartialEq)]
pub struct LanguagePicker {
    path: PathBuf,
    overrides: Overrides,
}

impl LanguagePicker {
    pub fn app(domain: &str) -> Self {
        Self {
            path: crate::app::domain(domain),
            overrides: Overrides::default(),
        }
    }

    pub fn system() -> Self {
        Self {
            path: ctappearance::config_dir().join("language.domain"),
            overrides: Overrides::default(),
        }
    }
}

pub fn app_language_row(domain: &str) -> Row {
    Row::new(text!("language"))
        .detail(text!("language-applies-on-reopen"))
        .control(LanguagePicker::app(domain))
}

pub fn system_language_row() -> Row {
    Row::new(text!("language")).control(LanguagePicker::system())
}

forward_overrides!(LanguagePicker);

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
        let selection = use_mirror(selected(), move |index: usize| {
            if let Err(error) = write_language(&path, LANGUAGES[index]) {
                tracing::warn!(path = %path.display(), %error, "could not save the language");
            }
            selected.set(index);
        });
        let root = PopUpButton::new(text!("language"), selection)
            .item(text!("language-automatic"))
            .item("Español")
            .item("English");
        self.overrides.delegate(root)
    }
}

pub fn write_language(path: &Path, language: Option<&str>) -> io::Result<()> {
    let mut table: toml::Table = try_read_toml(path)?;
    match language {
        Some(language) => table.insert("language".into(), language.into()),
        None => table.remove("language"),
    };
    write_toml(path, &table)
}

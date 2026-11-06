use std::sync::{LazyLock, OnceLock};

use i18n_embed::I18nAssets;
use i18n_embed::fluent::{FluentLanguageLoader, fluent_language_loader};
use i18n_embed::unic_langid::LanguageIdentifier;
use rust_embed::RustEmbed;
use vow_appearance::language::{read_language, requested_languages};

static APP_LANGUAGE: OnceLock<Option<String>> = OnceLock::new();

pub fn use_app_language(domain: &str) {
    let path = vow_appearance::config_dir().join(format!("{domain}.domain"));
    let _ = APP_LANGUAGE.set(read_language(&path));
}

pub(crate) fn languages() -> Vec<String> {
    requested_languages(APP_LANGUAGE.get().cloned().flatten().as_deref())
}

pub fn load(loader: FluentLanguageLoader, assets: &dyn I18nAssets) -> FluentLanguageLoader {
    let requested: Vec<LanguageIdentifier> = languages()
        .iter()
        .filter_map(|tag| tag.parse().ok())
        .collect();
    if let Err(error) = i18n_embed::select(&loader, assets, &requested) {
        tracing::warn!(%error, "could not load the translations");
    }
    loader.set_use_isolating(false);
    loader
}

#[macro_export]
macro_rules! tr {
    ($loader:expr, $id:literal) => {{
        static TEXT: ::std::sync::OnceLock<&'static str> = ::std::sync::OnceLock::new();
        *TEXT.get_or_init(|| ::i18n_embed_fl::fl!($loader, $id).leak())
    }};
    ($loader:expr, $id:literal, $($args:tt)*) => {
        ::i18n_embed_fl::fl!($loader, $id, $($args)*)
    };
}

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

pub(crate) static LOADER: LazyLock<FluentLanguageLoader> =
    LazyLock::new(|| load(fluent_language_loader!(), &Localizations));

macro_rules! text {
    ($($args:tt)*) => { $crate::tr!($crate::i18n::LOADER, $($args)*) };
}
pub(crate) use text;

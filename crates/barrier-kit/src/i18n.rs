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

// Expands in the app crate: `crate`, the `i18n/` folder and the package name are the app's.
#[allow(clippy::crate_in_macro_def)]
#[macro_export]
macro_rules! localize {
    () => {
        $crate::localize!(@with $);
    };
    (@with $d:tt) => {
        #[derive(::rust_embed::RustEmbed)]
        #[folder = "i18n/"]
        struct Localizations;

        pub static LOADER: ::std::sync::LazyLock<::i18n_embed::fluent::FluentLanguageLoader> =
            ::std::sync::LazyLock::new(|| {
                $crate::i18n::load(::i18n_embed::fluent::fluent_language_loader!(), &Localizations)
            });

        macro_rules! tr {
            ($d($d args:tt)*) => { $crate::tr!(crate::localize::LOADER, $d($d args)*) };
        }
        pub(crate) use tr;
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

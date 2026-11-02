use std::io;
use std::path::PathBuf;

use serde::Serialize;
use serde::de::DeserializeOwned;

fn domain(app_id: &str) -> PathBuf {
    vow_appearance::config_dir().join(format!("{app_id}.domain"))
}

pub fn load_defaults<T: DeserializeOwned + Default>(app_id: &str) -> T {
    std::fs::read_to_string(domain(app_id))
        .ok()
        .and_then(|text| toml::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_defaults<T: Serialize>(app_id: &str, value: &T) -> io::Result<()> {
    let path = domain(app_id);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let text = toml::to_string(value).map_err(io::Error::other)?;
    let staged = path.with_extension("domain.tmp");
    std::fs::write(&staged, text)?;
    std::fs::rename(staged, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
    struct Prefs {
        step: i32,
    }

    #[test]
    fn saved_defaults_load_back_and_a_missing_domain_gives_the_default() {
        let dir = std::env::temp_dir().join(format!("barrierkit-defaults-{}", std::process::id()));
        // SAFETY: single-threaded test; the variable only redirects this process's config dir.
        unsafe { std::env::set_var("MALAYSIA_CONFIG_DIR", &dir) };
        assert_eq!(load_defaults::<Prefs>("demo.missing"), Prefs::default());
        save_defaults("demo.saved", &Prefs { step: 5 }).unwrap();
        assert_eq!(load_defaults::<Prefs>("demo.saved"), Prefs { step: 5 });
        let _ = std::fs::remove_dir_all(dir);
    }
}

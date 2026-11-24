use std::env;
use std::path::{Path, PathBuf};

fn absolute(variable: &str) -> Option<PathBuf> {
    env::var_os(variable)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}

fn home() -> Option<PathBuf> {
    absolute("HOME")
}

pub fn config_home() -> Option<PathBuf> {
    absolute("XDG_CONFIG_HOME").or_else(|| home().map(|home| home.join(".config")))
}

pub fn data_home() -> Option<PathBuf> {
    absolute("XDG_DATA_HOME").or_else(|| home().map(|home| home.join(".local/share")))
}

pub fn cache_home() -> Option<PathBuf> {
    absolute("XDG_CACHE_HOME").or_else(|| home().map(|home| home.join(".cache")))
}

pub fn data_dirs() -> Vec<PathBuf> {
    let dirs: Vec<PathBuf> = env::var_os("XDG_DATA_DIRS")
        .map(|value| {
            env::split_paths(&value)
                .filter(|path| path.is_absolute())
                .collect()
        })
        .unwrap_or_default();
    if dirs.is_empty() {
        vec![
            PathBuf::from("/usr/local/share"),
            PathBuf::from("/usr/share"),
        ]
    } else {
        dirs
    }
}

pub fn user_dir(kind: &str) -> Option<PathBuf> {
    let text = std::fs::read_to_string(config_home()?.join("user-dirs.dirs")).ok()?;
    user_dir_in(&text, kind, &home()?)
}

fn user_dir_in(text: &str, kind: &str, home: &Path) -> Option<PathBuf> {
    let key = format!("XDG_{kind}_DIR");
    let value = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .rfind(|(name, _)| name.trim() == key)?
        .1
        .trim()
        .trim_matches('"');
    match value.strip_prefix("$HOME") {
        Some(rest) if rest.is_empty() || rest.starts_with('/') => {
            Some(home.join(rest.trim_start_matches('/')))
        }
        Some(_) => None,
        None => Some(PathBuf::from(value)).filter(|path| path.is_absolute()),
    }
}

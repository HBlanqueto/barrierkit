use std::env;
use std::path::PathBuf;

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

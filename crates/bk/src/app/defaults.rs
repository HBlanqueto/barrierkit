use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use freya_core::prelude::*;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::notifications::lock;

#[derive(Clone, Copy)]
pub(crate) struct DefaultsRevision(pub(crate) State<u64>);

static WATCHED_DOMAINS: Mutex<Vec<OsString>> = Mutex::new(Vec::new());
static WATCHES_EVERY_DOMAIN: AtomicBool = AtomicBool::new(false);

impl DefaultsRevision {
    // window.domain is read by the title bar, not by a hook.
    pub(crate) fn concerns(domain: &std::ffi::OsStr) -> bool {
        domain == "window.domain"
            || WATCHES_EVERY_DOMAIN.load(Ordering::Relaxed)
            || lock(&WATCHED_DOMAINS)
                .iter()
                .any(|watched| watched == domain)
    }
}

pub fn use_defaults<T>(app_id: &'static str) -> Memo<T>
where
    T: DeserializeOwned + Default + PartialEq + Clone + 'static,
{
    let file = domain(app_id).file_name().unwrap_or_default().to_owned();
    let mut watched = lock(&WATCHED_DOMAINS);
    if !watched.contains(&file) {
        watched.push(file);
    }
    drop(watched);
    reload_on_revision(move || load_defaults(app_id))
}

// The loader may read any file, so every domain change reaches this process.
pub fn use_watched<T: PartialEq + Clone + 'static>(load: impl Fn() -> T + 'static) -> Memo<T> {
    WATCHES_EVERY_DOMAIN.store(true, Ordering::Relaxed);
    reload_on_revision(load)
}

fn reload_on_revision<T: PartialEq + Clone + 'static>(load: impl Fn() -> T + 'static) -> Memo<T> {
    let revision = try_consume_root_context::<DefaultsRevision>();
    use_memo(move || {
        if let Some(DefaultsRevision(revision)) = revision {
            revision.read();
        }
        load()
    })
}

pub(crate) fn domain(app_id: &str) -> PathBuf {
    ctappearance::config_dir().join(format!("{app_id}.domain"))
}

fn state(app_id: &str) -> PathBuf {
    ctappearance::data_dir().join(format!("{app_id}.state"))
}

pub fn read_toml<T: DeserializeOwned + Default>(path: &Path) -> T {
    try_read_toml(path).unwrap_or_else(|error| {
        tracing::warn!(path = %path.display(), %error, "unreadable file, using defaults");
        T::default()
    })
}

// A missing file is the defaults; a file that exists but cannot be read is an error, so a save never replaces it.
pub(crate) fn try_read_toml<T: DeserializeOwned + Default>(path: &Path) -> io::Result<T> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(T::default()),
        Err(error) => return Err(error),
    };
    Ok(toml::from_str(&text).unwrap_or_else(|error| {
        tracing::warn!(path = %path.display(), %error, "invalid TOML, using defaults");
        back_up(path);
        T::default()
    }))
}

fn back_up(path: &Path) {
    let mut backup = path.as_os_str().to_owned();
    backup.push(".bak");
    if let Err(error) = std::fs::copy(path, &backup) {
        tracing::warn!(path = %path.display(), %error, "backing up invalid TOML");
    }
}

pub fn write_toml<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    ctappearance::replace_file(path, &toml::to_string(value).map_err(io::Error::other)?)
}

pub fn load_defaults<T: DeserializeOwned + Default>(app_id: &str) -> T {
    read_toml(&domain(app_id))
}

pub fn save_defaults<T: Serialize>(app_id: &str, value: &T) -> io::Result<()> {
    write_toml(&domain(app_id), value)
}

pub fn update_defaults<T: DeserializeOwned + Default + Serialize>(
    app_id: &str,
    change: impl FnOnce(&mut T),
) -> io::Result<()> {
    let path = domain(app_id);
    // The lock keeps two processes from reading the same file and each saving over the other's change. Without a
    // directory there is no file to race for.
    let mut lock = path.as_os_str().to_owned();
    lock.push(".lock");
    let lock = std::fs::File::create(lock).ok();
    if let Some(lock) = &lock {
        lock.lock()?;
    }
    let mut value = try_read_toml(&path)?;
    change(&mut value);
    write_toml(&path, &value)
}

pub fn load_restoration<T: DeserializeOwned + Default>(app_id: &str) -> T {
    read_toml(&state(app_id))
}

pub fn save_restoration<T: Serialize>(app_id: &str, value: &T) -> io::Result<()> {
    write_toml(&state(app_id), value)
}

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use freya_core::prelude::*;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::{lock, platform::app_folders};

#[derive(Clone, Copy)]
pub(crate) struct DefaultsRevision(pub(crate) State<u64>);

static WATCHED_DOMAINS: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());
static WATCHES_EVERY_DOMAIN: AtomicBool = AtomicBool::new(false);
static FOLDER_ADDED: AtomicBool = AtomicBool::new(false);
static APP_ID: OnceLock<String> = OnceLock::new();

impl DefaultsRevision {
    // window.domain is read by the title bar, not by a hook.
    pub(crate) fn concerns(domain: &std::ffi::OsStr) -> bool {
        domain == "window.domain"
            || WATCHES_EVERY_DOMAIN.load(Ordering::Relaxed)
            || lock(&WATCHED_DOMAINS)
                .iter()
                .any(|watched| watched.file_name() == Some(domain))
    }
}

pub(crate) fn watched_folders() -> Vec<PathBuf> {
    lock(&WATCHED_DOMAINS)
        .iter()
        .filter_map(|domain| domain.parent())
        .map(Path::to_path_buf)
        .collect()
}

pub(crate) fn folder_added() -> bool {
    FOLDER_ADDED.swap(false, Ordering::Relaxed)
}

pub(crate) fn record_app_id(app_id: &str) {
    APP_ID.get_or_init(|| app_id.to_owned());
}

/// SwiftUI's `@AppStorage`: `key` of the app's `<app_id>.domain`, saved when it changes. Before the app runs (a test) it lives in memory only.
pub fn use_app_storage<T>(key: &'static str, default: T) -> State<T>
where
    T: Serialize + DeserializeOwned + Clone + PartialEq + 'static,
{
    let path = use_hook(|| APP_ID.get().map(|app_id| domain(app_id)));
    let value = use_state(|| {
        path.as_deref()
            .and_then(|path| stored(path, key))
            .unwrap_or(default)
    });
    let mut saved = None;
    use_side_effect(move || {
        let current = value.read().clone();
        if saved
            .replace(current.clone())
            .is_none_or(|saved| saved == current)
        {
            return;
        }
        if let Some(path) = &path
            && let Err(error) = store(path, key, &current)
        {
            tracing::warn!(path = %path.display(), %error, key, "saving a setting");
        }
    });
    value
}

fn stored<T: DeserializeOwned>(path: &Path, key: &str) -> Option<T> {
    read_toml::<toml::Table>(path).remove(key)?.try_into().ok()
}

fn store<T: Serialize>(path: &Path, key: &str, value: &T) -> io::Result<()> {
    let value = toml::Value::try_from(value).map_err(io::Error::other)?;
    update_file(path, |table: &mut toml::Table| {
        table.insert(key.to_owned(), value);
    })
}

pub fn use_defaults<T>(app_id: &'static str) -> Memo<T>
where
    T: DeserializeOwned + Default + PartialEq + Clone + 'static,
{
    let file = domain(app_id);
    let mut watched = lock(&WATCHED_DOMAINS);
    if !watched.contains(&file) {
        watched.push(file);
        FOLDER_ADDED.store(true, Ordering::Relaxed);
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
    let (settings, _) = app_folders(app_id);
    moved(
        &ctappearance::config_dir(),
        &settings,
        &format!("{app_id}.domain"),
    )
}

pub(crate) fn restoration_file(app_id: &str, name: &str) -> PathBuf {
    let (_, state) = app_folders(app_id);
    moved(&ctappearance::data_dir(), &state, &format!("{name}.state"))
}

// Apps saved into the desktop's shared folder before each had its own; moving the file keeps the user's settings.
fn moved(shared: &Path, folder: &Path, name: &str) -> PathBuf {
    let (old, new) = (shared.join(name), folder.join(name));
    if old == new || new.exists() || !old.exists() {
        return new;
    }
    match ctappearance::create_private_dir(folder).and_then(|()| std::fs::rename(&old, &new)) {
        Ok(()) => new,
        Err(error) => {
            tracing::warn!(path = %old.display(), %error, "moving settings to the app's folder");
            old
        }
    }
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
    update_file(&domain(app_id), change)
}

fn update_file<T: DeserializeOwned + Default + Serialize>(
    path: &Path,
    change: impl FnOnce(&mut T),
) -> io::Result<()> {
    // The lock keeps two processes from reading the same file and each saving over the other's change. Without a
    // directory there is no file to race for.
    let mut lock = path.as_os_str().to_owned();
    lock.push(".lock");
    let lock = std::fs::File::create(lock).ok();
    if let Some(lock) = &lock {
        lock.lock()?;
    }
    let mut value = try_read_toml(path)?;
    change(&mut value);
    write_toml(path, &value)
}

pub fn load_restoration<T: DeserializeOwned + Default>(app_id: &str) -> T {
    read_toml(&restoration_file(app_id, app_id))
}

pub fn save_restoration<T: Serialize>(app_id: &str, value: &T) -> io::Result<()> {
    write_toml(&restoration_file(app_id, app_id), value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_in_the_shared_folder_moves_to_the_app_folder_once() {
        let root = std::env::temp_dir().join(format!("bk-moved-{}", std::process::id()));
        let (shared, folder) = (root.join("malaysia"), root.join("notes"));
        std::fs::create_dir_all(&shared).unwrap();
        std::fs::write(shared.join("notes.domain"), "old").unwrap();

        let path = moved(&shared, &folder, "notes.domain");
        assert_eq!(path, folder.join("notes.domain"));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "old");
        assert!(!shared.join("notes.domain").exists());

        std::fs::write(shared.join("notes.domain"), "stale").unwrap();
        let again = moved(&shared, &folder, "notes.domain");
        assert_eq!(std::fs::read_to_string(again).unwrap(), "old");
        assert_eq!(
            moved(&shared, &shared, "notes.domain"),
            shared.join("notes.domain")
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn storing_one_key_keeps_the_others() {
        let root = std::env::temp_dir().join(format!("bk-store-{}", std::process::id()));
        let path = root.join("hello.domain");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&path, "italic = true\nname = \"Ana\"\n").unwrap();

        store(&path, "bold", &true).unwrap();
        assert_eq!(stored::<bool>(&path, "bold"), Some(true));
        assert_eq!(stored::<bool>(&path, "italic"), Some(true));
        assert_eq!(stored::<String>(&path, "name").as_deref(), Some("Ana"));
        std::fs::remove_dir_all(root).unwrap();
    }
}

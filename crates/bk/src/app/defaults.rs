use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use freya_core::prelude::*;
use serde::Serialize;
use serde::de::DeserializeOwned;

#[derive(Clone, Copy)]
pub(crate) struct DefaultsRevision(pub(crate) State<u64>);

static WATCHED_DOMAINS: Mutex<Vec<OsString>> = Mutex::new(Vec::new());
static WATCHES_EVERY_DOMAIN: AtomicBool = AtomicBool::new(false);

impl DefaultsRevision {
    // window.domain is read by the title bar, not by a hook.
    pub(crate) fn concerns(domain: &std::ffi::OsStr) -> bool {
        domain == "window.domain"
            || WATCHES_EVERY_DOMAIN.load(Ordering::Relaxed)
            || WATCHED_DOMAINS
                .lock()
                .unwrap()
                .iter()
                .any(|watched| watched == domain)
    }
}

pub fn use_defaults<T>(app_id: &'static str) -> Memo<T>
where
    T: DeserializeOwned + Default + PartialEq + Clone + 'static,
{
    let file = domain(app_id).file_name().unwrap_or_default().to_owned();
    let mut watched = WATCHED_DOMAINS.lock().unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
    struct Prefs {
        step: i32,
    }

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("barrierkit-defaults-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_file_that_cannot_be_read_is_an_error_and_a_missing_one_is_the_defaults() {
        let dir = scratch("unreadable");
        std::fs::create_dir_all(&dir).unwrap();
        assert!(try_read_toml::<Prefs>(&dir).is_err());
        assert_eq!(
            try_read_toml::<Prefs>(&dir.join("missing.domain")).unwrap(),
            Prefs::default()
        );
    }

    #[test]
    fn use_watched_reloads_when_the_revision_moves() {
        use freya_core::integration::{App, AppComponent};
        use freya_testing::TestingRunner;
        use torin::prelude::Size2D;

        struct Reader {
            seen: State<i32>,
            path: PathBuf,
        }
        impl App for Reader {
            fn render(&self) -> impl IntoElement {
                let path = self.path.clone();
                let prefs = use_watched(move || read_toml::<Prefs>(&path));
                let mut seen = self.seen;
                seen.set_if_modified(prefs.read().step);
                rect()
            }
        }

        let dir = scratch("watched");
        let path = dir.join("demo.domain");
        write_toml(&path, &Prefs { step: 1 }).unwrap();
        let seen = State::create_global(0);
        let revision = State::create_global(0u64);
        let app = AppComponent::new(Reader {
            seen,
            path: path.clone(),
        });
        let (mut test, _) = TestingRunner::new(
            app,
            Size2D::new(10.0, 10.0),
            move |runner| {
                runner.provide_root_context(move || DefaultsRevision(revision));
            },
            1.0,
        );
        test.sync_and_update();
        assert_eq!(*seen.peek(), 1);

        write_toml(&path, &Prefs { step: 7 }).unwrap();
        let mut bump = revision;
        *bump.write() += 1;
        test.poll_n(std::time::Duration::from_millis(16), 3);
        assert_eq!(*seen.peek(), 7);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn written_toml_reads_back_and_a_missing_file_gives_the_default() {
        let dir = scratch("roundtrip");
        assert_eq!(
            read_toml::<Prefs>(&dir.join("missing.domain")),
            Prefs::default()
        );
        let path = dir.join("saved.domain");
        write_toml(&path, &Prefs { step: 5 }).unwrap();
        assert_eq!(read_toml::<Prefs>(&path), Prefs { step: 5 });
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn an_invalid_domain_is_backed_up_before_defaults_overwrite_it() {
        let dir = scratch("invalid");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("demo.domain");
        std::fs::write(&path, "step = \"seven\"").unwrap();
        assert_eq!(read_toml::<Prefs>(&path), Prefs::default());
        write_toml(&path, &Prefs::default()).unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.join("demo.domain.bak")).unwrap(),
            "step = \"seven\""
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}

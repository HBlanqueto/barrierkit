//! Single inotify watcher for the configuration and data directories.

use std::io;
use std::os::fd::AsFd;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify, InotifyEvent, WatchDescriptor};

use crate::{config_dir, data_dir};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppearanceEvent {
    Appearance,
    Wallpaper,
    WallpaperSelection,
    Defaults,
}

impl AppearanceEvent {
    pub fn restyles(self) -> bool {
        matches!(self, Self::Appearance | Self::Wallpaper)
    }
}

#[derive(Debug)]
pub struct AppearanceWatch {
    inotify: Inotify,
    config: WatchDescriptor,
    data: WatchDescriptor,
}

impl AppearanceWatch {
    pub fn new() -> io::Result<Self> {
        Self::new_for(config_dir(), data_dir())
    }

    fn new_for(config_dir: PathBuf, data_dir: PathBuf) -> io::Result<Self> {
        std::fs::create_dir_all(&config_dir)?;
        std::fs::create_dir_all(&data_dir)?;
        let inotify =
            Inotify::init(InitFlags::IN_NONBLOCK | InitFlags::IN_CLOEXEC).map_err(into_io_error)?;
        let config = inotify
            .add_watch(&config_dir, Self::flags())
            .map_err(into_io_error)?;
        let data = inotify
            .add_watch(&data_dir, Self::flags())
            .map_err(into_io_error)?;

        Ok(Self {
            inotify,
            config,
            data,
        })
    }

    fn flags() -> AddWatchFlags {
        AddWatchFlags::IN_CLOSE_WRITE | AddWatchFlags::IN_MOVED_TO | AddWatchFlags::IN_DELETE
    }

    pub fn drain(&self) -> io::Result<Vec<AppearanceEvent>> {
        let mut events = Vec::with_capacity(2);
        loop {
            match self.inotify.read_events() {
                Ok(batch) => {
                    if batch.is_empty() {
                        break;
                    }
                    for event in batch {
                        self.classify(&event, &mut events);
                    }
                }
                Err(nix::Error::EAGAIN) => break,
                Err(err) => return Err(into_io_error(err)),
            }
        }
        Ok(events)
    }

    fn classify(&self, event: &InotifyEvent, out: &mut Vec<AppearanceEvent>) {
        if event.mask.contains(AddWatchFlags::IN_Q_OVERFLOW) {
            for lost in [
                AppearanceEvent::Appearance,
                AppearanceEvent::Wallpaper,
                AppearanceEvent::WallpaperSelection,
                AppearanceEvent::Defaults,
            ] {
                push_unique(out, lost);
            }
            return;
        }
        let Some(name) = event.name.as_deref() else {
            return;
        };
        let (name, in_config, in_data) = (
            name.as_bytes(),
            event.wd == self.config,
            event.wd == self.data,
        );
        let kind = match name {
            b"appearance.domain" if in_config => AppearanceEvent::Appearance,
            b"wallpaper.domain" if in_data => AppearanceEvent::WallpaperSelection,
            b"wallpaper.state" if in_data => AppearanceEvent::Wallpaper,
            _ if in_config && name.ends_with(b".domain") => AppearanceEvent::Defaults,
            _ => return,
        };
        push_unique(out, kind);
    }
}

impl AsFd for AppearanceWatch {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.inotify.as_fd()
    }
}

fn push_unique(events: &mut Vec<AppearanceEvent>, event: AppearanceEvent) {
    if !events.contains(&event) {
        events.push(event);
    }
}

fn into_io_error(err: nix::Error) -> io::Error {
    io::Error::from(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_appearance_and_wallpaper_changes_restyle_the_desktop() {
        assert!(AppearanceEvent::Appearance.restyles());
        assert!(AppearanceEvent::Wallpaper.restyles());
        assert!(!AppearanceEvent::Defaults.restyles());
        assert!(!AppearanceEvent::WallpaperSelection.restyles());
    }
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, Instant};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn tmp_dirs() -> (PathBuf, PathBuf) {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let config = std::env::temp_dir().join(format!("vow-appearance-watch-config-{pid}-{n}"));
        let data = std::env::temp_dir().join(format!("vow-appearance-watch-data-{pid}-{n}"));
        std::fs::create_dir_all(&config).unwrap();
        std::fs::create_dir_all(&data).unwrap();
        (config, data)
    }

    fn write(path: &Path, text: &str) {
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, text).unwrap();
        std::fs::rename(&tmp, path).unwrap();
    }

    #[test]
    fn creates_watched_directories_when_missing() {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let config_dir =
            std::env::temp_dir().join(format!("vow-appearance-watch-missing-config-{pid}-{n}"));
        let data_dir =
            std::env::temp_dir().join(format!("vow-appearance-watch-missing-data-{pid}-{n}"));
        let _ = std::fs::remove_dir_all(&config_dir);
        let _ = std::fs::remove_dir_all(&data_dir);
        assert!(!config_dir.exists());
        assert!(!data_dir.exists());

        let watch = AppearanceWatch::new_for(config_dir.clone(), data_dir.clone()).unwrap();
        assert!(config_dir.exists());
        assert!(data_dir.exists());

        let path = config_dir.join("appearance.domain");
        write(&path, "color_scheme = \"dark\"\n");

        let deadline = Instant::now() + Duration::from_secs(2);
        let mut found = false;
        while Instant::now() < deadline {
            if let Ok(events) = watch.drain()
                && events.contains(&AppearanceEvent::Appearance)
            {
                found = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(
            found,
            "appearance.domain rename was not detected after creating dirs"
        );
        let _ = std::fs::remove_dir_all(&config_dir);
        let _ = std::fs::remove_dir_all(&data_dir);
    }

    #[test]
    fn detects_appearance_domain_rename() {
        let (config_dir, data_dir) = tmp_dirs();
        let watch = AppearanceWatch::new_for(config_dir.clone(), data_dir.clone()).unwrap();
        let path = config_dir.join("appearance.domain");

        write(&path, "color_scheme = \"dark\"\n");

        let deadline = Instant::now() + Duration::from_secs(2);
        let mut found = false;
        while Instant::now() < deadline {
            if let Ok(events) = watch.drain()
                && events.contains(&AppearanceEvent::Appearance)
            {
                found = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(found, "appearance.domain rename was not detected");
    }

    #[test]
    fn an_app_domain_change_is_reported_as_defaults() {
        let (config_dir, data_dir) = tmp_dirs();
        let watch = AppearanceWatch::new_for(config_dir.clone(), data_dir.clone()).unwrap();
        write(
            &config_dir.join("org.malaysia.Contador.domain"),
            "step = 5\n",
        );

        let deadline = Instant::now() + Duration::from_secs(2);
        let mut found = false;
        while Instant::now() < deadline {
            if let Ok(events) = watch.drain()
                && events.contains(&AppearanceEvent::Defaults)
            {
                found = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(found, "an app domain change was not detected");
    }

    #[test]
    fn a_domain_in_the_data_dir_is_not_reported_as_defaults() {
        let (config_dir, data_dir) = tmp_dirs();
        let watch = AppearanceWatch::new_for(config_dir, data_dir.clone()).unwrap();
        write(&data_dir.join("window.domain"), "title = \"Terminal\"\n");
        write(
            &data_dir.join("wallpaper.state"),
            "dominant = [0.8, 0.2, 0.1]\n",
        );

        let deadline = Instant::now() + Duration::from_secs(2);
        let mut events = Vec::new();
        while Instant::now() < deadline && !events.contains(&AppearanceEvent::Wallpaper) {
            events.extend(watch.drain().unwrap());
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(events, [AppearanceEvent::Wallpaper]);
    }

    #[test]
    fn detects_wallpaper_state_close_write() {
        let (config_dir, data_dir) = tmp_dirs();
        let watch = AppearanceWatch::new_for(config_dir.clone(), data_dir.clone()).unwrap();
        let path = data_dir.join("wallpaper.state");

        write(&path, "dominant = [0.8, 0.2, 0.1]\n");

        let deadline = Instant::now() + Duration::from_secs(2);
        let mut found = false;
        while Instant::now() < deadline {
            if let Ok(events) = watch.drain()
                && events.contains(&AppearanceEvent::Wallpaper)
            {
                found = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(found, "wallpaper.state close_write was not detected");
    }

    #[test]
    fn detects_wallpaper_domain_rename() {
        let (config_dir, data_dir) = tmp_dirs();
        let watch = AppearanceWatch::new_for(config_dir, data_dir.clone()).unwrap();
        let path = data_dir.join("wallpaper.domain");

        write(&path, "[desktops]\ndefault = \"file:///Solid Gray.png\"\n");

        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if let Ok(events) = watch.drain()
                && events.contains(&AppearanceEvent::WallpaperSelection)
            {
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("wallpaper.domain rename was not detected");
    }

    #[test]
    fn detects_wallpaper_state_delete() {
        let (config_dir, data_dir) = tmp_dirs();
        let watch = AppearanceWatch::new_for(config_dir, data_dir.clone()).unwrap();
        let path = data_dir.join("wallpaper.state");

        write(&path, "dominant = [0.8, 0.2, 0.1]\n");
        let _ = watch.drain();

        std::fs::remove_file(&path).unwrap();

        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if let Ok(events) = watch.drain()
                && events.contains(&AppearanceEvent::Wallpaper)
            {
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("wallpaper.state delete was not detected");
    }

    #[test]
    fn latency_from_write_to_event_is_under_50ms() {
        let (config_dir, data_dir) = tmp_dirs();
        let watch = AppearanceWatch::new_for(config_dir.clone(), data_dir.clone()).unwrap();
        let path = config_dir.join("appearance.domain");

        write(&path, "color_scheme = \"light\"\n");
        let _ = watch.drain();

        let start = Instant::now();
        write(&path, "color_scheme = \"dark\"\n");

        let deadline = start + Duration::from_millis(50);
        let mut latency = None;
        while Instant::now() < deadline {
            if let Ok(events) = watch.drain()
                && events.contains(&AppearanceEvent::Appearance)
            {
                latency = Some(start.elapsed());
                break;
            }
            std::thread::sleep(Duration::from_micros(100));
        }

        let elapsed = latency.expect("event did not arrive within 50 ms");
        assert!(
            elapsed < Duration::from_millis(50),
            "latency {elapsed:?} is not under 50 ms"
        );
    }
}

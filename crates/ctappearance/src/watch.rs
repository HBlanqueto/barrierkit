use std::ffi::{OsStr, OsString};
use std::io;
use std::os::fd::AsFd;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify, InotifyEvent, WatchDescriptor};

use crate::config::create_private_dir;
use crate::{config_dir, data_dir};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppearanceEvent {
    Appearance,
    Wallpaper,
    WallpaperSelection,
    Defaults(Option<OsString>),
}

impl AppearanceEvent {
    pub fn restyles(&self) -> bool {
        matches!(self, Self::Appearance | Self::Wallpaper)
    }
}

#[derive(Debug)]
pub struct AppearanceWatch {
    inotify: Inotify,
    config_dir: PathBuf,
    data_dir: PathBuf,
    config: WatchDescriptor,
    data: WatchDescriptor,
}

impl AppearanceWatch {
    pub fn new() -> io::Result<Self> {
        Self::new_for(config_dir(), data_dir())
    }

    fn new_for(config_dir: PathBuf, data_dir: PathBuf) -> io::Result<Self> {
        create_private_dir(&config_dir)?;
        create_private_dir(&data_dir)?;
        let inotify = Inotify::init(InitFlags::IN_NONBLOCK | InitFlags::IN_CLOEXEC)?;
        let config = inotify.add_watch(&config_dir, Self::flags())?;
        let data = inotify.add_watch(&data_dir, Self::flags())?;

        Ok(Self {
            inotify,
            config_dir,
            data_dir,
            config,
            data,
        })
    }

    fn flags() -> AddWatchFlags {
        AddWatchFlags::IN_CLOSE_WRITE
            | AddWatchFlags::IN_MOVED_TO
            | AddWatchFlags::IN_MOVED_FROM
            | AddWatchFlags::IN_DELETE
            | AddWatchFlags::IN_DELETE_SELF
            | AddWatchFlags::IN_MOVE_SELF
    }

    pub fn drain(&mut self) -> io::Result<Vec<AppearanceEvent>> {
        let mut events = Vec::with_capacity(2);
        loop {
            match self.inotify.read_events() {
                Ok(batch) => {
                    if batch.is_empty() {
                        break;
                    }
                    for event in batch {
                        if event
                            .mask
                            .intersects(AddWatchFlags::IN_DELETE_SELF | AddWatchFlags::IN_MOVE_SELF)
                            && let Err(error) = self.rewatch(event.wd)
                        {
                            tracing::warn!(%error, "watching the settings folders again");
                        }
                        self.classify(&event, &mut events);
                    }
                }
                Err(nix::Error::EAGAIN) => break,
                Err(err) => return Err(err.into()),
            }
        }
        Ok(events)
    }

    // inotify(7): a deleted directory loses its watch and a moved one keeps watching the old inode.
    fn rewatch(&mut self, lost: WatchDescriptor) -> io::Result<()> {
        for (watch, dir) in [
            (&mut self.config, &self.config_dir),
            (&mut self.data, &self.data_dir),
        ] {
            if *watch == lost {
                let _ = self.inotify.rm_watch(lost);
                create_private_dir(dir)?;
                *watch = self.inotify.add_watch(dir, Self::flags())?;
            }
        }
        Ok(())
    }

    fn classify(&self, event: &InotifyEvent, out: &mut Vec<AppearanceEvent>) {
        if event.mask.intersects(
            AddWatchFlags::IN_Q_OVERFLOW
                | AddWatchFlags::IN_DELETE_SELF
                | AddWatchFlags::IN_MOVE_SELF,
        ) {
            for lost in [
                AppearanceEvent::Appearance,
                AppearanceEvent::Wallpaper,
                AppearanceEvent::WallpaperSelection,
                AppearanceEvent::Defaults(None),
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
            b"wallpaper-selection.state" if in_data => AppearanceEvent::WallpaperSelection,
            b"wallpaper.state" if in_data => AppearanceEvent::Wallpaper,
            _ if in_config && name.ends_with(b".domain") => {
                AppearanceEvent::Defaults(Some(OsStr::from_bytes(name).to_owned()))
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, Instant};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn tmp_dirs() -> (PathBuf, PathBuf) {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let config = std::env::temp_dir().join(format!("ctappearance-watch-config-{pid}-{n}"));
        let data = std::env::temp_dir().join(format!("ctappearance-watch-data-{pid}-{n}"));
        std::fs::create_dir_all(&config).unwrap();
        std::fs::create_dir_all(&data).unwrap();
        (config, data)
    }

    fn write(path: &Path, text: &str) {
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, text).unwrap();
        std::fs::rename(&tmp, path).unwrap();
    }

    fn wait_for(watch: &mut AppearanceWatch, event: AppearanceEvent) -> bool {
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if watch.drain().is_ok_and(|events| events.contains(&event)) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        false
    }

    #[test]
    fn creates_watched_directories_when_missing() {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let config_dir =
            std::env::temp_dir().join(format!("ctappearance-watch-missing-config-{pid}-{n}"));
        let data_dir =
            std::env::temp_dir().join(format!("ctappearance-watch-missing-data-{pid}-{n}"));
        let _ = std::fs::remove_dir_all(&config_dir);
        let _ = std::fs::remove_dir_all(&data_dir);
        assert!(!config_dir.exists());
        assert!(!data_dir.exists());

        let mut watch = AppearanceWatch::new_for(config_dir.clone(), data_dir.clone()).unwrap();
        assert!(config_dir.exists());
        assert!(data_dir.exists());

        write(
            &config_dir.join("appearance.domain"),
            "color_scheme = \"dark\"\n",
        );
        assert!(
            wait_for(&mut watch, AppearanceEvent::Appearance),
            "appearance.domain rename was not detected after creating dirs"
        );
        let _ = std::fs::remove_dir_all(&config_dir);
        let _ = std::fs::remove_dir_all(&data_dir);
    }

    fn defaults(name: &str) -> AppearanceEvent {
        AppearanceEvent::Defaults(Some(name.into()))
    }

    #[test]
    fn an_app_domain_change_is_reported_as_defaults_with_its_file_name() {
        let (config_dir, data_dir) = tmp_dirs();
        let mut watch = AppearanceWatch::new_for(config_dir.clone(), data_dir.clone()).unwrap();
        write(&config_dir.join("one.domain"), "step = 5\n");
        write(&config_dir.join("two.domain"), "step = 6\n");

        let deadline = Instant::now() + Duration::from_secs(2);
        let mut events = Vec::new();
        while Instant::now() < deadline && events.len() < 2 {
            for event in watch.drain().unwrap() {
                push_unique(&mut events, event);
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(events, [defaults("one.domain"), defaults("two.domain")]);
    }

    #[test]
    fn a_deleted_or_moved_config_folder_is_created_and_watched_again() {
        for moved in [false, true] {
            let (config_dir, data_dir) = tmp_dirs();
            let mut watch = AppearanceWatch::new_for(config_dir.clone(), data_dir).unwrap();
            if moved {
                std::fs::rename(&config_dir, config_dir.with_extension("moved")).unwrap();
            } else {
                std::fs::remove_dir_all(&config_dir).unwrap();
            }
            let deadline = Instant::now() + Duration::from_secs(2);
            while Instant::now() < deadline && !config_dir.exists() {
                let _ = watch.drain();
                std::thread::sleep(Duration::from_millis(5));
            }
            assert!(config_dir.exists());

            write(
                &config_dir.join("appearance.domain"),
                "color_scheme = \"dark\"\n",
            );
            assert!(
                wait_for(&mut watch, AppearanceEvent::Appearance),
                "no event after the folder came back (moved: {moved})"
            );
        }
    }

    #[test]
    fn a_domain_in_the_data_dir_is_not_reported_as_defaults() {
        let (config_dir, data_dir) = tmp_dirs();
        let mut watch = AppearanceWatch::new_for(config_dir, data_dir.clone()).unwrap();
        write(&data_dir.join("example.domain"), "key = 1\n");
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
        let mut watch = AppearanceWatch::new_for(config_dir, data_dir.clone()).unwrap();
        write(
            &data_dir.join("wallpaper.state"),
            "dominant = [0.8, 0.2, 0.1]\n",
        );
        assert!(wait_for(&mut watch, AppearanceEvent::Wallpaper));
    }

    #[test]
    fn detects_wallpaper_selection_rename() {
        let (config_dir, data_dir) = tmp_dirs();
        let mut watch = AppearanceWatch::new_for(config_dir, data_dir.clone()).unwrap();
        write(
            &data_dir.join("wallpaper-selection.state"),
            "[desktops]\ndefault = \"file:///Solid Gray.png\"\n",
        );
        assert!(wait_for(&mut watch, AppearanceEvent::WallpaperSelection));
    }

    #[test]
    fn detects_appearance_domain_moved_away() {
        let (config_dir, data_dir) = tmp_dirs();
        let mut watch = AppearanceWatch::new_for(config_dir.clone(), data_dir).unwrap();
        let path = config_dir.join("appearance.domain");
        write(&path, "color_scheme = \"dark\"\n");
        let _ = watch.drain();

        std::fs::rename(&path, config_dir.join("appearance.old")).unwrap();
        assert!(wait_for(&mut watch, AppearanceEvent::Appearance));
    }

    #[test]
    fn detects_wallpaper_state_delete() {
        let (config_dir, data_dir) = tmp_dirs();
        let mut watch = AppearanceWatch::new_for(config_dir, data_dir.clone()).unwrap();
        let path = data_dir.join("wallpaper.state");
        write(&path, "dominant = [0.8, 0.2, 0.1]\n");
        let _ = watch.drain();

        std::fs::remove_file(&path).unwrap();
        assert!(wait_for(&mut watch, AppearanceEvent::Wallpaper));
    }
}

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

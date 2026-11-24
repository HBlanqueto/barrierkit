use std::{collections::HashMap, ffi::OsString, rc::Rc, time::Duration, time::SystemTime};

use freya_core::prelude::State;

use crate::{app::DefaultsRevision, platform::timer::TimerQueue};

const POLL: Duration = Duration::from_secs(1);

// shortcut: polls the config directory each second; FSEvents would wake only on a change.
pub(super) fn watch(timers: TimerQueue, revision: State<u64>, seen: HashMap<OsString, SystemTime>) {
    let queue = timers.clone();
    let poll = move || {
        let now = stamps();
        let changed = now
            .iter()
            .filter(|(name, time)| seen.get(*name) != Some(time))
            .map(|(name, _)| name)
            .chain(seen.keys().filter(|name| !now.contains_key(*name)))
            .any(|name| DefaultsRevision::concerns(name));
        if changed {
            let mut revision = revision;
            *revision.write() += 1;
        }
        watch(queue.clone(), revision, now);
    };
    timers.borrow_mut().push((POLL, Rc::new(poll)));
}

pub(super) fn stamps() -> HashMap<OsString, SystemTime> {
    std::fs::read_dir(ctappearance::config_dir())
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "domain"))
        .filter_map(|entry| Some((entry.file_name(), entry.metadata().ok()?.modified().ok()?)))
        .collect()
}

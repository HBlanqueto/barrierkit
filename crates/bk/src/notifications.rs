use std::sync::Mutex;

pub use crate::platform::notifications::{play_event_sound, post};

type Clicked = Box<dyn FnOnce(Option<String>) + Send>;

pub(crate) static WAITING: Mutex<Vec<(u32, Clicked)>> = Mutex::new(Vec::new());

pub struct Notification {
    pub app_name: &'static str,
    pub desktop_entry: &'static str,
    pub summary: String,
    pub body: String,
    pub action: String,
}

pub(crate) fn take<T>(pending: &Mutex<Vec<(u32, T)>>, id: u32) -> Option<T> {
    let mut pending = lock(pending);
    let index = pending.iter().position(|(waiting, _)| *waiting == id)?;
    Some(pending.remove(index).1)
}

pub(crate) fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

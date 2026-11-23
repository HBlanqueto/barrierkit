use std::process::Command;
use std::sync::Mutex;
use std::thread;

pub use crate::platform::notifications::post;

type Clicked = Box<dyn FnOnce(Option<String>) + Send>;

pub(crate) static WAITING: Mutex<Vec<(u32, Clicked)>> = Mutex::new(Vec::new());

pub struct Notification {
    pub app_name: &'static str,
    pub desktop_entry: &'static str,
    pub summary: String,
    pub body: String,
    pub action: String,
}

// `sound` is a name from the freedesktop sound naming specification.
pub fn play_event_sound(sound: &str) {
    match Command::new("canberra-gtk-play")
        .args(["--id", sound])
        .spawn()
    {
        Ok(mut child) => {
            thread::spawn(move || child.wait());
        }
        Err(error) => tracing::warn!(%error, "could not run canberra-gtk-play"),
    }
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

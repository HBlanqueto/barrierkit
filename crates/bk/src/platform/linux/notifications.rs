use std::collections::HashMap;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::thread;

use zbus::blocking::Proxy;
use zbus::zvariant::Value;

use crate::notifications::{Notification, WAITING, lock, take};
pub use crate::platform::linux::portal::notify;

const SERVICE: &str = "org.freedesktop.Notifications";
const PATH: &str = "/org/freedesktop/Notifications";
const DEFAULT_ACTION: &str = "default";

static TOKENS: Mutex<Vec<(u32, String)>> = Mutex::new(Vec::new());

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

// `clicked` gets the xdg-activation token the server sends before ActionInvoked (spec 1.2).
pub fn post(notice: Notification, clicked: impl FnOnce(Option<String>) + Send + 'static) {
    thread::spawn(move || {
        let Some(proxy) = proxy() else {
            return;
        };
        let hints = HashMap::from([("desktop-entry", Value::from(notice.desktop_entry))]);
        // Held across Notify so a NotificationClosed for this id waits until its callback is listed.
        let mut waiting = lock(&WAITING);
        let sent = proxy.call::<_, _, u32>(
            "Notify",
            &(
                notice.app_name,
                0u32,
                "",
                notice.summary.as_str(),
                notice.body.as_str(),
                vec![DEFAULT_ACTION, notice.action.as_str()],
                hints,
                -1i32,
            ),
        );
        match sent {
            Ok(id) => waiting.push((id, Box::new(clicked))),
            Err(error) => tracing::warn!(%error, "could not post a notification"),
        }
    });
}

static PROXY: OnceLock<Proxy<'static>> = OnceLock::new();
static CONNECTING: Mutex<()> = Mutex::new(());

fn proxy() -> Option<&'static Proxy<'static>> {
    if let Some(proxy) = PROXY.get() {
        return Some(proxy);
    }
    let _connecting = lock(&CONNECTING);
    if let Some(proxy) = PROXY.get() {
        return Some(proxy);
    }
    let connected = crate::platform::linux::blocking_session_bus()
        .and_then(|connection| Proxy::new(&connection, SERVICE, PATH, SERVICE));
    let proxy = match connected {
        Ok(proxy) => proxy,
        Err(error) => {
            tracing::warn!(%error, "no notification service");
            return None;
        }
    };
    match proxy.receive_all_signals() {
        Ok(signals) => {
            thread::spawn(move || signals.for_each(|message| hear(&message)));
        }
        Err(error) => tracing::warn!(%error, "notification clicks will not be heard"),
    }
    Some(PROXY.get_or_init(|| proxy))
}

fn hear(message: &zbus::Message) {
    let header = message.header();
    let body = message.body();
    match header.member().map(|member| member.as_str()) {
        Some("ActivationToken") => {
            if let Ok((id, token)) = body.deserialize::<(u32, String)>() {
                lock(&TOKENS).push((id, token));
            }
        }
        Some("ActionInvoked") => {
            if let Ok((id, _)) = body.deserialize::<(u32, String)>()
                && let Some(clicked) = take(&WAITING, id)
            {
                clicked(take(&TOKENS, id));
            }
        }
        Some("NotificationClosed") => {
            if let Ok((id, _)) = body.deserialize::<(u32, u32)>() {
                take(&WAITING, id);
                take(&TOKENS, id);
            }
        }
        _ => {}
    }
}

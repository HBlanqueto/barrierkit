use std::sync::atomic::{AtomicBool, Ordering};

use ctappearance::{AppearanceConfig, ColorScheme, load_appearance, set_system_scheme};
use sctk::reexports::calloop::ping::Ping;
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::OwnedValue;

use crate::platform::linux::blocking_session_bus;

const NAMESPACE: &str = "org.freedesktop.appearance";
const SCHEME: &str = "color-scheme";
const KEYS: [&str; 3] = [SCHEME, "contrast", "reduced-motion"];

static HIGHER_CONTRAST: AtomicBool = AtomicBool::new(false);
static REDUCED_MOTION: AtomicBool = AtomicBool::new(false);

// https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html
fn scheme(value: u32) -> Option<ColorScheme> {
    match value {
        1 => Some(ColorScheme::Dark),
        2 => Some(ColorScheme::Light),
        _ => None,
    }
}

fn settings(connection: &Connection) -> zbus::Result<Proxy<'static>> {
    Proxy::new(
        connection,
        "org.freedesktop.portal.Desktop",
        "/org/freedesktop/portal/desktop",
        "org.freedesktop.portal.Settings",
    )
}

fn read(connection: &Connection, key: &str) -> zbus::Result<u32> {
    let value: OwnedValue = settings(connection)?.call("ReadOne", &(NAMESPACE, key))?;
    Ok(u32::try_from(value)?)
}

fn record(key: &str, value: u32) {
    match key {
        SCHEME => set_system_scheme(scheme(value)),
        "contrast" => HIGHER_CONTRAST.store(value == 1, Ordering::Relaxed),
        _ => REDUCED_MOTION.store(value == 1, Ordering::Relaxed),
    }
}

// The shell is the desktop that answers the portal: it keeps the appearance domain (ADR 0114).
pub(crate) fn read_settings() {
    if cfg!(feature = "shell") {
        return;
    }
    let connection = match blocking_session_bus() {
        Ok(connection) => connection,
        Err(error) => {
            tracing::debug!(%error, "BarrierKit: no session bus for the settings portal");
            return;
        }
    };
    for key in KEYS {
        match read(&connection, key) {
            Ok(value) => record(key, value),
            Err(error) => {
                tracing::debug!(%error, key, "BarrierKit: no value from the settings portal")
            }
        }
    }
}

pub(crate) fn observe_settings(changed: Ping) {
    if cfg!(feature = "shell") {
        return;
    }
    let watch = move || -> zbus::Result<()> {
        let proxy = settings(&blocking_session_bus()?)?;
        for message in proxy.receive_signal("SettingChanged")? {
            let (namespace, key, value): (String, String, OwnedValue) =
                message.body().deserialize()?;
            if namespace == NAMESPACE && KEYS.contains(&key.as_str()) {
                record(&key, u32::try_from(value)?);
                changed.ping();
            }
        }
        Ok(())
    };
    let spawned = std::thread::Builder::new()
        .name("bk-color-scheme".into())
        .spawn(move || {
            if let Err(error) = watch() {
                tracing::debug!(%error, "BarrierKit: stopped watching the settings portal");
            }
        });
    if let Err(error) = spawned {
        tracing::warn!(%error, "BarrierKit: could not watch the color scheme");
    }
}

// The portal's accessibility settings add to the file's, as macOS's do (macos/system.rs).
pub(crate) fn appearance() -> AppearanceConfig {
    let config = load_appearance();
    AppearanceConfig {
        contrast: config.contrast || HIGHER_CONTRAST.load(Ordering::Relaxed),
        reduced_motion: config.reduced_motion || REDUCED_MOTION.load(Ordering::Relaxed),
        ..config
    }
}

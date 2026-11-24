use ctappearance::{AppearanceConfig, ColorScheme, load_appearance, set_system_scheme};
use sctk::reexports::calloop::ping::Ping;
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::OwnedValue;

use crate::platform::linux::blocking_session_bus;

const NAMESPACE: &str = "org.freedesktop.appearance";
const KEY: &str = "color-scheme";

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

fn read() -> zbus::Result<u32> {
    let value: OwnedValue =
        settings(&blocking_session_bus()?)?.call("ReadOne", &(NAMESPACE, KEY))?;
    Ok(u32::try_from(value)?)
}

// The shell is the desktop that answers the portal: it keeps the appearance domain (ADR 0114).
pub(crate) fn read_scheme() {
    if cfg!(feature = "shell") {
        return;
    }
    match read() {
        Ok(value) => set_system_scheme(scheme(value)),
        Err(error) => {
            tracing::debug!(%error, "BarrierKit: no color scheme from the settings portal")
        }
    }
}

pub(crate) fn observe_scheme(changed: Ping) {
    if cfg!(feature = "shell") {
        return;
    }
    let watch = move || -> zbus::Result<()> {
        let proxy = settings(&blocking_session_bus()?)?;
        for message in proxy.receive_signal("SettingChanged")? {
            let (namespace, key, value): (String, String, OwnedValue) =
                message.body().deserialize()?;
            if namespace == NAMESPACE && key == KEY {
                set_system_scheme(scheme(u32::try_from(value)?));
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

pub(crate) fn appearance() -> AppearanceConfig {
    load_appearance()
}

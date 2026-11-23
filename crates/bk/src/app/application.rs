#[cfg(not(target_os = "macos"))]
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
#[cfg(not(target_os = "macos"))]
use zbus::blocking::Connection;
#[cfg(not(target_os = "macos"))]
use zbus::fdo::{RequestNameFlags, RequestNameReply};
#[cfg(not(target_os = "macos"))]
use zbus::zvariant::OwnedValue;

use freya_core::prelude::try_consume_root_context;

use super::MenuActions;
use crate::document::{OpenedUrl, deliver_open};
use crate::platform::{Timers, WakeHandle, WindowControls};
use crate::portal::file_path;

pub enum Launch {
    Primary(Instance),
    Forwarded,
}

#[derive(Default)]
struct Requests {
    tokens: Vec<Option<String>>,
    actions: Vec<String>,
    uris: Vec<String>,
    wake: Option<WakeHandle>,
}

#[derive(Clone)]
pub struct Instance {
    requests: Arc<Mutex<Requests>>,
}

impl Instance {
    pub fn listen(&self) {
        let requests = self.requests.clone();
        let controls = WindowControls::get();
        let menu_actions = try_consume_root_context::<MenuActions>();
        let wake = Timers::get().wakeup(move || {
            let (tokens, actions, uris) = requests
                .lock()
                .map(|mut pending| {
                    (
                        std::mem::take(&mut pending.tokens),
                        std::mem::take(&mut pending.actions),
                        std::mem::take(&mut pending.uris),
                    )
                })
                .unwrap_or_default();
            for uri in uris {
                deliver_open(match file_path(&uri) {
                    Some(path) => OpenedUrl::File(path),
                    None => OpenedUrl::Other(uri),
                });
            }
            for token in tokens {
                match token {
                    Some(token) => controls.present(token),
                    None => controls.present_without_token(),
                }
            }
            if let Some(menu_actions) = &menu_actions {
                for action in actions {
                    menu_actions.dispatch(&action);
                }
            }
        });
        wake.wake();
        if let Ok(mut pending) = self.requests.lock() {
            pending.wake = Some(wake);
        }
    }
}

#[cfg(not(target_os = "macos"))]
struct Activation {
    requests: Arc<Mutex<Requests>>,
}

#[cfg(not(target_os = "macos"))]
#[zbus::interface(name = "org.freedesktop.Application")]
impl Activation {
    fn activate(&self, platform_data: HashMap<String, OwnedValue>) {
        let token = platform_data
            .get("activation-token")
            .and_then(|value| String::try_from(value.clone()).ok());
        present(&self.requests, token);
    }

    fn open(&self, uris: Vec<String>, platform_data: HashMap<String, OwnedValue>) {
        if let Ok(mut pending) = self.requests.lock() {
            pending.uris.extend(uris);
        }
        self.activate(platform_data);
    }

    fn activate_action(
        &self,
        name: String,
        _parameter: Vec<OwnedValue>,
        _platform_data: HashMap<String, OwnedValue>,
    ) {
        if let Ok(mut pending) = self.requests.lock() {
            pending.actions.push(name);
            if let Some(wake) = &pending.wake {
                wake.wake();
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn object_path(app_id: &str) -> String {
    format!("/{}", app_id.replace('.', "/").replace('-', "_"))
}

#[cfg(not(target_os = "macos"))]
fn forward(
    connection: &Connection,
    app_id: &str,
    path: &str,
    token: Option<String>,
) -> zbus::Result<()> {
    let platform_data: HashMap<String, OwnedValue> = token
        .and_then(|token| OwnedValue::try_from(zbus::zvariant::Value::from(token)).ok())
        .map(|token| ("activation-token".to_string(), token))
        .into_iter()
        .collect();
    connection.call_method(
        Some(app_id),
        path,
        Some("org.freedesktop.Application"),
        "Activate",
        &(platform_data,),
    )?;
    Ok(())
}

fn present(requests: &Mutex<Requests>, token: Option<String>) {
    if let Ok(mut pending) = requests.lock() {
        pending.tokens.push(token);
        if let Some(wake) = &pending.wake {
            wake.wake();
        }
    }
}

#[cfg(target_os = "macos")]
pub fn launch(app_id: &str) -> zbus::Result<Launch> {
    let requests = Arc::new(Mutex::new(Requests::default()));
    let pending = requests.clone();
    let primary = crate::platform::instance::claim(app_id, move || present(&pending, None))?;
    Ok(if primary {
        Launch::Primary(Instance { requests })
    } else {
        Launch::Forwarded
    })
}

// Export before owning the name: a second instance calls Activate as soon as the name is ours.
#[cfg(not(target_os = "macos"))]
pub fn launch(app_id: &str) -> zbus::Result<Launch> {
    let connection = crate::session_bus::blocking()?;
    let path = object_path(app_id);
    let requests = Arc::new(Mutex::new(Requests::default()));
    let activation = Activation {
        requests: requests.clone(),
    };
    if !async_io::block_on(crate::session_bus::export(
        connection.inner(),
        &path,
        activation,
    )) {
        return Err(zbus::Error::Failure(path));
    }
    let taken =
        match connection.request_name_with_flags(app_id, RequestNameFlags::DoNotQueue.into()) {
            Ok(RequestNameReply::Exists) | Err(zbus::Error::NameTaken) => true,
            Ok(_) => false,
            Err(error) => return Err(error),
        };
    if taken {
        connection
            .object_server()
            .remove::<Activation, _>(path.as_str())?;
        forward(
            &connection,
            app_id,
            &path,
            std::env::var("XDG_ACTIVATION_TOKEN").ok(),
        )?;
        return Ok(Launch::Forwarded);
    }
    Ok(Launch::Primary(Instance { requests }))
}

#[cfg(all(test, not(target_os = "macos")))]
mod tests {
    use super::*;

    #[test]
    fn a_second_launch_forwards_its_activation_token_to_the_first() {
        if std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_none() {
            return;
        }
        let app_id = format!("org.malaysia.test{}", std::process::id());
        let Ok(Launch::Primary(first)) = launch(&app_id) else {
            panic!("the first launch must own the name");
        };
        let second = Connection::session().unwrap();
        forward(
            &second,
            &app_id,
            &object_path(&app_id),
            Some("token-1".to_string()),
        )
        .unwrap();
        let tokens = std::mem::take(&mut first.requests.lock().unwrap().tokens);
        assert_eq!(tokens, [Some("token-1".to_string())]);
    }

    #[test]
    fn an_activation_without_a_token_still_asks_to_present_the_window() {
        if std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_none() {
            return;
        }
        let app_id = format!("org.malaysia.untokened{}", std::process::id());
        let Ok(Launch::Primary(first)) = launch(&app_id) else {
            panic!("the first launch must own the name");
        };
        let second = Connection::session().unwrap();
        forward(&second, &app_id, &object_path(&app_id), None).unwrap();
        let tokens = std::mem::take(&mut first.requests.lock().unwrap().tokens);
        assert_eq!(tokens, [None]);
    }

    #[test]
    fn the_object_path_follows_the_app_id() {
        assert_eq!(
            object_path("org.malaysia.tengen-app-show"),
            "/org/malaysia/tengen_app_show"
        );
    }
}

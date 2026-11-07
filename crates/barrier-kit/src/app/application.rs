use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use zbus::blocking::Connection;
use zbus::fdo::{RequestNameFlags, RequestNameReply};
use zbus::zvariant::OwnedValue;

use freya_core::prelude::try_consume_root_context;

use super::MenuActions;
use crate::platform::{Timers, WakeHandle, WindowControls};

pub enum Launch {
    Primary(Instance),
    Forwarded,
}

#[derive(Default)]
struct Requests {
    tokens: Vec<String>,
    actions: Vec<String>,
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
            let (tokens, actions) = requests
                .lock()
                .map(|mut pending| {
                    (
                        std::mem::take(&mut pending.tokens),
                        std::mem::take(&mut pending.actions),
                    )
                })
                .unwrap_or_default();
            for token in tokens {
                controls.present(token);
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

struct Activation {
    requests: Arc<Mutex<Requests>>,
}

#[zbus::interface(name = "org.freedesktop.Application")]
impl Activation {
    fn activate(&self, platform_data: HashMap<String, OwnedValue>) {
        let token = platform_data
            .get("activation-token")
            .and_then(|value| String::try_from(value.clone()).ok());
        if let (Some(token), Ok(mut pending)) = (token, self.requests.lock()) {
            pending.tokens.push(token);
            if let Some(wake) = &pending.wake {
                wake.wake();
            }
        }
    }

    fn open(&self, _uris: Vec<String>, _platform_data: HashMap<String, OwnedValue>) {}

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

fn object_path(app_id: &str) -> String {
    format!("/{}", app_id.replace('.', "/").replace('-', "_"))
}

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

pub fn launch(app_id: &str) -> zbus::Result<Launch> {
    let connection = crate::session_bus::blocking()?;
    let taken =
        match connection.request_name_with_flags(app_id, RequestNameFlags::DoNotQueue.into()) {
            Ok(RequestNameReply::Exists) | Err(zbus::Error::NameTaken) => true,
            Ok(_) => false,
            Err(error) => return Err(error),
        };
    let path = object_path(app_id);
    if taken {
        forward(
            &connection,
            app_id,
            &path,
            std::env::var("XDG_ACTIVATION_TOKEN").ok(),
        )?;
        return Ok(Launch::Forwarded);
    }
    let requests = Arc::new(Mutex::new(Requests::default()));
    connection.object_server().at(
        path.as_str(),
        Activation {
            requests: requests.clone(),
        },
    )?;
    Ok(Launch::Primary(Instance { requests }))
}

#[cfg(test)]
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
        assert_eq!(tokens, ["token-1"]);
    }

    #[test]
    fn the_object_path_follows_the_app_id() {
        assert_eq!(
            object_path("org.malaysia.tengen-app-show"),
            "/org/malaysia/tengen_app_show"
        );
    }
}

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use zbus::blocking::Connection;
use zbus::fdo::{RequestNameFlags, RequestNameReply};
use zbus::zvariant::OwnedValue;

use crate::app::{Requests, present};

struct Activation {
    requests: Arc<Mutex<Requests>>,
}

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

// Export before owning the name: a second instance calls Activate as soon as the name is ours.
pub(crate) fn claim(app_id: &str, requests: Arc<Mutex<Requests>>) -> zbus::Result<bool> {
    let connection = crate::platform::linux::blocking_session_bus()?;
    let path = object_path(app_id);
    let activation = Activation { requests };
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
        return Ok(false);
    }
    Ok(true)
}

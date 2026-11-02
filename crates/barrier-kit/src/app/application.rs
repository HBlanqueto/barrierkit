use std::collections::HashMap;

use zbus::blocking::Connection;
use zbus::fdo::{RequestNameFlags, RequestNameReply};
use zbus::zvariant::OwnedValue;

pub enum Launch {
    Primary(Connection),
    Forwarded,
}

struct Activation;

#[zbus::interface(name = "org.freedesktop.Application")]
impl Activation {
    fn activate(&self, _platform_data: HashMap<String, OwnedValue>) {}

    fn open(&self, _uris: Vec<String>, _platform_data: HashMap<String, OwnedValue>) {}

    fn activate_action(
        &self,
        _name: String,
        _parameter: Vec<OwnedValue>,
        _platform_data: HashMap<String, OwnedValue>,
    ) {
    }
}

fn object_path(app_id: &str) -> String {
    format!("/{}", app_id.replace('.', "/").replace('-', "_"))
}

pub fn launch(app_id: &str) -> zbus::Result<Launch> {
    let connection = Connection::session()?;
    let reply = connection.request_name_with_flags(app_id, RequestNameFlags::DoNotQueue.into())?;
    let path = object_path(app_id);
    if reply == RequestNameReply::Exists {
        let platform_data: HashMap<String, OwnedValue> = std::env::var("XDG_ACTIVATION_TOKEN")
            .ok()
            .and_then(|token| OwnedValue::try_from(zbus::zvariant::Value::from(token)).ok())
            .map(|token| ("activation-token".to_string(), token))
            .into_iter()
            .collect();
        connection.call_method(
            Some(app_id),
            path.as_str(),
            Some("org.freedesktop.Application"),
            "Activate",
            &(platform_data,),
        )?;
        return Ok(Launch::Forwarded);
    }
    connection.object_server().at(path.as_str(), Activation)?;
    Ok(Launch::Primary(connection))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_object_path_follows_the_app_id() {
        assert_eq!(
            object_path("org.malaysia.tengen-app-show"),
            "/org/malaysia/tengen_app_show"
        );
    }
}

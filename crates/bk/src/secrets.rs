use std::collections::HashMap;

#[cfg(not(target_os = "macos"))]
use zbus::blocking::Connection;
use zbus::zvariant::OwnedObjectPath;
#[cfg(not(target_os = "macos"))]
use zbus::zvariant::{ObjectPath, OwnedValue, Value};
#[cfg(not(target_os = "macos"))]
use zeroize::Zeroize;
use zeroize::Zeroizing;

#[cfg(not(target_os = "macos"))]
const SERVICE: &str = "org.freedesktop.secrets";
#[cfg(not(target_os = "macos"))]
const SERVICE_PATH: &str = "/org/freedesktop/secrets";
#[cfg(not(target_os = "macos"))]
const SERVICE_INTERFACE: &str = "org.freedesktop.Secret.Service";
#[cfg(not(target_os = "macos"))]
const ITEM_INTERFACE: &str = "org.freedesktop.Secret.Item";
// Secret Service API 0.2 §“Collections”: the alias for the user's default collection.
#[cfg(not(target_os = "macos"))]
const DEFAULT_COLLECTION: &str = "/org/freedesktop/secrets/aliases/default";
#[cfg(not(target_os = "macos"))]
const NO_PROMPT: &str = "/";

#[cfg(not(target_os = "macos"))]
type Secret = (OwnedObjectPath, Vec<u8>, Vec<u8>, String);

#[derive(Debug)]
pub enum Error {
    // The keyring asked to be unlocked; its own prompt is showing.
    Locked,
    Bus(zbus::Error),
    // The OSStatus of a failed Keychain call.
    #[cfg(target_os = "macos")]
    Keychain(i32),
}

impl From<zbus::Error> for Error {
    fn from(error: zbus::Error) -> Self {
        Self::Bus(error)
    }
}

impl From<zbus::zvariant::Error> for Error {
    fn from(error: zbus::zvariant::Error) -> Self {
        Self::Bus(error.into())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub path: OwnedObjectPath,
    pub label: String,
    pub attributes: HashMap<String, String>,
}

#[cfg(target_os = "macos")]
pub struct Secrets;

// The Keychain has no object paths; the label and the attributes find an item again.
#[cfg(target_os = "macos")]
pub(crate) fn keychain_path() -> OwnedObjectPath {
    OwnedObjectPath::try_from("/keychain").expect("a valid object path")
}

#[cfg(target_os = "macos")]
impl Secrets {
    pub fn open() -> Result<Self, Error> {
        Ok(Self)
    }

    pub fn search(&self, attributes: &HashMap<&str, &str>) -> Result<Vec<Item>, Error> {
        crate::platform::keychain::search(attributes)
    }

    pub fn secret(&self, item: &Item) -> Result<Zeroizing<Vec<u8>>, Error> {
        crate::platform::keychain::secret(item)
    }

    pub fn create(
        &self,
        label: &str,
        attributes: &HashMap<&str, &str>,
        secret: &[u8],
    ) -> Result<(), Error> {
        crate::platform::keychain::store(label, attributes, secret)
    }

    pub fn set_secret(&self, item: &Item, secret: &[u8]) -> Result<(), Error> {
        crate::platform::keychain::replace(item, secret)
    }

    pub fn delete(&self, item: &Item) -> Result<(), Error> {
        crate::platform::keychain::delete(item)
    }
}

#[cfg(not(target_os = "macos"))]
pub struct Secrets {
    connection: Connection,
    session: OwnedObjectPath,
}

#[cfg(not(target_os = "macos"))]
impl Secrets {
    // §“Transfer of Secrets”: the plain algorithm; the bus is local to the user.
    pub fn open() -> Result<Self, Error> {
        let connection = crate::session_bus::blocking()?;
        let reply = connection.call_method(
            Some(SERVICE),
            SERVICE_PATH,
            Some(SERVICE_INTERFACE),
            "OpenSession",
            &("plain", Value::from("")),
        )?;
        let (_, session): (OwnedValue, OwnedObjectPath) = reply.body().deserialize()?;
        Ok(Self {
            connection,
            session,
        })
    }

    fn call<B, R>(&self, path: &str, interface: &str, method: &str, body: &B) -> Result<R, Error>
    where
        B: serde::Serialize + zbus::zvariant::DynamicType,
        R: for<'d> serde::Deserialize<'d> + zbus::zvariant::Type,
    {
        let reply =
            self.connection
                .call_method(Some(SERVICE), path, Some(interface), method, body)?;
        Ok(reply.body().deserialize()?)
    }

    // §“Prompts”: a prompt other than “/” is shown by the service, which answers later.
    fn prompted(&self, prompt: &OwnedObjectPath) -> Result<(), Error> {
        if prompt.as_str() == NO_PROMPT {
            return Ok(());
        }
        let _: () = self.call(
            prompt.as_str(),
            "org.freedesktop.Secret.Prompt",
            "Prompt",
            &"",
        )?;
        Err(Error::Locked)
    }

    fn property(&self, path: &str, name: &str) -> Result<OwnedValue, Error> {
        self.call(
            path,
            "org.freedesktop.DBus.Properties",
            "Get",
            &(ITEM_INTERFACE, name),
        )
    }

    pub fn search(&self, attributes: &HashMap<&str, &str>) -> Result<Vec<Item>, Error> {
        let (mut unlocked, locked): (Vec<OwnedObjectPath>, Vec<OwnedObjectPath>) =
            self.call(SERVICE_PATH, SERVICE_INTERFACE, "SearchItems", attributes)?;
        if !locked.is_empty() {
            let (opened, prompt): (Vec<OwnedObjectPath>, OwnedObjectPath) =
                self.call(SERVICE_PATH, SERVICE_INTERFACE, "Unlock", &locked)?;
            self.prompted(&prompt)?;
            unlocked.extend(opened);
        }
        unlocked
            .into_iter()
            .map(|path| {
                let label = String::try_from(self.property(path.as_str(), "Label")?)?;
                let attributes = HashMap::<String, String>::try_from(
                    self.property(path.as_str(), "Attributes")?,
                )?;
                Ok(Item {
                    path,
                    label,
                    attributes,
                })
            })
            .collect()
    }

    pub fn secret(&self, item: &Item) -> Result<Zeroizing<Vec<u8>>, Error> {
        let (_, _, value, _): Secret = self.call(
            item.path.as_str(),
            ITEM_INTERFACE,
            "GetSecret",
            &self.session,
        )?;
        Ok(Zeroizing::new(value))
    }

    fn wrapped(&self, secret: &[u8]) -> Secret {
        (
            self.session.clone(),
            Vec::new(),
            secret.to_vec(),
            "text/plain".to_string(),
        )
    }

    pub fn create(
        &self,
        label: &str,
        attributes: &HashMap<&str, &str>,
        secret: &[u8],
    ) -> Result<(), Error> {
        let mut properties: HashMap<&str, Value> = HashMap::new();
        properties.insert("org.freedesktop.Secret.Item.Label", Value::from(label));
        properties.insert(
            "org.freedesktop.Secret.Item.Attributes",
            Value::from(attributes.clone()),
        );
        let mut wrapped = self.wrapped(secret);
        let created: Result<(OwnedObjectPath, OwnedObjectPath), Error> = self.call(
            DEFAULT_COLLECTION,
            "org.freedesktop.Secret.Collection",
            "CreateItem",
            &(properties, &wrapped, true),
        );
        wrapped.2.zeroize();
        self.prompted(&created?.1)
    }

    pub fn set_secret(&self, item: &Item, secret: &[u8]) -> Result<(), Error> {
        let mut wrapped = self.wrapped(secret);
        let set: Result<(), Error> = self.call(
            item.path.as_str(),
            ITEM_INTERFACE,
            "SetSecret",
            &(&wrapped,),
        );
        wrapped.2.zeroize();
        set
    }

    pub fn delete(&self, item: &Item) -> Result<(), Error> {
        let prompt: OwnedObjectPath =
            self.call(item.path.as_str(), ITEM_INTERFACE, "Delete", &())?;
        self.prompted(&prompt)
    }
}

#[cfg(not(target_os = "macos"))]
impl Drop for Secrets {
    fn drop(&mut self) {
        let path = ObjectPath::from(&self.session);
        let _ = self.connection.call_method(
            Some(SERVICE),
            path,
            Some("org.freedesktop.Secret.Session"),
            "Close",
            &(),
        );
    }
}

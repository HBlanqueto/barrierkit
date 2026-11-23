use std::collections::HashMap;

use zbus::zvariant::OwnedObjectPath;

pub use crate::platform::keychain::Secrets;

#[derive(Debug)]
pub enum Error {
    // The keyring asked to be unlocked; its own prompt is showing.
    Locked,
    Bus(zbus::Error),
    // The OSStatus of a failed Keychain call.
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

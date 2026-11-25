use std::collections::HashMap;

pub use crate::platform::keychain::Secrets;
use crate::platform::keychain::{BusError, ItemPath};

#[derive(Debug)]
pub enum Error {
    // The keyring asked to be unlocked; its own prompt is showing.
    Locked,
    Bus(BusError),
    // The OSStatus of a failed Keychain call.
    Keychain(i32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub path: ItemPath,
    pub label: String,
    pub attributes: HashMap<String, String>,
}

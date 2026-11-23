use std::collections::HashMap;

use security_framework::item::{ItemClass, ItemSearchOptions, Limit};
use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
};
use zbus::zvariant::OwnedObjectPath;
use zeroize::Zeroizing;

use crate::secrets::{Error, Item};

// A Keychain generic password has a service and an account; the service carries the attributes, so a search is a list
// filtered here.
const PREFIX: &str = "barrierkit?";

fn escape(text: &str) -> String {
    text.replace('%', "%25")
        .replace('=', "%3D")
        .replace('&', "%26")
}

fn unescape(text: &str) -> String {
    text.replace("%3D", "=")
        .replace("%26", "&")
        .replace("%25", "%")
}

pub(crate) fn service(attributes: &HashMap<&str, &str>) -> String {
    let mut pairs: Vec<_> = attributes
        .iter()
        .map(|(key, value)| format!("{}={}", escape(key), escape(value)))
        .collect();
    pairs.sort();
    format!("{PREFIX}{}", pairs.join("&"))
}

fn attributes_of(service: &str) -> Option<HashMap<String, String>> {
    let rest = service.strip_prefix(PREFIX)?;
    Some(
        rest.split('&')
            .filter_map(|pair| pair.split_once('='))
            .map(|(key, value)| (unescape(key), unescape(value)))
            .collect(),
    )
}

fn holds(query: &HashMap<&str, &str>, attributes: &HashMap<String, String>) -> bool {
    query
        .iter()
        .all(|(key, value)| attributes.get(*key).is_some_and(|held| held == value))
}

fn failed(error: security_framework::base::Error) -> Error {
    Error::Keychain(error.code())
}

fn service_of(item: &Item) -> String {
    let attributes: HashMap<&str, &str> = item
        .attributes
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    service(&attributes)
}

// The Keychain has no object paths; the label and the attributes find an item again.
fn keychain_path() -> OwnedObjectPath {
    OwnedObjectPath::try_from("/keychain").expect("a valid object path")
}

pub struct Secrets;

impl Secrets {
    pub fn open() -> Result<Self, Error> {
        Ok(Self)
    }

    pub fn search(&self, query: &HashMap<&str, &str>) -> Result<Vec<Item>, Error> {
        let found = match ItemSearchOptions::new()
            .class(ItemClass::generic_password())
            .load_attributes(true)
            .limit(Limit::All)
            .search()
        {
            Ok(found) => found,
            // errSecItemNotFound: nothing is stored yet.
            Err(error) if error.code() == -25300 => return Ok(Vec::new()),
            Err(error) => return Err(failed(error)),
        };
        Ok(found
            .iter()
            .filter_map(|result| {
                let fields = result.simplify_dict()?;
                let attributes = attributes_of(fields.get("svce")?)?;
                holds(query, &attributes).then(|| Item {
                    path: keychain_path(),
                    label: fields.get("acct").cloned().unwrap_or_default(),
                    attributes,
                })
            })
            .collect())
    }

    pub fn secret(&self, item: &Item) -> Result<Zeroizing<Vec<u8>>, Error> {
        get_generic_password(&service_of(item), &item.label)
            .map(Zeroizing::new)
            .map_err(failed)
    }

    pub fn create(
        &self,
        label: &str,
        attributes: &HashMap<&str, &str>,
        secret: &[u8],
    ) -> Result<(), Error> {
        set_generic_password(&service(attributes), label, secret).map_err(failed)
    }

    pub fn set_secret(&self, item: &Item, secret: &[u8]) -> Result<(), Error> {
        set_generic_password(&service_of(item), &item.label, secret).map_err(failed)
    }

    pub fn delete(&self, item: &Item) -> Result<(), Error> {
        delete_generic_password(&service_of(item), &item.label).map_err(failed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_attributes_of_an_item_survive_the_service_that_carries_them() {
        let attributes = HashMap::from([("user", "ada=1&2"), ("host", "100%.example")]);
        let back = attributes_of(&service(&attributes)).unwrap();
        assert_eq!(back.get("user").map(String::as_str), Some("ada=1&2"));
        assert_eq!(back.get("host").map(String::as_str), Some("100%.example"));
        assert_eq!(back.len(), 2);
    }

    #[test]
    fn the_same_attributes_make_the_same_service_whatever_their_order() {
        let one = HashMap::from([("a", "1"), ("b", "2")]);
        let other = HashMap::from([("b", "2"), ("a", "1")]);
        assert_eq!(service(&one), service(&other));
    }

    #[test]
    fn a_service_the_kit_did_not_write_is_not_an_item_of_the_kit() {
        assert!(attributes_of("com.apple.network.password").is_none());
    }

    #[test]
    fn a_query_holds_when_every_pair_of_it_is_in_the_item() {
        let item = HashMap::from([
            ("a".to_string(), "1".to_string()),
            ("b".to_string(), "2".to_string()),
        ]);
        assert!(holds(&HashMap::from([("a", "1")]), &item));
        assert!(!holds(&HashMap::from([("a", "2")]), &item));
        assert!(!holds(&HashMap::from([("c", "1")]), &item));
        assert!(holds(&HashMap::new(), &item));
    }
}

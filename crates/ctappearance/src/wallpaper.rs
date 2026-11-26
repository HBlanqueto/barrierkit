use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};

pub fn file_uri(path: &Path) -> String {
    let mut uri = String::from("file://");
    for &byte in path.as_os_str().as_encoded_bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~!$&'()*+,;=:@/".contains(&byte) {
            uri.push(byte as char);
        } else {
            uri.push_str(&format!("%{byte:02X}"));
        }
    }
    uri
}

pub fn file_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let (host, path) = rest.split_at(rest.find('/')?);
    if !(host.is_empty() || host == "localhost") {
        return None;
    }
    let rest = path.as_bytes();
    let mut bytes = Vec::with_capacity(rest.len());
    let mut index = 0;
    while index < rest.len() {
        let escaped = (rest[index] == b'%')
            .then(|| rest.get(index + 1..index + 3))
            .flatten()
            .filter(|hex| hex.iter().all(u8::is_ascii_hexdigit))
            .and_then(|hex| u8::from_str_radix(std::str::from_utf8(hex).ok()?, 16).ok());
        match escaped {
            Some(byte) => {
                bytes.push(byte);
                index += 3;
            }
            None => {
                bytes.push(rest[index]);
                index += 1;
            }
        }
    }
    Some(PathBuf::from(std::ffi::OsString::from_vec(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_path_drops_a_local_host_rejects_another_and_decodes_only_two_hex_digits() {
        let path = |uri| file_path(uri).map(|path| path.into_os_string());
        assert_eq!(path("file://localhost/tmp/a%20b"), Some("/tmp/a b".into()));
        assert_eq!(path("file:///tmp/%+1"), Some("/tmp/%+1".into()));
        assert_eq!(path("file://example.org/tmp/a"), None);
    }
}

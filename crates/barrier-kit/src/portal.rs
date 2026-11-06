use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use futures_util::StreamExt;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

static NEXT_REQUEST: AtomicU64 = AtomicU64::new(0);

const PORTAL: &str = "org.freedesktop.portal.Desktop";
const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";

pub async fn open_files(title: &str, multiple: bool, directory: bool) -> Option<Vec<PathBuf>> {
    let options = HashMap::from([
        ("multiple", Value::from(multiple)),
        ("directory", Value::from(directory)),
    ]);
    choose("OpenFile", title, options).await
}

pub async fn save_file(title: &str, current_name: &str) -> Option<PathBuf> {
    let options = HashMap::from([("current_name", Value::from(current_name))]);
    choose("SaveFile", title, options).await?.into_iter().next()
}

pub async fn open_uri(uri: &str, ask: bool) {
    let result = async {
        let connection = zbus::Connection::session().await?;
        let proxy = zbus::Proxy::new(
            &connection,
            PORTAL,
            PORTAL_PATH,
            "org.freedesktop.portal.OpenURI",
        )
        .await?;
        let options = HashMap::from([("ask", Value::from(ask))]);
        let _: OwnedObjectPath = proxy.call("OpenURI", &("", uri, options)).await?;
        zbus::Result::Ok(())
    }
    .await;
    if let Err(error) = result {
        tracing::warn!(%error, "BarrierKit: the open URI portal failed");
    }
}

pub async fn notify(id: &str, title: &str, body: &str) {
    let result = async {
        let connection = zbus::Connection::session().await?;
        let proxy = zbus::Proxy::new(
            &connection,
            PORTAL,
            PORTAL_PATH,
            "org.freedesktop.portal.Notification",
        )
        .await?;
        let notification =
            HashMap::from([("title", Value::from(title)), ("body", Value::from(body))]);
        proxy
            .call::<_, _, ()>("AddNotification", &(id, notification))
            .await
    }
    .await;
    if let Err(error) = result {
        tracing::warn!(%error, "BarrierKit: the notification portal failed");
    }
}

async fn choose(
    method: &str,
    title: &str,
    options: HashMap<&str, Value<'_>>,
) -> Option<Vec<PathBuf>> {
    match request(method, title, options).await {
        Ok(paths) => paths,
        Err(error) => {
            tracing::warn!(%error, "BarrierKit: the file chooser portal failed");
            None
        }
    }
}

// The Response signal is subscribed before the call so a fast portal cannot answer first
// (https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Request.html).
async fn request(
    method: &str,
    title: &str,
    mut options: HashMap<&str, Value<'_>>,
) -> zbus::Result<Option<Vec<PathBuf>>> {
    let connection = zbus::Connection::session().await?;
    let token = format!("barrierkit{}", NEXT_REQUEST.fetch_add(1, Ordering::Relaxed));
    let sender = connection
        .unique_name()
        .map(|name| name.trim_start_matches(':').replace('.', "_"))
        .unwrap_or_default();
    let handle = format!("{PORTAL_PATH}/request/{sender}/{token}");
    let request = zbus::Proxy::new(
        &connection,
        PORTAL,
        handle.as_str(),
        "org.freedesktop.portal.Request",
    )
    .await?;
    let mut responses = request.receive_signal("Response").await?;
    options.insert("handle_token", Value::from(token.as_str()));
    let chooser = zbus::Proxy::new(
        &connection,
        PORTAL,
        PORTAL_PATH,
        "org.freedesktop.portal.FileChooser",
    )
    .await?;
    let _: OwnedObjectPath = chooser.call(method, &("", title, options)).await?;
    let Some(message) = responses.next().await else {
        return Ok(None);
    };
    let (code, results): (u32, HashMap<String, OwnedValue>) = message.body().deserialize()?;
    if code != 0 {
        return Ok(None);
    }
    let uris = results
        .get("uris")
        .and_then(|uris| Vec::<String>::try_from(uris.clone()).ok())
        .unwrap_or_default();
    Ok(Some(uris.iter().filter_map(|uri| file_path(uri)).collect()))
}

pub fn file_path(uri: &str) -> Option<PathBuf> {
    let encoded = uri.strip_prefix("file://")?;
    let bytes = encoded.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let hex = bytes
            .get(index + 1..index + 3)
            .and_then(|pair| std::str::from_utf8(pair).ok())
            .and_then(|pair| u8::from_str_radix(pair, 16).ok());
        match (bytes[index], hex) {
            (b'%', Some(byte)) => {
                decoded.push(byte);
                index += 3;
            }
            (byte, _) => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    use std::os::unix::ffi::OsStringExt;
    Some(PathBuf::from(std::ffi::OsString::from_vec(decoded)))
}

#[cfg(test)]
mod tests {
    use super::file_path;

    #[test]
    fn file_uris_decode_their_percent_escapes_and_other_schemes_are_refused() {
        assert_eq!(
            file_path("file:///home/ana/Mis%20fotos/a%C3%B1o.png").unwrap(),
            std::path::PathBuf::from("/home/ana/Mis fotos/año.png")
        );
        assert!(file_path("https://example.org/a").is_none());
    }
}

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use futures_util::StreamExt;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

use crate::print::PageSetup;

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
async fn subscribe(
    connection: &zbus::Connection,
) -> zbus::Result<(String, zbus::proxy::SignalStream<'static>)> {
    let token = format!("barrierkit{}", NEXT_REQUEST.fetch_add(1, Ordering::Relaxed));
    let sender = connection
        .unique_name()
        .map(|name| name.trim_start_matches(':').replace('.', "_"))
        .unwrap_or_default();
    let handle = format!("{PORTAL_PATH}/request/{sender}/{token}");
    let request = zbus::Proxy::new_owned(
        connection.clone(),
        PORTAL,
        handle,
        "org.freedesktop.portal.Request",
    )
    .await?;
    let responses = request.receive_signal("Response").await?;
    Ok((token, responses))
}

async fn answer(
    mut responses: zbus::proxy::SignalStream<'static>,
) -> zbus::Result<Option<HashMap<String, OwnedValue>>> {
    let Some(message) = responses.next().await else {
        return Ok(None);
    };
    let (code, results): (u32, HashMap<String, OwnedValue>) = message.body().deserialize()?;
    Ok((code == 0).then_some(results))
}

async fn portal(
    connection: &zbus::Connection,
    interface: &str,
) -> zbus::Result<zbus::Proxy<'static>> {
    zbus::Proxy::new_owned(
        connection.clone(),
        PORTAL,
        PORTAL_PATH,
        interface.to_owned(),
    )
    .await
}

async fn request(
    method: &str,
    title: &str,
    mut options: HashMap<&str, Value<'_>>,
) -> zbus::Result<Option<Vec<PathBuf>>> {
    let connection = zbus::Connection::session().await?;
    let (token, responses) = subscribe(&connection).await?;
    options.insert("handle_token", Value::from(token.as_str()));
    let chooser = portal(&connection, "org.freedesktop.portal.FileChooser").await?;
    let _: OwnedObjectPath = chooser.call(method, &("", title, options)).await?;
    let Some(results) = answer(responses).await? else {
        return Ok(None);
    };
    let uris = results
        .get("uris")
        .and_then(|uris| Vec::<String>::try_from(uris.clone()).ok())
        .unwrap_or_default();
    Ok(Some(uris.iter().filter_map(|uri| file_path(uri)).collect()))
}

pub async fn print(title: &str, document: impl FnOnce(&PageSetup) -> Vec<u8>) {
    if let Err(error) = print_document(title, document).await {
        tracing::warn!(%error, "BarrierKit: the print portal failed");
    }
}

async fn print_document(
    title: &str,
    document: impl FnOnce(&PageSetup) -> Vec<u8>,
) -> zbus::Result<()> {
    let connection = zbus::Connection::session().await?;
    let printing = portal(&connection, "org.freedesktop.portal.Print").await?;
    let (token, responses) = subscribe(&connection).await?;
    let empty: HashMap<&str, Value<'_>> = HashMap::new();
    let options = HashMap::from([("handle_token", Value::from(token.as_str()))]);
    let _: OwnedObjectPath = printing
        .call("PreparePrint", &("", title, &empty, &empty, options))
        .await?;
    let Some(prepared) = answer(responses).await? else {
        return Ok(());
    };
    let setup = prepared
        .get("page-setup")
        .and_then(|setup| HashMap::<String, OwnedValue>::try_from(setup.clone()).ok())
        .map(|setup| page_setup(&setup))
        .unwrap_or_default();
    let pdf = document(&setup);
    let mut file = tempfile()?;
    std::io::Write::write_all(&mut file, &pdf)?;
    std::io::Seek::rewind(&mut file)?;
    let (token, responses) = subscribe(&connection).await?;
    let mut options = HashMap::from([("handle_token", Value::from(token.as_str()))]);
    if let Some(prepared) = prepared.get("token").and_then(|t| u32::try_from(t).ok()) {
        options.insert("token", Value::from(prepared));
    }
    let fd = zbus::zvariant::Fd::from(&file);
    let _: OwnedObjectPath = printing.call("Print", &("", title, fd, options)).await?;
    answer(responses).await?;
    Ok(())
}

fn tempfile() -> std::io::Result<std::fs::File> {
    let path = std::env::temp_dir().join(format!(
        "barrierkit-print-{}-{}.pdf",
        std::process::id(),
        NEXT_REQUEST.fetch_add(1, Ordering::Relaxed)
    ));
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&path)?;
    std::fs::remove_file(&path)?;
    Ok(file)
}

fn page_setup(setup: &HashMap<String, OwnedValue>) -> PageSetup {
    let number = |key: &str| setup.get(key).and_then(|value| f64::try_from(value).ok());
    let landscape = setup
        .get("Orientation")
        .and_then(|value| String::try_from(value.clone()).ok())
        .is_some_and(|orientation| orientation.ends_with("landscape"));
    let fallback = PageSetup::default();
    let (width, height) = (
        number("Width").map_or(fallback.width, |w| w as f32),
        number("Height").map_or(fallback.height, |h| h as f32),
    );
    let margin = |key: &str, default: f32| number(key).map_or(default, |m| m as f32);
    PageSetup {
        width: if landscape { height } else { width },
        height: if landscape { width } else { height },
        margins: [
            margin("MarginTop", fallback.margins[0]),
            margin("MarginRight", fallback.margins[1]),
            margin("MarginBottom", fallback.margins[2]),
            margin("MarginLeft", fallback.margins[3]),
        ],
    }
}

pub fn file_uri(path: &std::path::Path) -> String {
    use std::os::unix::ffi::OsStrExt;
    let mut uri = String::from("file://");
    for &byte in path.as_os_str().as_bytes() {
        if byte.is_ascii_alphanumeric() || b"/-._~".contains(&byte) {
            uri.push(byte as char);
        } else {
            uri.push_str(&format!("%{byte:02X}"));
        }
    }
    uri
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
    use super::{file_path, file_uri};

    #[test]
    fn file_uris_decode_their_percent_escapes_and_other_schemes_are_refused() {
        assert_eq!(
            file_path("file:///home/ana/Mis%20fotos/a%C3%B1o.png").unwrap(),
            std::path::PathBuf::from("/home/ana/Mis fotos/año.png")
        );
        assert!(file_path("https://example.org/a").is_none());
    }

    #[test]
    fn a_path_survives_the_round_trip_through_a_file_uri() {
        let path = std::path::Path::new("/home/ana/Mis fotos/año #1.png");
        assert_eq!(
            file_uri(path),
            "file:///home/ana/Mis%20fotos/a%C3%B1o%20%231.png"
        );
        assert_eq!(file_path(&file_uri(path)).unwrap(), path);
    }
}

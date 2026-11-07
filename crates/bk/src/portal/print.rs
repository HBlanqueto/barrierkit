use std::collections::HashMap;
use std::sync::atomic::Ordering;

use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

use super::{NEXT_REQUEST, answer, portal, subscribe};
use crate::print::PageSetup;

pub async fn print(title: &str, document: impl FnOnce(&PageSetup) -> Vec<u8>) {
    if let Err(error) = print_document(title, document).await {
        tracing::warn!(%error, "BarrierKit: the print portal failed");
    }
}

async fn print_document(
    title: &str,
    document: impl FnOnce(&PageSetup) -> Vec<u8>,
) -> zbus::Result<()> {
    let connection = crate::session_bus::connection().await?;
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

// XDG Base Directory: $XDG_RUNTIME_DIR is the user's alone (mode 0700); the file is 0600 anyway.
fn tempfile() -> std::io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;

    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let path = dir.join(format!(
        "barrierkit-print-{}-{}.pdf",
        std::process::id(),
        NEXT_REQUEST.fetch_add(1, Ordering::Relaxed)
    ));
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
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

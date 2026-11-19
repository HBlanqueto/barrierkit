use std::sync::OnceLock;
use std::time::Duration;

// libdbus and GDBus give up on a method reply after 25 s (DBUS_TIMEOUT_USE_DEFAULT); zbus waits forever.
const METHOD_TIMEOUT: Duration = Duration::from_secs(25);

static SESSION: OnceLock<zbus::Connection> = OnceLock::new();

pub(crate) async fn connection() -> zbus::Result<zbus::Connection> {
    if let Some(connection) = SESSION.get() {
        return Ok(connection.clone());
    }
    let connection = private_connection().await?;
    Ok(SESSION.get_or_init(|| connection).clone())
}

pub(crate) async fn private_connection() -> zbus::Result<zbus::Connection> {
    zbus::connection::Builder::session()?
        .method_timeout(METHOD_TIMEOUT)
        .build()
        .await
}

pub(crate) fn blocking() -> zbus::Result<zbus::blocking::Connection> {
    async_io::block_on(connection()).map(Into::into)
}

pub(crate) async fn export<I: zbus::object_server::Interface>(
    connection: &zbus::Connection,
    path: &str,
    interface: I,
) -> bool {
    match connection.object_server().at(path, interface).await {
        Ok(true) => true,
        Ok(false) => {
            tracing::warn!(
                path,
                "BarrierKit: another D-Bus object already serves this path"
            );
            false
        }
        Err(error) => {
            tracing::warn!(%error, path, "BarrierKit: could not export a D-Bus object");
            false
        }
    }
}

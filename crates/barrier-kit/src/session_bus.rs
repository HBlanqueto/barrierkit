use std::sync::OnceLock;
use std::time::Duration;

// libdbus and GDBus give up on a method reply after 25 s (DBUS_TIMEOUT_USE_DEFAULT); zbus waits forever.
const METHOD_TIMEOUT: Duration = Duration::from_secs(25);

static SESSION: OnceLock<zbus::Connection> = OnceLock::new();

pub(crate) async fn connection() -> zbus::Result<zbus::Connection> {
    if let Some(connection) = SESSION.get() {
        return Ok(connection.clone());
    }
    let connection = zbus::connection::Builder::session()?
        .method_timeout(METHOD_TIMEOUT)
        .build()
        .await?;
    Ok(SESSION.get_or_init(|| connection).clone())
}

pub(crate) fn blocking() -> zbus::Result<zbus::blocking::Connection> {
    async_io::block_on(connection()).map(Into::into)
}

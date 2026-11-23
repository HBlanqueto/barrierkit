use std::fs::File;
use std::io;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::thread;
use std::time::Duration;

// Launch Services already keeps a bundled app to one process; this also holds a bare binary and `open -n`. The user's
// temporary directory is per user, so each user has an instance of their own.
pub(crate) fn claim(app_id: &str, activated: impl Fn() + Send + 'static) -> io::Result<bool> {
    claim_in(&std::env::temp_dir(), app_id, activated)
}

// A connection to the socket is the activation; the lock is released by the system when the process ends.
fn claim_in(dir: &Path, app_id: &str, activated: impl Fn() + Send + 'static) -> io::Result<bool> {
    let lock = File::create(dir.join(format!("{app_id}.lock")))?;
    let socket = dir.join(format!("{app_id}.sock"));
    match lock.try_lock() {
        Ok(()) => {}
        Err(std::fs::TryLockError::WouldBlock) => {
            connect(&socket)?;
            return Ok(false);
        }
        Err(std::fs::TryLockError::Error(error)) => return Err(error),
    }
    let _ = std::fs::remove_file(&socket);
    let listener = UnixListener::bind(&socket)?;
    thread::spawn(move || {
        let _lock = lock;
        for _ in listener.incoming().flatten() {
            activated();
        }
    });
    Ok(true)
}

// The first instance holds the lock a moment before it binds the socket.
fn connect(socket: &Path) -> io::Result<UnixStream> {
    let mut attempt = 0;
    loop {
        match UnixStream::connect(socket) {
            Err(error)
                if attempt < 20
                    && matches!(
                        error.kind(),
                        io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused
                    ) =>
            {
                attempt += 1;
                thread::sleep(Duration::from_millis(50));
            }
            other => return other,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("bk-instance-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_first_launch_is_primary_and_a_second_one_activates_it() {
        let dir = scratch("second");
        let (sent, received) = mpsc::channel();
        assert!(claim_in(&dir, "app", move || sent.send(()).unwrap()).unwrap());
        assert!(!claim_in(&dir, "app", || {}).unwrap());
        received.recv_timeout(Duration::from_secs(2)).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn two_apps_do_not_share_an_instance() {
        let dir = scratch("apps");
        assert!(claim_in(&dir, "one", || {}).unwrap());
        assert!(claim_in(&dir, "two", || {}).unwrap());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_stale_socket_does_not_stop_the_first_launch() {
        let dir = scratch("stale");
        std::fs::write(dir.join("app.sock"), b"").unwrap();
        assert!(claim_in(&dir, "app", || {}).unwrap());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

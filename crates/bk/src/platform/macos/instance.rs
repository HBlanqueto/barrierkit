use std::fs::File;
use std::io;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::app::{Requests, present};

// Launch Services already keeps a bundled app to one process; this also holds a bare binary and `open -n`. The user's
// temporary directory is per user, so each user has an instance of their own.
pub type Error = io::Error;

pub(crate) fn claim(app_id: &str, requests: Arc<Mutex<Requests>>) -> io::Result<bool> {
    claim_in(&std::env::temp_dir(), app_id, move || {
        present(&requests, None)
    })
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

use std::sync::{Arc, Mutex};

use freya_core::prelude::try_consume_root_context;

use crate::app::MenuActions;
use crate::document::{OpenedUrl, deliver_open};
use crate::platform::{Timers, WakeHandle, WindowControls, instance};
use crate::portal::file_path;

pub enum Launch {
    Primary(Instance),
    Forwarded,
}

#[derive(Default)]
pub(crate) struct Requests {
    pub(crate) tokens: Vec<Option<String>>,
    pub(crate) actions: Vec<String>,
    pub(crate) uris: Vec<String>,
    pub(crate) wake: Option<WakeHandle>,
}

#[derive(Clone)]
pub struct Instance {
    requests: Arc<Mutex<Requests>>,
}

impl Instance {
    pub fn listen(&self) {
        let requests = self.requests.clone();
        let controls = WindowControls::get();
        let menu_actions = try_consume_root_context::<MenuActions>();
        let wake = Timers::get().wakeup(move || {
            let (tokens, actions, uris) = requests
                .lock()
                .map(|mut pending| {
                    (
                        std::mem::take(&mut pending.tokens),
                        std::mem::take(&mut pending.actions),
                        std::mem::take(&mut pending.uris),
                    )
                })
                .unwrap_or_default();
            for uri in uris {
                deliver_open(match file_path(&uri) {
                    Some(path) => OpenedUrl::File(path),
                    None => OpenedUrl::Other(uri),
                });
            }
            for token in tokens {
                match token {
                    Some(token) => controls.present(token),
                    None => controls.present_without_token(),
                }
            }
            if let Some(menu_actions) = &menu_actions {
                for action in actions {
                    menu_actions.dispatch(&action);
                }
            }
        });
        wake.wake();
        if let Ok(mut pending) = self.requests.lock() {
            pending.wake = Some(wake);
        }
    }
}

pub(crate) fn present(requests: &Mutex<Requests>, token: Option<String>) {
    if let Ok(mut pending) = requests.lock() {
        pending.tokens.push(token);
        if let Some(wake) = &pending.wake {
            wake.wake();
        }
    }
}

pub fn launch(app_id: &str) -> Result<Launch, instance::Error> {
    let requests = Arc::new(Mutex::new(Requests::default()));
    Ok(if instance::claim(app_id, requests.clone())? {
        Launch::Primary(Instance { requests })
    } else {
        Launch::Forwarded
    })
}

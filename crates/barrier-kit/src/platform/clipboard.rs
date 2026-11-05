use std::io::{Read, Write};
use std::sync::{Arc, Mutex, OnceLock};

use freya_clipboard::copypasta::ClipboardProvider;
use sctk::data_device_manager::data_device::{DataDevice, DataDeviceHandler};
use sctk::data_device_manager::data_offer::{DataOfferHandler, DragOffer, SelectionOffer};
use sctk::data_device_manager::data_source::{CopyPasteSource, DataSourceHandler};
use sctk::data_device_manager::{DataDeviceManagerState, WritePipe};
use sctk::reexports::calloop::ping::Ping;
use sctk::reexports::client::protocol::wl_data_device::WlDataDevice;
use sctk::reexports::client::protocol::wl_data_device_manager::DndAction;
use sctk::reexports::client::protocol::wl_data_source::WlDataSource;
use sctk::reexports::client::protocol::wl_surface::WlSurface;
use sctk::reexports::client::{Connection, QueueHandle};

use super::runtime::Runtime;

const TEXT_MIMES: [&str; 3] = ["text/plain;charset=utf-8", "text/plain", "UTF8_STRING"];

#[derive(Default)]
struct Shared {
    own: Option<String>,
    to_publish: bool,
    foreign: Option<Arc<OnceLock<String>>>,
}

#[derive(Clone)]
pub(crate) struct Clipboard {
    shared: Arc<Mutex<Shared>>,
    ping: Ping,
}

pub(crate) struct ClipboardDevice {
    pub(crate) manager: DataDeviceManagerState,
    pub(crate) device: Option<DataDevice>,
    source: Option<CopyPasteSource>,
}

impl ClipboardDevice {
    pub(crate) fn new(manager: DataDeviceManagerState) -> Self {
        Self {
            manager,
            device: None,
            source: None,
        }
    }
}

impl Clipboard {
    pub(crate) fn new(ping: Ping) -> Self {
        Self {
            shared: Arc::new(Mutex::new(Shared::default())),
            ping,
        }
    }

    pub(crate) fn provider(&self) -> Box<dyn ClipboardProvider> {
        Box::new(self.clone())
    }
}

impl ClipboardProvider for Clipboard {
    fn get_contents(&mut self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let shared = self.shared.lock().map_err(|_| "clipboard poisoned")?;
        if let Some(own) = &shared.own {
            return Ok(own.clone());
        }
        Ok(shared
            .foreign
            .as_ref()
            .and_then(|text| text.get())
            .ok_or("the clipboard is empty")?
            .clone())
    }

    fn set_contents(
        &mut self,
        text: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut shared = self.shared.lock().map_err(|_| "clipboard poisoned")?;
        shared.own = Some(text);
        shared.to_publish = true;
        drop(shared);
        self.ping.ping();
        Ok(())
    }
}

impl Runtime {
    pub(crate) fn attach_clipboard(
        &mut self,
        seat: &sctk::reexports::client::protocol::wl_seat::WlSeat,
    ) {
        if let Some(data) = self.clipboard_device.as_mut() {
            data.device = Some(data.manager.get_data_device(&self.qh, seat));
        }
    }

    pub(crate) fn publish_clipboard(&mut self) {
        let Ok(mut shared) = self.clipboard.shared.lock() else {
            return;
        };
        if !std::mem::take(&mut shared.to_publish) {
            return;
        }
        let Some(data) = self.clipboard_device.as_mut() else {
            return;
        };
        let Some(device) = &data.device else {
            return;
        };
        let source = data.manager.create_copy_paste_source(&self.qh, TEXT_MIMES);
        source.set_selection(device, self.last_input_serial.get());
        data.source = Some(source);
    }
}

fn receive_text(offer: &SelectionOffer) -> Option<Arc<OnceLock<String>>> {
    let mime = offer.with_mime_types(|offered| {
        TEXT_MIMES
            .into_iter()
            .find(|mime| offered.iter().any(|offered| offered == mime))
    })?;
    let mut pipe = offer.receive(mime.to_string()).ok()?;
    let text = Arc::new(OnceLock::new());
    let slot = text.clone();
    std::thread::spawn(move || {
        let mut read = String::new();
        if pipe.read_to_string(&mut read).is_ok() {
            let _ = slot.set(read);
        }
    });
    Some(text)
}

impl DataDeviceHandler for Runtime {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlDataDevice,
        _: f64,
        _: f64,
        _: &WlSurface,
    ) {
    }

    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {}

    fn motion(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice, _: f64, _: f64) {}

    fn selection(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {
        let offer = self
            .clipboard_device
            .as_ref()
            .and_then(|data| data.device.as_ref())
            .and_then(|device| device.data().selection_offer());
        let foreign = offer.as_ref().and_then(receive_text);
        if let Ok(mut shared) = self.clipboard.shared.lock() {
            shared.foreign = foreign;
        }
    }

    fn drop_performed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {}
}

impl DataOfferHandler for Runtime {
    fn source_actions(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &mut DragOffer,
        _: DndAction,
    ) {
    }

    fn selected_action(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &mut DragOffer,
        _: DndAction,
    ) {
    }
}

impl DataSourceHandler for Runtime {
    fn accept_mime(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlDataSource,
        _: Option<String>,
    ) {
    }

    fn send_request(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlDataSource,
        _: String,
        mut pipe: WritePipe,
    ) {
        let own = self
            .clipboard
            .shared
            .lock()
            .ok()
            .and_then(|shared| shared.own.clone());
        if let Some(text) = own {
            std::thread::spawn(move || {
                if let Err(error) = pipe.write_all(text.as_bytes()) {
                    tracing::warn!(%error, "portapapeles: no se pudo enviar el texto");
                }
            });
        }
    }

    fn cancelled(&mut self, _: &Connection, _: &QueueHandle<Self>, source: &WlDataSource) {
        let Some(data) = self.clipboard_device.as_mut() else {
            return;
        };
        if data
            .source
            .as_ref()
            .is_some_and(|current| current.inner() == source)
        {
            data.source = None;
            if let Ok(mut shared) = self.clipboard.shared.lock() {
                shared.own = None;
            }
        }
    }

    fn dnd_dropped(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataSource) {}

    fn dnd_finished(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataSource) {}

    fn action(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataSource, _: DndAction) {}
}

sctk::delegate_data_device!(Runtime);

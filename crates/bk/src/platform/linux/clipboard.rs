use std::io::{Read, Write};
use std::sync::{Arc, Mutex, OnceLock};

use freya_clipboard::copypasta::ClipboardProvider;
use sctk::data_device_manager::data_device::{DataDevice, DataDeviceHandler};
use sctk::data_device_manager::data_offer::{DataOfferHandler, DragOffer, SelectionOffer};
use sctk::data_device_manager::data_source::{CopyPasteSource, DataSourceHandler, DragSource};
use sctk::data_device_manager::{DataDeviceManagerState, WritePipe};
use sctk::reexports::calloop::ping::Ping;
use sctk::reexports::client::protocol::wl_data_device::WlDataDevice;
use sctk::reexports::client::protocol::wl_data_device_manager::DndAction;
use sctk::reexports::client::protocol::wl_data_source::WlDataSource;
use sctk::reexports::client::protocol::wl_surface::WlSurface;
use sctk::reexports::client::{Connection, QueueHandle};

use std::path::PathBuf;

use freya_core::integration::{FileEventName, PlatformEvent};
use sctk::reexports::client::Proxy;
use sctk::reexports::client::backend::ObjectId;
use torin::prelude::CursorPoint;

use super::runtime::Runtime;
use super::surface::physical_point;
use crate::drag::{DragPayload, URI_LIST};
use crate::portal::file_uri;

const HTML: &str = "text/html";
const PNG: &str = "image/png";
pub(super) const TEXT_MIMES: [&str; 3] = ["text/plain;charset=utf-8", "text/plain", "UTF8_STRING"];
const MAX_RECEIVED: u64 = 16 << 20;

pub(crate) struct PendingDrop {
    target: (ObjectId, CursorPoint),
    offer: DragOffer,
    uris: Arc<OnceLock<String>>,
}

pub(crate) fn uri_list_paths(list: &str) -> Vec<PathBuf> {
    list.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter_map(crate::portal::file_path)
        .collect()
}

impl Runtime {
    fn drag_offer(&self) -> Option<DragOffer> {
        self.clipboard_device
            .as_ref()
            .and_then(|data| data.device.as_ref())
            .and_then(|device| device.data().drag_offer())
    }

    fn push_file(&mut self, name: FileEventName, file_path: Option<PathBuf>) {
        let Some((surface, cursor)) = self.drag_target.clone() else {
            return;
        };
        if let Some(index) = self.surface_index_by_id(&surface) {
            self.surfaces[index].app.push(PlatformEvent::File {
                name,
                cursor,
                file_paths: file_path.into_iter().collect(),
            });
            self.surfaces[index].needs_redraw = true;
        }
    }

    fn hover(&mut self, surface: &WlSurface, x: f64, y: f64) {
        let Some(index) = self.surface_for(surface) else {
            return;
        };
        let cursor = physical_point((x, y), self.surfaces[index].scale);
        self.drag_target = Some((surface.id(), cursor));
        self.push_file(FileEventName::FileHover, None);
    }

    pub(crate) fn start_drag(&mut self, surface: &ObjectId, payload: DragPayload) {
        let Some(index) = self.surface_index_by_id(surface) else {
            return;
        };
        let serial = self.last_input_serial.get();
        let origin = self.surfaces[index].surface.clone();
        let Some(data) = self.clipboard_device.as_mut() else {
            return;
        };
        let Some(device) = data.device.as_ref() else {
            return;
        };
        let source =
            data.manager
                .create_drag_and_drop_source(&self.qh, payload.mimes(), DndAction::Copy);
        source.start_drag(device, &origin, None, serial);
        data.drag = Some((source, payload));
    }

    fn end_drag(&mut self, source: &WlDataSource) {
        if let Some(data) = self.clipboard_device.as_mut()
            && data
                .drag
                .as_ref()
                .is_some_and(|(drag, _)| drag.inner() == source)
        {
            data.drag = None;
        }
    }

    pub(crate) fn deliver_drop(&mut self) {
        let Some(list) = self
            .pending_drop
            .as_ref()
            .and_then(|pending| pending.uris.get().cloned())
        else {
            return;
        };
        let Some(pending) = self.pending_drop.take() else {
            return;
        };
        self.drag_target = Some(pending.target);
        for path in uri_list_paths(&list) {
            self.push_file(FileEventName::FileDrop, Some(path));
        }
        pending.offer.finish();
        pending.offer.destroy();
        self.drag_target = None;
    }
}

#[derive(Default)]
struct Shared {
    own: Option<String>,
    own_html: Option<String>,
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
    drag: Option<(DragSource, DragPayload)>,
}

impl ClipboardDevice {
    pub(crate) fn new(manager: DataDeviceManagerState) -> Self {
        Self {
            manager,
            device: None,
            source: None,
            drag: None,
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

    pub(crate) fn set_rich(&self, text: String, html: String) {
        if let Ok(mut shared) = self.shared.lock() {
            shared.own = Some(text);
            shared.own_html = Some(html);
            shared.to_publish = true;
        }
        self.ping.ping();
    }

    pub(crate) fn take_publish(&self) -> bool {
        self.shared
            .lock()
            .is_ok_and(|mut shared| std::mem::take(&mut shared.to_publish))
    }

    pub(crate) fn own(&self) -> Option<String> {
        self.shared
            .lock()
            .ok()
            .and_then(|shared| shared.own.clone())
    }

    pub(crate) fn clear_own(&self) {
        if let Ok(mut shared) = self.shared.lock() {
            shared.own = None;
        }
    }

    pub(crate) fn set_foreign(&self, foreign: Option<Arc<OnceLock<String>>>) {
        if let Ok(mut shared) = self.shared.lock() {
            shared.foreign = foreign;
        }
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
        shared.own_html = None;
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
        let html = shared.own_html.is_some().then_some(HTML);
        drop(shared);
        let Some(data) = self.clipboard_device.as_mut() else {
            return;
        };
        let Some(device) = &data.device else {
            return;
        };
        let source = data
            .manager
            .create_copy_paste_source(&self.qh, TEXT_MIMES.into_iter().chain(html));
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
    offer.receive(mime.to_string()).ok().map(read_in_background)
}

// A peer chooses what it sends: the read stops at `MAX_RECEIVED` bytes.
fn read_capped(pipe: sctk::data_device_manager::ReadPipe) -> Option<String> {
    let mut bytes = Vec::new();
    pipe.take(MAX_RECEIVED).read_to_end(&mut bytes).ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

pub(crate) fn read_in_background(
    pipe: sctk::data_device_manager::ReadPipe,
) -> Arc<OnceLock<String>> {
    let text = Arc::new(OnceLock::new());
    let slot = text.clone();
    std::thread::spawn(move || {
        if let Some(read) = read_capped(pipe) {
            let _ = slot.set(read);
        }
    });
    text
}

impl DataDeviceHandler for Runtime {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlDataDevice,
        x: f64,
        y: f64,
        surface: &WlSurface,
    ) {
        let Some(offer) = self.drag_offer() else {
            return;
        };
        let has_files = offer.with_mime_types(|mimes| mimes.iter().any(|mime| mime == URI_LIST));
        offer.accept_mime_type(offer.serial, has_files.then(|| URI_LIST.to_owned()));
        if has_files {
            offer.set_actions(DndAction::Copy, DndAction::Copy);
            self.hover(surface, x, y);
        }
    }

    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {
        self.push_file(FileEventName::FileHoverCancelled, None);
        self.drag_target = None;
    }

    fn motion(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice, x: f64, y: f64) {
        if self.drag_target.is_some()
            && let Some(offer) = self.drag_offer()
        {
            self.hover(&offer.surface, x, y);
        }
    }

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

    fn drop_performed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {
        let (Some(offer), Some(target)) = (self.drag_offer(), self.drag_target.clone()) else {
            return;
        };
        let Ok(pipe) = offer.receive(URI_LIST.to_owned()) else {
            return;
        };
        let uris = Arc::new(OnceLock::new());
        let (slot, ping) = (uris.clone(), self.ping.clone());
        std::thread::spawn(move || {
            let _ = slot.set(read_capped(pipe).unwrap_or_default());
            ping.ping();
        });
        self.pending_drop = Some(PendingDrop {
            target,
            offer,
            uris,
        });
    }
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
        source: &WlDataSource,
        mime: String,
        mut pipe: WritePipe,
    ) {
        let dragged = self
            .clipboard_device
            .as_ref()
            .and_then(|data| data.drag.as_ref())
            .filter(|(drag, _)| drag.inner() == source)
            .map(|(_, payload)| payload.bytes(&mime));
        if let Some(bytes) = dragged {
            std::thread::spawn(move || {
                if let Err(error) = pipe.write_all(&bytes) {
                    tracing::warn!(%error, "BarrierKit: could not send the dragged data");
                }
            });
            return;
        }
        let own = self.clipboard.shared.lock().ok().and_then(|shared| {
            if mime == HTML {
                shared.own_html.clone()
            } else {
                shared.own.clone()
            }
        });
        if let Some(text) = own {
            std::thread::spawn(move || {
                if let Err(error) = pipe.write_all(text.as_bytes()) {
                    tracing::warn!(%error, "clipboard: could not send the text");
                }
            });
        }
    }

    fn cancelled(&mut self, _: &Connection, _: &QueueHandle<Self>, source: &WlDataSource) {
        self.end_drag(source);
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
                shared.own_html = None;
            }
        }
    }

    fn dnd_dropped(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataSource) {}

    fn dnd_finished(&mut self, _: &Connection, _: &QueueHandle<Self>, source: &WlDataSource) {
        self.end_drag(source);
    }

    fn action(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataSource, _: DndAction) {}
}

impl DragPayload {
    pub(crate) fn mimes(&self) -> Vec<&'static str> {
        match self {
            DragPayload::Files(_) => std::iter::once(URI_LIST).chain(TEXT_MIMES).collect(),
            DragPayload::Text(_) => TEXT_MIMES.to_vec(),
            DragPayload::Html { .. } => TEXT_MIMES.into_iter().chain([HTML]).collect(),
            DragPayload::Image(_) => vec![PNG],
        }
    }

    pub(crate) fn bytes(&self, mime: &str) -> Vec<u8> {
        match self {
            DragPayload::Files(paths) if mime == URI_LIST => paths
                .iter()
                .map(|path| file_uri(path) + "\r\n")
                .collect::<String>()
                .into_bytes(),
            DragPayload::Files(paths) => paths
                .iter()
                .map(|path| path.to_string_lossy())
                .collect::<Vec<_>>()
                .join("\n")
                .into_bytes(),
            DragPayload::Html { html, .. } if mime == HTML => html.clone().into_bytes(),
            DragPayload::Text(text) | DragPayload::Html { text, .. } => text.clone().into_bytes(),
            DragPayload::Image(bytes) => bytes.clone(),
        }
    }
}

sctk::delegate_data_device!(Runtime);

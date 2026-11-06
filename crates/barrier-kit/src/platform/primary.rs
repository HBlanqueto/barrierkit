use std::io::Write;

use sctk::data_device_manager::WritePipe;
use sctk::primary_selection::PrimarySelectionManagerState;
use sctk::primary_selection::device::{PrimarySelectionDevice, PrimarySelectionDeviceHandler};
use sctk::primary_selection::selection::{PrimarySelectionSource, PrimarySelectionSourceHandler};
use sctk::reexports::client::protocol::wl_seat::WlSeat;
use sctk::reexports::client::{Connection, QueueHandle};
use sctk::reexports::protocols::wp::primary_selection::zv1::client::zwp_primary_selection_device_v1::ZwpPrimarySelectionDeviceV1;
use sctk::reexports::protocols::wp::primary_selection::zv1::client::zwp_primary_selection_source_v1::ZwpPrimarySelectionSourceV1;

use super::clipboard::read_in_background;
use super::runtime::Runtime;
use crate::drag::TEXT_MIMES;

pub(crate) struct PrimaryDevice {
    manager: PrimarySelectionManagerState,
    device: Option<PrimarySelectionDevice>,
    source: Option<PrimarySelectionSource>,
}

impl PrimaryDevice {
    pub(crate) fn new(manager: PrimarySelectionManagerState) -> Self {
        Self {
            manager,
            device: None,
            source: None,
        }
    }
}

impl Runtime {
    pub(crate) fn attach_primary(&mut self, seat: &WlSeat) {
        if let Some(primary) = self.primary_device.as_mut() {
            primary.device = Some(primary.manager.get_selection_device(&self.qh, seat));
        }
    }

    pub(crate) fn publish_primary(&mut self) {
        if !self.primary.take_publish() {
            return;
        }
        let Some(primary) = self.primary_device.as_mut() else {
            return;
        };
        let Some(device) = &primary.device else {
            return;
        };
        let source = primary
            .manager
            .create_selection_source(&self.qh, TEXT_MIMES);
        source.set_selection(device, self.last_input_serial.get());
        primary.source = Some(source);
    }
}

impl PrimarySelectionDeviceHandler for Runtime {
    fn selection(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &ZwpPrimarySelectionDeviceV1,
    ) {
        let offer = self
            .primary_device
            .as_ref()
            .and_then(|primary| primary.device.as_ref())
            .and_then(|device| device.data().selection_offer());
        let foreign = offer.and_then(|offer| {
            let mime = offer.with_mime_types(|offered| {
                TEXT_MIMES
                    .into_iter()
                    .find(|mime| offered.iter().any(|offered| offered == mime))
            })?;
            offer.receive(mime.to_string()).ok().map(read_in_background)
        });
        self.primary.set_foreign(foreign);
    }
}

impl PrimarySelectionSourceHandler for Runtime {
    fn send_request(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &ZwpPrimarySelectionSourceV1,
        _: String,
        mut pipe: WritePipe,
    ) {
        if let Some(text) = self.primary.own() {
            std::thread::spawn(move || {
                if let Err(error) = pipe.write_all(text.as_bytes()) {
                    tracing::warn!(%error, "primary selection: could not send the text");
                }
            });
        }
    }

    fn cancelled(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        source: &ZwpPrimarySelectionSourceV1,
    ) {
        let Some(primary) = self.primary_device.as_mut() else {
            return;
        };
        if primary
            .source
            .as_ref()
            .is_some_and(|current| current.inner() == source)
        {
            primary.source = None;
            self.primary.clear_own();
        }
    }
}

sctk::delegate_primary_selection!(Runtime);

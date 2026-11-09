use sctk::reexports::client::{Connection, Dispatch, QueueHandle};
use wayland_protocols::xdg::dialog::v1::client::xdg_dialog_v1::{self, XdgDialogV1};
use wayland_protocols::xdg::dialog::v1::client::xdg_wm_dialog_v1::{self, XdgWmDialogV1};

use super::runtime::Runtime;

impl Dispatch<XdgWmDialogV1, (), Runtime> for Runtime {
    fn event(
        _: &mut Runtime,
        _: &XdgWmDialogV1,
        _: xdg_wm_dialog_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Runtime>,
    ) {
    }
}

impl Dispatch<XdgDialogV1, (), Runtime> for Runtime {
    fn event(
        _: &mut Runtime,
        _: &XdgDialogV1,
        _: xdg_dialog_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Runtime>,
    ) {
    }
}
